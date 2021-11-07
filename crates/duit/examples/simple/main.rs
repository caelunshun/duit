use std::{fs, time::Instant};

use duit::{widget, widgets::Text, Ui, WindowPositioner};
use duit_core::spec::Spec;
use dume::Rect;
use glam::Vec2;
use rand::Rng;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

use crate::generated::Simple;

mod generated;

struct SelectedItem(u32);

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

    for i in 0..10 {
        let mut pick_list = instance_handle.the_pick_list.get_mut();
        pick_list.add_option(widget(Text::new(duit::text!("#{}", i))), move || {
            SelectedItem(i)
        });
    }

    // Add table rows
    {
        let mut table = instance_handle.the_table.get_mut();

        for i in 0..100 {
            let name = Text::new(dume::text!("Player #{}", i));
            let value = Text::new(dume::text!("{}", rand::thread_rng().gen_range(1u32..100)));

            table.add_row([("name", widget(name)), ("value", widget(value))]);
        }
    }

    ui.create_window(root, Positioner, 1);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Duit Simple Example")
        .with_inner_size(LogicalSize::new(1920, 1080))
        .build(&event_loop)
        .unwrap();

    let start = Instant::now();

    duit_platform::run(
        event_loop,
        window,
        ui,
        |cv| {
            let mut texture_set = cv.context().create_texture_set_builder();
            texture_set
                .add_texture(
                    include_bytes!("../../../../assets/ozymandias.jpeg"),
                    "ozymandias",
                )
                .unwrap();
            cv.context()
                .add_texture_set(texture_set.build(1024, 4096).unwrap());
            cv.context()
                .add_font(
                    include_bytes!("../../../../assets/CormorantGaramond-Regular.ttf").to_vec(),
                )
                .unwrap();
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
