use std::{iter, sync::Arc};

use duit::Ui;
use dume_renderer::Canvas;
use glam::Vec2;
use wgpu::*;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub fn run(
    event_loop: EventLoop<()>,
    window: Window,
    mut ui: Ui,
    init_canvas: impl FnOnce(&mut Canvas),
    // mut handle_msg: impl FnMut(Message) + 'static,
    mut update: impl FnMut(&mut Ui) + 'static,
) {
    let instance = Instance::new(Backends::all());

    let surface = unsafe { instance.create_surface(&window) };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    }))
    .expect("failed to find a suitable adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: Features::default(),
            limits: Limits::default(),
        },
        None,
    ))
    .expect("failed to get device");

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let mut swap_chain_desc = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: dume_renderer::TARGET_FORMAT,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: PresentMode::Fifo,
    };
    surface.configure(&device, &swap_chain_desc);

    let mut sample_texture = create_sample_texture(window.inner_size(), &*device);

    let mut canvas = Canvas::new(Arc::clone(&device), Arc::clone(&queue));
    init_canvas(&mut canvas);

    canvas.set_scale_factor(window.scale_factor());

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        let window_logical_size = Vec2::new(
            window.inner_size().to_logical(window.scale_factor()).width,
            window.inner_size().to_logical(window.scale_factor()).height,
        );
        match event {
            Event::RedrawRequested(_) => {
                update(&mut ui);

                ui.render(&mut canvas, window_logical_size);
                let frame = surface
                    .get_current_frame()
                    .expect("failed to get next frame");

                let mut encoder = device.create_command_encoder(&Default::default());

                canvas.render(
                    &sample_texture.create_view(&Default::default()),
                    &frame.output.texture.create_view(&Default::default()),
                    &mut encoder,
                    window_logical_size,
                );

                queue.submit(iter::once(encoder.finish()));
            }
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                swap_chain_desc.width = new_size.width;
                swap_chain_desc.height = new_size.height;
                surface.configure(&device, &swap_chain_desc);
                sample_texture = create_sample_texture(new_size, &*device);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event, .. } => {
                ui.handle_window_event(
                    &mut canvas,
                    &event,
                    window.scale_factor(),
                    window_logical_size,
                );

                // ui.handle_messages(|m: &Message| handle_msg(m));
            }
            _ => (),
        }
    });
}

fn create_sample_texture(window_size: PhysicalSize<u32>, device: &Device) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width: window_size.width,
            height: window_size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: dume_renderer::SAMPLE_COUNT,
        dimension: TextureDimension::D2,
        format: dume_renderer::TARGET_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT,
    })
}
