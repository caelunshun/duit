use std::{fs, time::Instant};

use duit::{Ui, WindowPositioner};
use duit_core::spec::Spec;
use dume_renderer::{Rect, SpriteData, SpriteDescriptor};
use glam::Vec2;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::generated::Simple;

mod generated;

struct Positioner;
impl WindowPositioner for Positioner {
    fn compute_position(&self, available_space: Vec2) -> Rect {
        Rect {
            size: available_space,
            pos: Vec2::ZERO,
        }
    }
}

enum Message {
    ButtonPressed,
}

fn main() {
    let mut ui = Ui::new();

    ui.add_spec(
        Spec::deserialize_from_str(
            &fs::read_to_string("crates/duit/examples/simple/root.yml").unwrap(),
        )
        .unwrap(),
    );

    let (instance_handle, root) = ui.create_spec_instance::<Simple>();

    instance_handle
        .the_button
        .get_mut()
        .on_click(|| Message::ButtonPressed);

    ui.create_window(root, Positioner, 1);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Duit Simple Example")
        .build(&event_loop)
        .unwrap();

    let start = Instant::now();

    duit_platform::run(
        event_loop,
        window,
        ui,
        |cv| {
            cv.create_sprite(SpriteDescriptor {
                name: "ozymandias",
                data: SpriteData::Encoded(include_bytes!("../../../../assets/ozymandias.jpeg")),
            });
            cv.load_font(
                include_bytes!("../../../../assets/CormorantGaramond-Regular.ttf").to_vec(),
            )
        },
        move |_| {
            let time = start.elapsed().as_secs_f32();
            instance_handle
                .progress_bar
                .get_mut()
                .set_progress((time.sin() + 1.0) / 2.0);
        },
    );
}
