use std::{any::Any, cell::Cell, rc::Rc};

use duit_core::{
    spec::widgets::{BaseSpec, FlexSpec, PickListSpec},
    Align, Axis,
};
use dume::{font::Query, Rect, Text, TextBlob, TextOptions, TextSection, TextStyle};
use glam::{vec2, Vec2};
use winit::event::MouseButton;

use crate::{
    widget, widget::Context, widget::HitTestResult, Color, Event, Widget, WidgetData, WidgetHandle,
    WidgetPodHandle,
};

use super::{Flex, Scrollable};

const CHILD_INDEX_PLACEHOLDER: usize = 0;
const CHILD_INDEX_OVERLAY: usize = 1;

pub struct PickList {
    width: Option<f32>,
    max_height: Option<f32>,

    options: WidgetHandle<Flex>,
    queued_child: Option<WidgetPodHandle>,
    child: WidgetPodHandle,

    arrow_down: Option<TextBlob>,

    opened: bool,

    selection_updated: Rc<Cell<bool>>,
}

impl PickList {
    pub fn from_spec(spec: &PickListSpec) -> Self {
        Self::new(spec.width, spec.max_height)
    }

    pub fn new(width: Option<f32>, max_height: Option<f32>) -> Self {
        let column = Flex::from_spec(
            &FlexSpec {
                base: BaseSpec::default(),
                align_h: Align::Start,
                align_v: Align::Start,
                children: Vec::new(),
                spacing: 0.,
            },
            Axis::Vertical,
        );

        let column = widget(column);

        let child = widget(Scrollable::new(Axis::Vertical));
        child.borrow_mut().data_mut().add_child(Rc::clone(&column));

        let selection_updated = Rc::new(Cell::new(false));

        Self {
            width,
            max_height,

            options: WidgetHandle::new(column),
            child: Rc::clone(&child),
            queued_child: Some(child),
            arrow_down: None,

            opened: false,
            selection_updated,
        }
    }

    pub fn add_option<Message: 'static>(
        &mut self,
        option: WidgetPodHandle,
        mut on_select: impl FnMut() -> Message + 'static,
    ) -> &mut Self {
        let container = widget(PickListOption {
            selection_updated: Rc::clone(&self.selection_updated),
            on_select: Box::new(move || Box::new(on_select())),
        });
        container.borrow_mut().data_mut().add_child(option);

        self.options.get_mut().add_child(container);

        self
    }
}

const ARROW_DOWN: &str = "▼";

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    border_color: Color,
    border_radius: f32,
    border_width: f32,
    background_color: Color,

    arrow_font_family: String,
    arrow_size: f32,
    arrow_color: Color,
}

impl Widget for PickList {
    type Style = Style;

    fn base_class(&self) -> &str {
        "pick_list"
    }

    fn layout(
        &mut self,
        _style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        let width = match self.width {
            Some(x) => x,
            None => max_size.x,
        };

        let padding = 10.;
        let height = {
            let mut placeholder = data.child(CHILD_INDEX_PLACEHOLDER);
            placeholder.layout(&mut cx, max_size - Vec2::splat(padding * 2.));
            placeholder.data_mut().set_origin(Vec2::splat(padding));
            placeholder.data().size().y
        };

        data.set_size(vec2(width, height) + Vec2::splat(padding * 2.));

        if let Some(child) = self.queued_child.take() {
            data.add_child(child);
        }

        let mut overlay = data.child(CHILD_INDEX_OVERLAY);
        let mut overlay_constraints = vec2(width, f32::INFINITY);
        if let Some(max_height) = self.max_height {
            overlay_constraints.y = max_height;
        }
        overlay.layout(&mut cx, overlay_constraints);

        overlay.data_mut().set_origin(vec2(0., data.size().y));
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        cx.canvas
            .begin_path()
            .rounded_rect(Vec2::ZERO, data.size(), style.border_radius)
            .solid_color(style.background_color)
            .fill();
        cx.canvas
            .stroke_width(style.border_width)
            .solid_color(style.border_color)
            .stroke();

        let arrow_down = self.arrow_down.get_or_insert_with(|| {
            let text = Text::from_sections(vec![TextSection::Text {
                text: ARROW_DOWN.into(),
                style: TextStyle {
                    font: Query {
                        family: Some(style.arrow_font_family.clone().into()),
                        ..Default::default()
                    },
                    size: Some(style.arrow_size),
                    color: Some(style.arrow_color.into()),
   
                },
            }]);
            cx.canvas.context().create_text_blob(
                text,
                TextOptions {
                    baseline: dume::Baseline::Middle,
                    align_h: dume::Align::Start,
                    align_v: dume::Align::Start,
                    wrap_lines: false,
                },
            )
        });
        cx.canvas.draw_text(
            arrow_down,
            data.size() - vec2(style.arrow_size, data.size().y / 2.),
            1.,
        );

        data.child(CHILD_INDEX_PLACEHOLDER).paint(&mut cx);
    }

    fn paint_overlay(&mut self, _style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        if self.opened {
            data.child(CHILD_INDEX_OVERLAY).paint(&mut cx);
        }
    }

    fn handle_event(&mut self, data: &mut WidgetData, mut cx: Context, event: &Event) {
        if self.opened {
            data.pass_event_to_children(&mut cx, event);
        }

        if self.selection_updated.get() {
            self.selection_updated.set(false);
            self.opened = false;
        }

        if let Event::MousePress {
            pos,
            button: MouseButton::Left,
            ..
        } = event
        {
            let overlay = self.child.borrow();

            if data.bounds().contains(*pos) {
                self.opened = !self.opened;
            } else if !Rect::new(overlay.data().origin(), overlay.data().size()).contains(*pos) {
                self.opened = false;
            }
        }
    }
}

struct PickListOption {
    selection_updated: Rc<Cell<bool>>,
    on_select: Box<dyn FnMut() -> Box<dyn Any>>,
}

#[derive(Debug, serde::Deserialize)]
struct OptionStyle {
    border_color: Color,
    border_width: f32,
    background_color: Color,
    padding: f32,
}

impl Widget for PickListOption {
    type Style = OptionStyle;

    fn base_class(&self) -> &str {
        "pick_list_option"
    }

    fn layout(
        &mut self,
        style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        let mut child_size = Vec2::default();
        data.for_each_child(|child| {
            child.layout(&mut cx, max_size - Vec2::splat(style.padding * 2.));
            child_size = child.data().size();
            child.data_mut().set_origin(Vec2::splat(style.padding));
        });
        data.set_size(vec2(max_size.x, child_size.y + style.padding * 2.));
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        cx.canvas
            .begin_path()
            .rect(Vec2::ZERO, data.size())
            .solid_color(style.background_color)
            .fill();
        cx.canvas
            .solid_color(style.border_color)
            .stroke_width(style.border_width)
            .stroke();

        data.paint_children(&mut cx);
    }

    fn handle_event(&mut self, data: &mut WidgetData, mut cx: Context, event: &Event) {
        if let Event::MousePress {
            button: MouseButton::Left,
            pos,
            ..
        } = event
        {
            if data.bounds().contains(*pos) {
                self.selection_updated.set(true);

                cx.send_message((self.on_select)());
            }
        }
        data.pass_event_to_children(&mut cx, event);
    }

    fn hit_test(&self, data: &WidgetData, pos: Vec2) -> HitTestResult {
        if data.bounds().contains(pos) {
            HitTestResult::Hit
        } else {
            HitTestResult::Missed
        }
    }
}
