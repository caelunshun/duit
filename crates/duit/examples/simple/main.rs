use duit::{widgets::Text, InstanceHandle, Ui, WidgetHandle, WindowPositioner};
use duit_core::spec::Spec;
use dume_renderer::Rect;
use framework::Example;
use glam::Vec2;

#[path = "../framework.rs"]
mod framework;

struct Simple {
    _the_text: WidgetHandle<Text>,
}

impl InstanceHandle for Simple {
    fn name() -> &'static str {
        "Simple"
    }

    fn init(mut widget_handles: Vec<(String, duit::WidgetPodHandle)>) -> Self {
        Self {
            _the_text: WidgetHandle::new(widget_handles.remove(0).1),
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

fn main() {
    let mut ui = Ui::new();

    ui.add_spec(Spec::deserialize_from_str(include_str!("root.yml")).unwrap());

    let (_instance_handle, root) = ui.create_spec_instance::<Simple>();

    ui.create_window(root, Positioner, 1);

    Example::new().run(ui);
}
