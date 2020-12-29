use core::{cartridge::MBC3, Button, GameBoy};
use log::{info, warn};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
};
use wgpu::{
    CommandEncoderDescriptor, DeviceDescriptor, RenderPassColorAttachmentDescriptor,
    RenderPassDescriptor, RenderPipelineDescriptor, RequestAdapterOptions, SwapChainDescriptor,
    TextureCopyView, TextureDataLayout, TextureDescriptor, TextureViewDescriptor,
};

#[cfg(feature = "camera")]
mod camera_sensor;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const BACKGROUND: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

fn main() {
    pretty_env_logger::init();

    // init sdl windowing and events
    let sdl = sdl2::init().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("GameBoy (wgpu)", 640, 480)
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
    let (device, queue) =
        futures::executor::block_on(adapter.request_device(&DeviceDescriptor::default(), None))
            .expect("Error requesting logical device");
    let mut swap_chain = device.create_swap_chain(
        &surface,
        &SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: WIDTH as _,
            height: HEIGHT as _,
            present_mode: wgpu::PresentMode::Immediate,
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
        },
    ));

    // emulator
    let mut emulator = GameBoy::new(MBC3::new(
        include_bytes!("../../roms/zelda_dx.gbc").to_vec(),
    ))
    .expect("Error initializing emulator");

    'main: loop {
        for event in event_pump.poll_iter() {
            if imgui_sdl2.ignore_event(&event) {
                imgui_sdl2.handle_event(&mut imgui, &event);
                continue;
            }

            #[rustfmt::skip]
            let _ = match event {
                Event::Window {
                    win_event: WindowEvent::Close,
                    ..
                } => break 'main,

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
                    warn!("Unhandled SDL2 event: {:?}", event);
                }
            };
        }

        emulator.update_frame().unwrap();

        // imgui ui
        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let ui = imgui.frame();
        let t = ui.push_style_vars(&[
            imgui::StyleVar::WindowPadding([0.0; 2]),
            imgui::StyleVar::WindowRounding(0.0),
        ]);
        ui.main_menu_bar(|| {
            ui.menu(imgui::im_str!("File"), true, || {});
            ui.menu(imgui::im_str!("Emulator"), false, || {});
            ui.menu(imgui::im_str!("Debug"), false, || {});
        });
        imgui::Window::new(imgui::im_str!("Display"))
            .always_auto_resize(true)
            .resizable(false)
            .build(&ui, || {
                let width = core::lcd::WIDTH as f32;
                let height = core::lcd::HEIGHT as f32;
                imgui::Image::new(texture_id, [width * 2.0, height * 2.0]).build(&ui);
            });
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
    }
}
