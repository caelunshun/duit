use duit_core::{spec::widgets::ScrollableSpec, Axis};
use dume::Rect;
use glam::{vec2, Vec2};
use winit::event::MouseButton;

use crate::{widget::Context, Color, Event, RectExt, Widget, WidgetData};

/// A widget that gives its child infinite size
/// along one axis, then displays a scrollbar
/// that moves the origin.
pub struct Scrollable {
    scroll_axis: Axis,
    cross_axis: Axis,

    child_size: Vec2,

    scroll_pos: f32,
    bar_grabbed: bool,
    bar_hovered: bool,
    grabbed_offset: f32,

    bar_width: f32,
}

impl Scrollable {
    pub fn from_spec(spec: &ScrollableSpec) -> Self {
        Self::new(spec.scroll_axis)
    }

    pub fn new(scroll_axis: Axis) -> Self {
        Self {
            scroll_axis,
            cross_axis: scroll_axis.cross(),
            child_size: Vec2::ZERO,

            scroll_pos: 0.,
            bar_grabbed: false,
            bar_hovered: false,
            bar_width: 0.,
            grabbed_offset: 0.,
        }
    }

    fn bar_rect(&self, self_size: Vec2) -> Rect {
        let bar_length = (self_size[self.scroll_axis as usize]
            / self.child_size[self.scroll_axis as usize])
            * self_size[self.scroll_axis as usize];
        let bar_pos = (self.scroll_pos / self.child_size[self.scroll_axis as usize])
            * self_size[self.scroll_axis as usize];
        match self.scroll_axis {
            Axis::Vertical => Rect::new(
                vec2(self_size.x - self.bar_width, bar_pos),
                vec2(self.bar_width, bar_length),
            ),
            Axis::Horizontal => Rect::new(
                vec2(bar_pos, self_size.y - self.bar_width),
                vec2(bar_length, self.bar_width),
            ),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    bar_width: f32,
    bar_border_radius: f32,

    bar_color: Color,
    hovered_bar_color: Color,
    grabbed_bar_color: Color,
}

impl Widget for Scrollable {
    type Style = Style;

    fn base_class(&self) -> &str {
        "scrollable"
    }

    fn layout(
        &mut self,
        style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        self.bar_width = style.bar_width;

        let mut child_max_size = max_size;
        child_max_size[self.scroll_axis as usize] = f32::INFINITY;

        let mut cross_size = 0.;
        data.for_each_child(|child| {
            child.layout(&mut cx, child_max_size);
            let mut origin = Vec2::ZERO;
            origin[self.scroll_axis as usize] = -self.scroll_pos;
            child.data_mut().set_origin(origin);

            cross_size = child.data().size()[self.cross_axis as usize];
            self.child_size = child.data().size();
        });

        let mut size = max_size;
        size[self.cross_axis as usize] = cross_size;
        if self.child_size[self.scroll_axis as usize] <= max_size[self.scroll_axis as usize] {
            size[self.scroll_axis as usize] = self.child_size[self.scroll_axis as usize];
        }
        data.set_size(size);
    }

    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        cx.canvas.scissor_rect(Rect::new(Vec2::ZERO, data.size()));
        data.paint_children(&mut cx);
        cx.canvas.clear_scissor();

        if self.child_size[self.scroll_axis as usize] > data.size()[self.scroll_axis as usize] {
            let bar = self.bar_rect(data.size());
            cx.canvas
                .begin_path()
                .rounded_rect(bar.pos, bar.size, style.bar_border_radius);

            let bar_color = if self.bar_grabbed {
                style.grabbed_bar_color
            } else if self.bar_hovered {
                style.hovered_bar_color
            } else {
                style.bar_color
            };
            cx.canvas.solid_color(bar_color.into()).fill();
        }
    }

    fn handle_event(&mut self, data: &mut WidgetData, mut cx: Context, event: &Event) {
        let bar = self.bar_rect(data.size()).expanded(5.);
        match event {
            Event::MousePress {
                pos,
                button: MouseButton::Left,
            } => {
                self.bar_grabbed = bar.contains(*pos);
                self.grabbed_offset =
                    pos[self.scroll_axis as usize] - bar.pos[self.scroll_axis as usize];
            }
            Event::MouseRelease {
                button: MouseButton::Left,
                ..
            } => self.bar_grabbed = false,
            Event::MouseMove { pos } => {
                if self.bar_grabbed {
                    self.scroll_pos = (pos[self.scroll_axis as usize] - self.grabbed_offset)
                        * (self.child_size[self.scroll_axis as usize]
                            / data.size()[self.scroll_axis as usize]);
                }
                self.bar_hovered = bar.contains(*pos);
            }
            Event::Scroll { offset, mouse_pos } if data.bounds().contains(*mouse_pos) => {
                self.scroll_pos -= offset[self.scroll_axis as usize];
            }
            _ => {}
        }
        self.scroll_pos = self.scroll_pos.clamp(
            0.,
            (self.child_size[self.scroll_axis as usize] - data.size()[self.scroll_axis as usize])
                .max(0.),
        );
        data.pass_event_to_children(&mut cx, event);
    }
}
