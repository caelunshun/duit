use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    rc::Rc,
};

use dume_renderer::{Canvas, Rect};
use glam::Vec2;
use serde::de::DeserializeOwned;

use crate::{style::StyleEngine, Event};

pub type WidgetPodHandle = Rc<RefCell<WidgetPod>>;

// Special style classes that are automatically added
// when a widget is hovered or pressed.
pub const CLASS_HOVERED: &str = "hovered";
pub const CLASS_PRESSED: &str = "pressed";

pub struct WidgetHandle<T> {
    pod: WidgetPodHandle,
    _marker: PhantomData<T>,
}

impl<T> WidgetHandle<T>
where
    T: Widget,
{
    #[doc(hidden)]
    pub fn new(pod: WidgetPodHandle) -> Self {
        Self {
            pod,
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> Ref<T> {
        Ref::map(self.pod.borrow(), |pod| {
            (*pod.widget).as_any().downcast_ref().unwrap()
        })
    }

    pub fn get_mut(&self) -> RefMut<T> {
        RefMut::map(self.pod.borrow_mut(), |pod| {
            (*pod.widget).as_any_mut().downcast_mut().unwrap()
        })
    }
}

/// Contains a `dyn Widget` and the `WidgetData` associated with the widget.
pub struct WidgetPod {
    pub(crate) widget: Box<dyn DynWidget>,
    data: WidgetData,
}

impl WidgetPod {
    pub(crate) fn new(widget: Box<dyn DynWidget>) -> Self {
        Self {
            widget,
            data: WidgetData::default(),
        }
    }

    pub fn mount(&mut self) {
        self.widget.mount(&mut self.data);
    }

    pub fn layout(&mut self, parent_cx: &mut Context, max_size: Vec2) {
        let cx = Context {
            canvas: parent_cx.canvas,
            style_engine: parent_cx.style_engine,
        };
        self.widget.layout(&mut self.data, cx, max_size);
    }

    pub fn paint(&mut self, parent_cx: &mut Context) {
        parent_cx.canvas.translate(self.data.origin());

        let cx = Context {
            canvas: parent_cx.canvas,
            style_engine: parent_cx.style_engine,
        };
        self.widget.paint(&mut self.data, cx);

        parent_cx.canvas.translate(-self.data().origin());
    }

    pub fn handle_event(&mut self, parent_cx: &mut Context, event: &Event) {
        let event = event.translated(-self.data().origin());

        self.update_widget_state(&event);

        let cx = Context {
            canvas: parent_cx.canvas,
            style_engine: parent_cx.style_engine,
        };
        self.widget.handle_event(&mut self.data, cx, &event);

        if self.data.are_classes_dirty() {
            let cx = Context {
                canvas: parent_cx.canvas,
                style_engine: parent_cx.style_engine,
            };
            self.widget.style_changed(&mut self.data, cx);
            self.data.mark_classes_clean();
        }
    }

    fn update_widget_state(&mut self, event: &Event) {
        let rect = Rect::new(Vec2::ZERO, self.data.size());
        match event {
            Event::MousePress { pos, .. } => {
                if rect.contains(*pos) {
                    self.data.state.pressed = true;
                    self.data.add_class(CLASS_PRESSED);
                }
            }
            Event::MouseRelease { .. } => {
                if self.data.state.pressed {
                    self.data.state.pressed = false;
                    self.data.remove_class(CLASS_PRESSED);
                }
            }
            Event::MouseMove { pos } => {
                if rect.contains(*pos) && !self.data.state.hovered {
                    self.data.state.hovered = true;
                    self.data.add_class(CLASS_HOVERED);
                } else if !rect.contains(*pos) && self.data.state.hovered {
                    self.data.state.hovered = false;
                    self.data.remove_class(CLASS_HOVERED);
                }
            }
            _ => {}
        }
    }

    pub fn data(&self) -> &WidgetData {
        &self.data
    }

    pub(crate) fn data_mut(&mut self) -> &mut WidgetData {
        &mut self.data
    }
}

/// Data associated with every widget.
///
/// Includes:
/// * a list of child widget handles
/// * the currently computed layout
/// * whether the widget is currently hovered or pressed
/// * the widget's style classes
pub struct WidgetData {
    /// Widget children
    children: Vec<WidgetPodHandle>,
    /// The origin of the widget's coordinate space, relative to the parent's coordinate space
    origin: Vec2,
    /// The size of the widget.
    size: Vec2,

    /// Flex parameter used by the `Flex` widget.
    flex: Option<f32>,

    /// Style classes.
    classes: Vec<String>,
    /// Whether classes have changed since the class call to `Widget::style_changed`
    classes_dirty: bool,

    state: WidgetState,
}

impl Default for WidgetData {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            origin: Vec2::ZERO,
            flex: None,
            size: Vec2::ZERO,
            classes: Vec::new(),
            classes_dirty: false,
            state: WidgetState::default(),
        }
    }
}

#[derive(Debug)]
pub enum LayoutStrategy {
    Shrink { padding: f32 },
    Fill,
}

impl WidgetData {
    /// Invokes a closure for each child of this widget.
    ///
    /// Note that this method is not recursive, i.e. it processes
    /// only the direct children of this node.
    pub fn for_each_child(&self, mut callback: impl FnMut(&mut WidgetPod)) {
        for handle in &self.children {
            let mut pod = handle.borrow_mut();
            callback(&mut *pod);
        }
    }

    pub fn num_children(&self) -> usize {
        self.children.len()
    }

    pub fn paint_children(&mut self, cx: &mut Context) {
        self.for_each_child(|child| {
            child.paint(cx);
        });
    }

    /// A convenience method to lay out a single child.
    ///
    /// The parameter `strategy` determines how to perform layout:
    /// * `LayoutStrategy::Shrink` - shrinks the size of this widget to the size of its child
    /// (optionally with some padding)
    /// * `LayoutStrategy::Fill` - fill all available space.
    pub fn lay_out_child(&mut self, strategy: LayoutStrategy, cx: &mut Context, max_size: Vec2) {
        let mut child = self.children[0].borrow_mut();
        match strategy {
            LayoutStrategy::Shrink { padding } => {
                child.layout(cx, max_size - (padding * 2.));
                child.data_mut().set_origin(Vec2::splat(padding));
                self.size = child.data().size() + (padding * 2.);
            }
            LayoutStrategy::Fill => {
                child.layout(cx, max_size);
                child.data_mut().set_origin(Vec2::ZERO);
                self.size = max_size;
            }
        };
    }

    pub fn pass_event_to_children(&mut self, cx: &mut Context, event: &Event) {
        self.for_each_child(|child| child.handle_event(cx, event));
    }

    pub fn origin(&self) -> Vec2 {
        self.origin
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn state(&self) -> WidgetState {
        self.state
    }

    pub fn flex(&self) -> Option<f32> {
        self.flex
    }

    pub fn set_origin(&mut self, origin: Vec2) {
        self.origin = origin;
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn add_child(&mut self, child: WidgetPodHandle) {
        self.children.push(child);
    }

    pub fn insert_child(&mut self, child: WidgetPodHandle, index: usize) {
        self.children.insert(index, child);
    }

    pub fn remove_child(&mut self, index: usize) {
        self.children.remove(index);
    }

    pub(crate) fn set_flex(&mut self, flex: Option<f32>) {
        self.flex = flex;
    }

    pub fn classes(&self) -> &[String] {
        &self.classes
    }

    pub fn remove_class(&mut self, class: &str) {
        if let Some(index) = self.classes.iter().position(|c| c == class) {
            self.classes.remove(index);
            self.classes_dirty = true;
        }
    }

    pub fn add_class(&mut self, class: &str) {
        self.classes.push(class.to_owned());
        self.classes_dirty = true;
    }

    pub fn are_classes_dirty(&self) -> bool {
        self.classes_dirty
    }

    pub fn mark_classes_clean(&mut self) {
        self.classes_dirty = false;
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct WidgetState {
    pub hovered: bool,
    pub pressed: bool,
}

#[non_exhaustive]
pub struct Context<'a> {
    pub canvas: &'a mut Canvas,
    pub(crate) style_engine: &'a mut StyleEngine,
}

pub trait Widget: AsAny + 'static {
    type Style: DeserializeOwned + 'static;

    /// Gets the default style class for this widget.
    fn base_class(&self) -> &str;

    /// Called when the widget is first added to the tree.
    ///
    /// The default implementation does nothing.
    #[allow(unused_variables)]
    fn mount(&mut self, data: &mut WidgetData) {}

    /// Handles an input event.
    ///
    /// The default implementation passes events onto the widget's children.
    #[allow(unused_variables)]
    fn handle_event(&mut self, data: &mut WidgetData, mut cx: Context, event: &Event) {
        data.pass_event_to_children(&mut cx, event);
    }

    /// Called when the widget's style has changed.
    ///
    /// The default implementation does nothing.
    #[allow(unused_variables)]
    fn style_changed(&mut self, style: &Self::Style, data: &mut WidgetData, cx: Context) {}

    /// Computes layout for this widget and its children.
    ///
    /// This method can call `layout` on each of its children
    /// to retrieve their sizes. It should call `set_origin` for
    /// each child to set their positions relative to this widget.
    ///
    /// This method should call `set_size` with the computed size of this widget.
    fn layout(&mut self, style: &Self::Style, data: &mut WidgetData, cx: Context, max_size: Vec2);

    /// Paints this widget and potentially its children.
    ///
    /// This method can call `paint` on each of its children.
    fn paint(&mut self, style: &Self::Style, data: &mut WidgetData, cx: Context);
}

/// A `Widget` with type parameters erased.
pub trait DynWidget: AsAny + 'static {
    fn base_class(&self) -> &str;

    fn mount(&mut self, data: &mut WidgetData);

    fn handle_event(&mut self, data: &mut WidgetData, cx: Context, event: &Event);

    fn style_changed(&mut self, data: &mut WidgetData, cx: Context);

    fn layout(&mut self, data: &mut WidgetData, cx: Context, max_size: Vec2);

    fn paint(&mut self, data: &mut WidgetData, cx: Context);
}

impl<T> DynWidget for T
where
    T: Widget,
{
    fn base_class(&self) -> &str {
        <T as Widget>::base_class(self)
    }

    fn mount(&mut self, data: &mut WidgetData) {
        <T as Widget>::mount(self, data)
    }

    fn handle_event(&mut self, data: &mut WidgetData, cx: Context, event: &Event) {
        <T as Widget>::handle_event(self, data, cx, event);
    }

    fn style_changed(&mut self, data: &mut WidgetData, cx: Context) {
        let style = cx
            .style_engine
            .get_style(data.classes())
            .expect("failed to compute widget style");
        <T as Widget>::style_changed(self, &*style, data, cx);
    }

    fn layout(&mut self, data: &mut WidgetData, cx: Context, max_size: Vec2) {
        let style = cx
            .style_engine
            .get_style(data.classes())
            .expect("failed to compute widget style");
        <T as Widget>::layout(self, &*style, data, cx, max_size)
    }

    fn paint(&mut self, data: &mut WidgetData, cx: Context) {
        let style = cx
            .style_engine
            .get_style(data.classes())
            .expect("failed to compute widget style");
        <T as Widget>::paint(self, &*style, data, cx)
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AsAny for T
where
    T: Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
