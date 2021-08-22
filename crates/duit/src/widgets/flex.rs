use std::mem;

use duit_core::{spec::widgets::FlexSpec, Align};
use glam::Vec2;

use crate::{widget::Context, Widget, WidgetData, WidgetPodHandle};

/// Indicates an axis used for layout.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    Horizontal = 0,
    Vertical = 1,
}

enum ChildUpdate {
    Add(WidgetPodHandle),
    Insert(WidgetPodHandle, usize),
    Remove(usize),
    Clear,
}

/// A widget that lays out its children according to a flexbox model.
///
/// This widget is the basis of most complex layouts.
pub struct Flex {
    main_axis: Axis,
    main_align: Align,
    cross_align: Align,
    spacing: f32,

    queued_child_updates: Vec<ChildUpdate>,
}

impl Flex {
    pub fn from_spec(spec: &FlexSpec, main_axis: Axis) -> Self {
        let (mut main_align, mut cross_align) = (spec.align_h, spec.align_v);
        if main_axis == Axis::Vertical {
            mem::swap(&mut main_align, &mut cross_align);
        }

        Self {
            main_axis,
            main_align,
            cross_align,
            spacing: spec.spacing,
            queued_child_updates: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: WidgetPodHandle) -> &mut Self {
        self.queued_child_updates.push(ChildUpdate::Add(child));
        self
    }

    pub fn insert_child(&mut self, child: WidgetPodHandle, index: usize) -> &mut Self {
        self.queued_child_updates
            .push(ChildUpdate::Insert(child, index));
        self
    }

    pub fn remove_child(&mut self, index: usize) -> &mut Self {
        self.queued_child_updates.push(ChildUpdate::Remove(index));
        self
    }

    pub fn clear_children(&mut self) -> &mut Self {
        self.queued_child_updates.push(ChildUpdate::Clear);
        self
    }

    fn process_queued_child_updates(&mut self, data: &mut WidgetData) {
        for update in self.queued_child_updates.drain(..) {
            match update {
                ChildUpdate::Add(widget) => data.add_child(widget),
                ChildUpdate::Insert(widget, index) => data.insert_child(widget, index),
                ChildUpdate::Remove(index) => data.remove_child(index),
                ChildUpdate::Clear => data.clear_children(),
            }
        }
    }

    fn cross_axis(&self) -> Axis {
        match self.main_axis {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }
}

impl Widget for Flex {
    type Style = ();

    fn base_class(&self) -> &str {
        "flex"
    }

    fn layout(
        &mut self,
        _style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        self.process_queued_child_updates(data);

        let main_axis = self.main_axis as usize;
        let cross_axis = self.cross_axis() as usize;

        // Flexbox-inspired layout algorithm.

        // Determine the sum of all flex factors.
        // Also determine the total size along the main axis of all non-flex children.
        let mut flex_factor_sum = 0.0f32;
        let mut non_flex_size = 0.0f32;

        data.for_each_child(|child| match child.data().flex() {
            Some(flex_factor) => flex_factor_sum += flex_factor,
            None => {
                child.layout(&mut cx, max_size);
                non_flex_size += child.data().size()[main_axis];
            }
        });

        // Available space for flex widgets is the total available space
        // minus what is consumed by non-flex widgets
        // minus spacing.
        let total_spacing = data.num_children() as f32 * self.spacing;
        let flex_space = max_size[main_axis] - non_flex_size - total_spacing;

        // Now that we can compute each widget's size, we can set their origins.
        let mut cursor = 0.0;
        let mut largest_cross_size = 0.0;
        data.for_each_child(|child| {
            let mut origin = Vec2::ZERO;
            origin[main_axis] = cursor;
            child.data_mut().set_origin(origin);

            match child.data().flex() {
                Some(flex_factor) => {
                    let widget_main_size = flex_space * (flex_factor / flex_factor_sum);
                    let mut widget_constraints = Vec2::ZERO;
                    widget_constraints[main_axis] = widget_main_size;
                    widget_constraints[cross_axis] = max_size[cross_axis];

                    child.layout(&mut cx, widget_constraints);

                    cursor += widget_main_size;
                }
                None => {
                    cursor += child.data().size()[main_axis];

                    let cross_size = child.data().size()[cross_axis];
                    if cross_size > largest_cross_size {
                        largest_cross_size = cross_size;
                    }
                }
            }

            cursor += self.spacing;
        });

        if cursor > 0. {
            // Remove spacing after the last widget,
            // since it isn't followed by another widget.
            cursor -= self.spacing;
        }

        let mut offset = Vec2::splat(f32::INFINITY);

        // Apply alignment - along both the main and cross axes.
        data.for_each_child(|child| {
            let mut origin = child.data().origin();
            match self.main_align {
                Align::Start => {}
                Align::Center => {
                    origin[main_axis] += max_size[main_axis] / 2. - cursor / 2.;
                }
                Align::End => {
                    origin[main_axis] += max_size[main_axis] - cursor;
                }
            }
            match self.cross_align {
                Align::Start => {}
                Align::Center => {
                    origin[cross_axis] +=
                        max_size[cross_axis] / 2. - child.data().size()[cross_axis] / 2.;
                }
                Align::End => {
                    origin[cross_axis] += max_size[cross_axis] - child.data().size()[cross_axis];
                }
            }

            child.data_mut().set_origin(origin);

            if origin.x < offset.x { offset.x = origin.x; }
            if origin.y < offset.y { offset.y = origin.y; }
        });

        if offset.x.is_infinite() {
            offset.x = 0.0;
        }
        if offset.y.is_infinite() {
            offset.y = 0.0;
        }
        data.set_offset(offset);

        let mut size = Vec2::ZERO;
        size[main_axis] = cursor;
        size[cross_axis] = largest_cross_size;
        data.set_size(size);
    }

    fn paint(&mut self, _style: &Self::Style, data: &mut WidgetData, mut cx: Context) {
        data.paint_children(&mut cx);
    }
}
