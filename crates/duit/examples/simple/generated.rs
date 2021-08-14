use duit::widgets::*;
use duit::{WidgetHandle, WidgetPodHandle};
pub struct Simple {
    pub the_button: WidgetHandle<Button>,
}
impl ::duit::InstanceHandle for Simple {
    fn name() -> &'static str {
        "Simple"
    }
    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
        let mut the_button = None;
        for (name, widget) in widget_handles {
            match name.as_str() {
                "the_button" => the_button = Some(widget),
                _ => {}
            }
        }
        Self {
            the_button: WidgetHandle::new(the_button.unwrap_or_else(|| {
                panic!(
                    "missing widget with ID '{}' (generated code not up to date)",
                    "the_button"
                )
            })),
        }
    }
}
