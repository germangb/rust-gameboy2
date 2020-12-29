use log::info;
use sdl2::event::{Event, WindowEvent};
use wgpu::{
    CommandEncoderDescriptor, DeviceDescriptor, RenderPassColorAttachmentDescriptor,
    RenderPassDescriptor, RenderPipelineDescriptor, RequestAdapterOptions, SwapChainDescriptor,
};

#[cfg(feature = "camera")]
mod camera_sensor;

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
    .expect("Unable to get wgpu Adapter");

    // device, queue, and swap chain
    let (device, queue) = futures::executor::block_on(adapter.request_device(
        &DeviceDescriptor {
            shader_validation: true,
            ..Default::default()
        },
        None,
    ))
    .expect("Error requesting");
    let mut swap_chain = device.create_swap_chain(
        &surface,
        &SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: 640,
            height: 480,
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

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::Close,
                    ..
                } => break 'main,
                _ if !imgui_sdl2.ignore_event(&event) => {
                    imgui_sdl2.handle_event(&mut imgui, &event);
                }
                _ => {}
            }
        }

        let frame = swap_chain
            .get_current_frame()
            .expect("Error requesting frame from swap chain");

        // imgui UI
        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let ui = imgui.frame();
        ui.show_demo_window(&mut true);
        imgui_sdl2.prepare_render(&ui, &window);

        let mut cmd = device.create_command_encoder(&CommandEncoderDescriptor::default());
        let mut pass = cmd.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[RenderPassColorAttachmentDescriptor {
                attachment: &frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: false,
                },
            }],
            depth_stencil_attachment: None,
        });
        //imgui_wgpu.render(ui.render(), &queue, &device, &mut pass);
        drop(pass);

        queue.submit(Some(cmd.finish()));
        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000 / 60));
    }
}
