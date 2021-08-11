use crate::widget::WidgetPodHandle;

pub trait InstanceHandle {
    fn name() -> &'static str;

    fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self;
}
