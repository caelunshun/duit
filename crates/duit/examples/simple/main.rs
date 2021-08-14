use std::fs;

use duit::{Ui, WindowPositioner};
use duit_core::spec::Spec;
use dume_renderer::Rect;
use framework::Example;
use glam::Vec2;

use crate::generated::Simple;

#[path = "../framework.rs"]
mod framework;
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

    Example::new().run(ui, |message: &Message| match message {
        Message::ButtonPressed => println!("Clicked!"),
    });
}
