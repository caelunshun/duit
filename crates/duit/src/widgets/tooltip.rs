use glam::{vec2, Vec2};

use crate::{widget::Context, Event, Widget, WidgetData};

const CHILD_INDEX: usize = 0;
const TOOLTIP_INDEX: usize = 1;

pub struct Tooltip {
    showing_tooltip: bool,
}

impl Tooltip {
    pub fn new() -> Self {
        Self {
            showing_tooltip: false,
        }
    }
}

impl Widget for Tooltip {
    type Style = ();

    fn base_class(&self) -> &str {
        "tooltip"
    }

    fn layout(
        &mut self,
        _style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        let mut child = data.child(CHILD_INDEX);
        child.layout(&mut cx, max_size);

        // Give the tooltip infinite size
        let mut tooltip = data.child(TOOLTIP_INDEX);
        tooltip.layout(&mut cx, Vec2::splat(f32::INFINITY));
        let pos = vec2(
            -tooltip.data().size().x - 10.,
            -tooltip.data().size().y / 2. + child.data().size().y / 2.,
        );
        tooltip.data_mut().set_origin(pos);

        let size = child.data().size();
        drop(child);
        drop(tooltip);
        data.set_size(size);
    }

    fn paint(&mut self, _style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        data.child(CHILD_INDEX).paint(&mut cx);
    }

    fn paint_overlay(&mut self, _style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        if self.showing_tooltip {
            data.child(TOOLTIP_INDEX).paint(&mut cx);
        }
    }

    fn handle_event(&mut self, data: &mut WidgetData, mut cx: Context, event: &Event) {
        if let Event::MouseMove { pos } = event {
            self.showing_tooltip = data.bounds().contains(*pos);
        }

        data.pass_event_to_children(&mut cx, event);
    }
}
