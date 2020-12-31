use core::{cartridge::MBC3, device::Device, Button, GameBoy};
use log::{info, warn};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::{Mod, Scancode},
};
use wgpu::{
    CommandEncoderDescriptor, DeviceDescriptor, RenderPassColorAttachmentDescriptor,
    RenderPassDescriptor, RenderPipelineDescriptor, RequestAdapterOptions, SamplerDescriptor,
    SwapChainDescriptor, TextureCopyView, TextureDataLayout, TextureDescriptor,
    TextureViewDescriptor,
};

#[cfg(feature = "camera")]
mod camera_sensor;

const WINDOW_WIDTH: usize = 640;
const WINDOW_HEIGHT: usize = 480;
const BACKGROUND: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

/// Window states.
#[derive(Default)]
struct Windows {
    ppu: bool,
    cpu: bool,
    debug: bool,
}

struct PPU {
    overlays: bool,
    scale: u32,
}

impl Default for PPU {
    fn default() -> Self {
        Self {
            overlays: false,
            scale: 2,
        }
    }
}

enum Breakpoint {
    Address(u16),
    Line(usize),
}

#[derive(Default)]
struct Debug {
    running: bool,
    /// running field will be set to 'false' when the breakpoint is reached.
    breakpoint: Option<Breakpoint>,
}

fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter(Some("core"), log::LevelFilter::Off)
        .filter(Some("gfx"), log::LevelFilter::Trace)
        .filter(Some("gfx_memory"), log::LevelFilter::Info)
        .filter(Some(module_path!()), log::LevelFilter::Trace)
        .init();

    // init sdl windowing and events
    let sdl = sdl2::init().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("GameBoy", WINDOW_WIDTH as _, WINDOW_HEIGHT as _)
        //.borderless()
        .position_centered()
        .build()
        .unwrap();

    // init wgpu context and link to window
    let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = futures::executor::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        compatible_surface: Some(&surface),
    }))
    .expect("Unable to request Adapter");

    // device, queue, and swap chain
    let (width, height) = window.size();
    let (device, queue) =
        futures::executor::block_on(adapter.request_device(&DeviceDescriptor::default(), None))
            .expect("Error requesting logical device");
    let mut swap_chain = device.create_swap_chain(
        &surface,
        &SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: width as _,
            height: height as _,
            present_mode: wgpu::PresentMode::Fifo,
        },
    );

    info!("{:?}", adapter.get_info());
    info!("Features {:?}", device.features());
    info!("{:?}", device.limits());

    // init imgui
    let mut imgui = imgui::Context::create();
    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
    let imgui_config = imgui_wgpu::RendererConfig {
        texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
        ..Default::default()
    };
    let mut imgui_wgpu = imgui_wgpu::Renderer::new(&mut imgui, &device, &queue, imgui_config);

    // texture where emulator display will be shown
    let texture_id = imgui_wgpu.textures.insert(imgui_wgpu::Texture::new(
        &device,
        &imgui_wgpu,
        imgui_wgpu::TextureConfig {
            size: wgpu::Extent3d {
                width: core::lcd::WIDTH as _,
                height: core::lcd::HEIGHT as _,
                depth: 1,
            },
            label: None,
            format: None, // same as swap chain
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            sampler_desc: SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
        },
    ));

    // emulator
    let mut emulator = GameBoy::new(MBC3::new(include_bytes!("../../roms/gold.gbc").to_vec()))
        .expect("Error initializing emulator");

    // ui state
    let mut windows = Windows::default();
    windows.ppu = true;
    let mut ppu = PPU::default();
    let mut debug = Debug::default();

    'main: loop {
        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) {
                continue;
            }

            #[rustfmt::skip]
            let _ = match event {
                Event::Window {
                    win_event: WindowEvent::Close,
                    ..
                } => break 'main,

                Event::KeyDown { scancode: Some(Scancode::D), keymod, .. } => emulator.set_debug_overlays(!keymod.contains(Mod::LSHIFTMOD)),

                // emulator inputs
                Event::KeyDown { scancode: Some(Scancode::Z), .. } => emulator.press(&Button::A),
                Event::KeyDown { scancode: Some(Scancode::X), .. } => emulator.press(&Button::B),
                Event::KeyDown { scancode: Some(Scancode::Return), .. } => emulator.press(&Button::Start),
                Event::KeyDown { scancode: Some(Scancode::RShift), .. } => emulator.press(&Button::Select),
                Event::KeyDown { scancode: Some(Scancode::Left), .. } => emulator.press(&Button::Left),
                Event::KeyDown { scancode: Some(Scancode::Right), .. } => emulator.press(&Button::Right),
                Event::KeyDown { scancode: Some(Scancode::Down), .. } => emulator.press(&Button::Down),
                Event::KeyDown { scancode: Some(Scancode::Up), .. } => emulator.press(&Button::Up),

                Event::KeyUp { scancode: Some(Scancode::Z), .. } => emulator.release(&Button::A),
                Event::KeyUp { scancode: Some(Scancode::X), .. } => emulator.release(&Button::B),
                Event::KeyUp { scancode: Some(Scancode::Return), .. } => emulator.release(&Button::Start),
                Event::KeyUp { scancode: Some(Scancode::RShift), .. } => emulator.release(&Button::Select),
                Event::KeyUp { scancode: Some(Scancode::Left), .. } => emulator.release(&Button::Left),
                Event::KeyUp { scancode: Some(Scancode::Right), .. } => emulator.release(&Button::Right),
                Event::KeyUp { scancode: Some(Scancode::Down), .. } => emulator.release(&Button::Down),
                Event::KeyUp { scancode: Some(Scancode::Up), .. } => emulator.release(&Button::Up),

                // forward imgui events
                _ => {
                    //warn!("Unhandled SDL2 event: {:?}", event);
                }
            };
        }

        // check breakpoints
        if let Some(breakpoint) = &debug.breakpoint {
            match breakpoint {
                Breakpoint::Address(addr) => {
                    let pc = emulator.cpu().registers().pc;

                    if pc == *addr {
                        debug.running = false;
                    }
                },
                Breakpoint::Line(line) => {
                    let ly = emulator.read(0xff44).unwrap() as usize;

                    if ly == *line {
                        debug.running = false;
                    }
                },
            }
        }

        if debug.running {
            emulator.update_frame().unwrap();
        }

        // imgui ui
        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let ui = imgui.frame();
        let t = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));

        let mut quit = false; // ugly

        #[rustfmt::skip]
        ui.main_menu_bar(|| {
            ui.menu(imgui::im_str!("File"), true, || unsafe {
                if imgui::sys::igMenuItemBool("Quit\0".as_ptr() as _, std::ptr::null(), false, true) {
                    //break;
                    quit = true;
                }
            });
            ui.menu(imgui::im_str!("Emulator"), true, || unsafe {
                if imgui::sys::igMenuItemBool("PPU (Display)\0".as_ptr() as _, std::ptr::null(), windows.ppu, !windows.ppu) {
                    windows.ppu = true;
                }
                if imgui::sys::igMenuItemBool("CPU\0".as_ptr() as _, std::ptr::null(), windows.cpu, !windows.cpu) {
                    windows.cpu = true;
                }
            });
            ui.menu(imgui::im_str!("Debug"), true, || unsafe {
                if imgui::sys::igMenuItemBool("Debug\0".as_ptr() as _, std::ptr::null(), windows.debug, !windows.debug) {
                    windows.debug = true;
                }
            });
        });

        let tt = ui.push_style_vars(&[
            imgui::StyleVar::WindowBorderSize(0.0),
            //imgui::StyleVar::WindowPadding([0.0; 2]),
        ]);
        imgui::Window::new(imgui::im_str!("Program"))
            .size(
                [width as _, (height as f32) - 16.0],
                imgui::Condition::Always,
            )
            .position([0.0, 16.0], imgui::Condition::Always)
            .bg_alpha(0.0)
            .title_bar(false)
            .bring_to_front_on_focus(false)
            .collapsible(false)
            .resizable(false)
            .build(&ui, || {
                let c = ui.push_style_color(imgui::StyleColor::Text, [0.25, 0.25, 0.25, 1.0]);
                let pc = emulator.cpu().registers().pc as i32;

                let lines = (ui.window_size()[1] as i32) / 16 - 2;
                for c_pc in pc - lines / 2..pc + lines / 2 {
                    if c_pc >= 0 {
                        let opcode = emulator.read(c_pc as _).unwrap();
                        if c_pc == pc {
                            let c =
                                ui.push_style_color(imgui::StyleColor::Text, [0.5, 0.5, 0.5, 1.0]);
                            let d8 = emulator.read((c_pc + 1) as _).unwrap();
                            let d16 = emulator.read_word((c_pc + 1) as _).unwrap();
                            let r8: i8 = unsafe { std::mem::transmute(d8) };
                            ui.text(format!(
                                "{:04x}: * Opcode: {:02X}, d8: {:02X}, d16: {:04X}, r8: {}",
                                c_pc, opcode, d8, d16, r8
                            ));
                            c.pop(&ui);
                        } else {
                            ui.text(format!("{:04x}: {:02X}", c_pc, opcode));
                        }
                    } else {
                        ui.text("");
                    }
                }
                c.pop(&ui);
            });
        tt.pop(&ui);
        if windows.ppu {
            //let t = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0; 2]));
            imgui::Window::new(imgui::im_str!("PPU (Display)"))
                .opened(&mut windows.ppu)
                .always_auto_resize(true)
                .resizable(false)
                .menu_bar(true)
                .build(&ui, || {
                    #[rustfmt::skip]
                    ui.menu_bar(|| {
                        ui.menu(imgui::im_str!("IO"), true, || unsafe {
                            if imgui::sys::igMenuItemBool("LCDC\0".as_ptr() as _, std::ptr::null(), false, false) {}
                            if imgui::sys::igMenuItemBool("STAT\0".as_ptr() as _, std::ptr::null(), false, false) {}
                            ui.separator();
                            if imgui::sys::igMenuItemBool("Scroll\0".as_ptr() as _, std::ptr::null(), false, false) {}
                            if imgui::sys::igMenuItemBool("Window\0".as_ptr() as _, std::ptr::null(), false, false) {}
                            if imgui::sys::igMenuItemBool("Line\0".as_ptr() as _, std::ptr::null(), false, false) {}
                            if imgui::sys::igMenuItemBool("Color\0".as_ptr() as _, std::ptr::null(), false, false) {}
                        });
                        ui.menu(imgui::im_str!("Scale"), true, || unsafe {
                            let scales = &[
                                ("x1\0", 1),
                                ("x2\0", 2),
                                ("x3\0", 3),
                                ("x4\0", 4),
                            ];
                            for (label, scale) in scales.iter() {
                                if imgui::sys::igMenuItemBool(label.as_ptr() as _, std::ptr::null(), ppu.scale == *scale, true) {
                                    ppu.scale = *scale;
                                }
                            }
                        });
                    });
                    let width = core::lcd::WIDTH as f32 * ppu.scale as f32;
                    let height = core::lcd::HEIGHT as f32 * ppu.scale as f32;
                    imgui::Image::new(texture_id, [width, height]).build(&ui);
                });
            //t.pop(&ui);
        }
        if windows.cpu {
            imgui::Window::new(imgui::im_str!("CPU"))
                .opened(&mut windows.cpu)
                .build(&ui, || {
                    ui.checkbox(imgui::im_str!("Read-only"), &mut true);

                    let cpu = emulator.cpu();

                    ui.input_text(
                        imgui::im_str!("AF"),
                        &mut imgui::im_str!("{:04x}", cpu.registers().af()),
                    )
                    .read_only(true)
                    .build();
                    ui.input_text(
                        imgui::im_str!("BC"),
                        &mut imgui::im_str!("{:04x}", cpu.registers().bc()),
                    )
                    .read_only(true)
                    .build();
                    ui.input_text(
                        imgui::im_str!("DE"),
                        &mut imgui::im_str!("{:04x}", cpu.registers().de()),
                    )
                    .read_only(true)
                    .build();
                    ui.input_text(
                        imgui::im_str!("HL"),
                        &mut imgui::im_str!("{:04x}", cpu.registers().hl()),
                    )
                    .read_only(true)
                    .build();
                    ui.input_text(
                        imgui::im_str!("SP"),
                        &mut imgui::im_str!("{:04x}", cpu.registers().sp),
                    )
                    .read_only(true)
                    .build();
                    ui.input_text(
                        imgui::im_str!("PC"),
                        &mut imgui::im_str!("{:04x}", cpu.registers().pc),
                    )
                    .read_only(true)
                    .build();
                    ui.checkbox(imgui::im_str!("IME"), &mut cpu.ime());
                    ui.checkbox(imgui::im_str!("HALT"), &mut cpu.halt());
                });
        }

        if windows.debug {
            imgui::Window::new(imgui::im_str!("Debug"))
                .opened(&mut windows.debug)
                .build(&ui, || {
                    if debug.running {
                        if ui.small_button(imgui::im_str!("Pause")) {
                            debug.running = false;
                        }
                    } else {
                        if ui.small_button(imgui::im_str!("Resume")) {
                            debug.running = true;
                        }
                    }

                    ui.separator();
                    ui.text("Breakpoint");
                    if ui.radio_button_bool(
                        imgui::im_str!("None"),
                        matches!(&debug.breakpoint, None),
                    ) {
                        debug.breakpoint = None;
                    }
                    if ui.radio_button_bool(
                        imgui::im_str!("Address"),
                        matches!(&debug.breakpoint, Some(Breakpoint::Address(_))),
                    ) {
                        debug.breakpoint = Some(Breakpoint::Address(0));
                    }
                    if ui.radio_button_bool(
                        imgui::im_str!("Line"),
                        matches!(&debug.breakpoint, Some(Breakpoint::Line(_))),
                    ) {
                        debug.breakpoint = Some(Breakpoint::Line(0));
                    }

                    match &mut debug.breakpoint {
                        Some(Breakpoint::Address(addr)) => {
                            ui.spacing();
                            ui.text_wrapped(imgui::im_str!("Breakpoint address:"));

                            let mut addr_i32 = *addr as _;
                            let changed = ui
                                .input_int(imgui::im_str!("Address"), &mut addr_i32)
                                .chars_hexadecimal(true)
                                .build();

                            if changed {
                                // TODO input validation
                                assert!(addr_i32 >= 0);
                                assert!(addr_i32 <= 0xffff);

                                *addr = addr_i32 as _;
                            }
                        }
                        Some(Breakpoint::Line(line)) => {
                            ui.spacing();
                            ui.text_wrapped(imgui::im_str!("Breakpoint line:"));

                            let mut line_i32 = *line as _;
                            let changed =
                                ui.input_int(imgui::im_str!("Line"), &mut line_i32).build();

                            if changed {
                                // TODO input validation
                                assert!(line_i32 >= 0);
                                assert!(line_i32 <= 153);

                                *line = line_i32 as _;
                            }
                        }
                        None => {
                            ui.spacing();
                            ui.text_wrapped(imgui::im_str!(
                                "Select a Breakpoint mode for more options."
                            ));
                        }
                    }
                });
        }

        t.pop(&ui);
        imgui_sdl2.prepare_render(&ui, &window);

        let frame = swap_chain
            .get_current_frame()
            .expect("Error requesting frame from swap chain");

        queue.write_texture(
            TextureCopyView {
                texture: imgui_wgpu.textures.get(texture_id).unwrap().texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytemuck::cast_slice(&emulator.display()[..]),
            TextureDataLayout {
                offset: 0,
                bytes_per_row: (core::lcd::WIDTH * 4) as _,
                rows_per_image: core::lcd::HEIGHT as _,
            },
            wgpu::Extent3d {
                width: core::lcd::WIDTH as _,
                height: core::lcd::HEIGHT as _,
                depth: 1,
            },
        );

        let mut cmd = device.create_command_encoder(&CommandEncoderDescriptor::default());
        let mut pass = cmd.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[RenderPassColorAttachmentDescriptor {
                attachment: &frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(BACKGROUND),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        imgui_wgpu.render(ui.render(), &queue, &device, &mut pass);
        drop(pass);

        queue.submit(Some(cmd.finish()));

        if quit {
            break;
        }
    }
}
