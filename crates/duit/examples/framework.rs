use std::{fs, iter, sync::Arc, time::Instant};

use duit::Ui;
use dume_renderer::{Canvas, SpriteData, SpriteDescriptor};
use glam::Vec2;
use wgpu::{
    BackendBit, Device, Extent3d, Features, Instance, Limits, PresentMode, Surface, SwapChain,
    SwapChainDescriptor, Texture, TextureDescriptor, TextureDimension, TextureUsage,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Example {
    event_loop: EventLoop<()>,
    window: Window,
    surface: Surface,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    canvas: Canvas,
    swap_chain: SwapChain,
    swap_chain_desc: SwapChainDescriptor,
    sample_texture: Texture,
}

impl Example {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .build(&event_loop)
            .expect("failed to create window");

        let instance = Instance::new(BackendBit::PRIMARY);

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

        let swap_chain_desc = SwapChainDescriptor {
            usage: TextureUsage::RENDER_ATTACHMENT,
            format: dume_renderer::TARGET_FORMAT,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        let sample_texture = create_sample_texture(window.inner_size(), &*device);

        let mut canvas = Canvas::new(Arc::clone(&device), Arc::clone(&queue));

        canvas.load_font(fs::read("assets/CormorantGaramond-Regular.ttf").unwrap());
        canvas.create_sprite(SpriteDescriptor {
            name: "ozymandias",
            data: SpriteData::Encoded(&fs::read("assets/ozymandias.jpeg").unwrap()),
        });

        canvas.set_scale_factor(window.scale_factor());

        Self {
            event_loop,
            window,
            surface,
            device,
            queue,
            canvas,
            swap_chain,
            swap_chain_desc,
            sample_texture,
        }
    }

    pub fn run<Message, Handler>(
        self,
        mut ui: Ui,
        mut handle_message: Handler,
        mut tick: impl FnMut(&mut Ui, f32) + 'static,
    ) where
        Handler: FnMut(&Message) + 'static,
        Message: 'static,
    {
        ui.add_stylesheet(include_bytes!("../../../themes/default.yml"))
            .unwrap();
        let Self {
            event_loop,
            window,
            surface,
            device,
            queue,
            mut canvas,
            mut swap_chain,
            mut swap_chain_desc,
            mut sample_texture,
        } = self;
        let mut previous_time = Instant::now();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            let window_logical_size = Vec2::new(
                window.inner_size().to_logical(window.scale_factor()).width,
                window.inner_size().to_logical(window.scale_factor()).height,
            );
            match event {
                Event::RedrawRequested(_) => {
                    let dt = previous_time.elapsed();
                    tick(&mut ui, dt.as_secs_f32());
                    previous_time = Instant::now();

                    ui.render(&mut canvas, window_logical_size);
                    let frame = swap_chain
                        .get_current_frame()
                        .expect("failed to get next frame");

                    let mut encoder = device.create_command_encoder(&Default::default());

                    canvas.render(
                        &sample_texture.create_view(&Default::default()),
                        &frame.output.view,
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
                    swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
                    sample_texture = create_sample_texture(new_size, &*device);
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::WindowEvent { event, .. } => {
                    ui.handle_window_event(&mut canvas, &event, window.scale_factor());

                    ui.handle_messages(|m| handle_message(m));
                }
                _ => (),
            }
        });
    }
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
        usage: TextureUsage::RENDER_ATTACHMENT,
    })
}

// This allows treating the framework as a standalone example,
// thus avoiding listing the example names in `Cargo.toml`.
#[allow(dead_code)]
fn main() {}
