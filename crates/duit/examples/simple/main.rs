use duit::{widgets::Button, InstanceHandle, Ui, WidgetHandle, WindowPositioner};
use duit_core::spec::Spec;
use dume_renderer::Rect;
use framework::Example;
use glam::Vec2;

#[path = "../framework.rs"]
mod framework;

struct Simple {
    the_button: WidgetHandle<Button>,
}

impl InstanceHandle for Simple {
    fn name() -> &'static str {
        "Simple"
    }

    fn init(mut widget_handles: Vec<(String, duit::WidgetPodHandle)>) -> Self {
        Self {
            the_button: WidgetHandle::new(widget_handles.remove(0).1),
        }
    }
}

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

    ui.add_spec(Spec::deserialize_from_str(include_str!("root.yml")).unwrap());

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
