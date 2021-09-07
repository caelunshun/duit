use std::{any::Any, cell::RefCell, collections::VecDeque, rc::Rc};

use ahash::AHashMap;
use duit_core::{
    spec::{self, Spec},
    Axis,
};
use dume_renderer::Canvas;
use glam::Vec2;
use slotmap::SlotMap;
use winit::event::WindowEvent;

use crate::{
    event::EventTracker,
    spec::InstanceHandle,
    style::{StyleEngine, StyleError},
    widget::{DynWidget, WidgetPod, WidgetPodHandle},
    widgets,
    window::{Window, WindowPositioner},
    Event, Widget,
};

slotmap::new_key_type! {
    pub struct WindowId;
}

/// Contains the entire UI state, including all windows and their widget trees.
#[derive(Default)]
pub struct Ui {
    windows: SlotMap<WindowId, Window>,
    sorted_windows: Vec<WindowId>,
    specs: AHashMap<String, Spec>,
    style_engine: StyleEngine,
    event_tracker: EventTracker,
    messages: VecDeque<Box<dyn Any>>,

    custom_widget_builders: AHashMap<String, Box<dyn Fn(&serde_yaml::Value) -> Box<dyn DynWidget>>>,
}

impl Ui {
    pub fn new() -> Self {
        let mut u = Self::default();
        u.add_stylesheet(include_bytes!("../../../themes/default.yml"))
            .expect("invalid default theme");
        u
    }

    pub fn add_spec(&mut self, spec: Spec) -> &mut Self {
        self.specs.insert(spec.name.clone(), spec);
        self
    }

    pub fn add_stylesheet(&mut self, stylesheet_bytes: &[u8]) -> Result<&mut Self, StyleError> {
        self.style_engine.append_sheet(stylesheet_bytes)?;
        Ok(self)
    }

    pub fn add_custom_widget<W: Widget>(
        &mut self,
        name: &str,
        builder: impl Fn(&serde_yaml::Value) -> W + 'static,
    ) -> &mut Self {
        self.custom_widget_builders.insert(
            name.to_owned(),
            Box::new(move |params| Box::new(builder(params))),
        );
        self
    }

    pub fn create_spec_instance<S: InstanceHandle>(&mut self) -> (S, WidgetPodHandle) {
        let spec = self
            .specs
            .get(S::name())
            .unwrap_or_else(|| panic!("spec '{}' is not registered with the UI", S::name()));

        let mut widgets_with_ids = Vec::new();

        // Instantiate the widget tree.
        let root = instantiate_widget(self, &spec.child, &mut widgets_with_ids);

        let spec_handle = S::init(widgets_with_ids);
        (spec_handle, root)
    }

    pub fn create_window(
        &mut self,
        root: WidgetPodHandle,
        positioner: impl WindowPositioner,
        z_index: u64,
    ) -> WindowId {
        let id = self.windows.insert(Window::new(root, positioner, z_index));
        self.sorted_windows.push(id);
        self.sort_windows();
        id
    }

    pub fn hide_window(&mut self, id: WindowId) {
        self.windows[id].hide();
    }

    pub fn close_window(&mut self, id: WindowId) {
        self.windows.remove(id);
    }

    pub fn render(&mut self, canvas: &mut Canvas, window_logical_size: Vec2) {
        for id in &self.sorted_windows {
            if let Some(window) = self.windows.get_mut(*id) {
                window.render(
                    canvas,
                    &mut self.style_engine,
                    &mut self.messages,
                    window_logical_size,
                );
            }
        }
    }

    pub fn convert_event(
        &mut self,
        event: &WindowEvent,
        window_scale_factor: f64,
    ) -> Option<Event> {
        self.event_tracker.handle_event(event, window_scale_factor)
    }

    pub fn handle_window_event(
        &mut self,
        canvas: &mut Canvas,
        event: &WindowEvent,
        window_scale_factor: f64,
        window_logical_size: Vec2,
    ) {
        if let Some(event) = self.convert_event(event, window_scale_factor) {
            for (_, window) in &mut self.windows {
                window.handle_event(
                    canvas,
                    &mut self.style_engine,
                    &mut self.messages,
                    &event,
                    window_logical_size,
                );
            }
        }
    }

    /// Invokes `callback` on all messages with a given type.
    /// Drains the messages.
    ///
    /// Skips any messages with a type other than `T`.
    pub fn handle_messages<T: 'static>(&mut self, mut callback: impl FnMut(&T)) {
        self.messages.retain(|message| {
            if let Some(message) = message.downcast_ref::<T>() {
                callback(message);
                false // delete message - it was handled
            } else {
                true // keep message - unhandled
            }
        });
    }

    pub fn pop_message<T: 'static>(&mut self) -> Option<T> {
        let mut index = None;
        for (i, msg) in self.messages.iter().enumerate() {
            if msg.downcast_ref::<T>().is_some() {
                index = Some(i);
                break;
            }
        }

        match index {
            Some(i) => Some(*self.messages.remove(i).unwrap().downcast().unwrap()),
            None => None,
        }
    }

    fn sort_windows(&mut self) {
        let windows = &self.windows;
        self.sorted_windows.retain(|w| windows.contains_key(*w));
        self.sorted_windows.sort_by_key(|w| windows[*w].z_index)
    }
}

fn instantiate_widget(
    ui: &Ui,
    spec_widget: &spec::Widget,
    widgets_with_ids: &mut Vec<(String, WidgetPodHandle)>,
) -> WidgetPodHandle {
    let widget: Box<dyn DynWidget> = match spec_widget {
        spec::Widget::Column(spec) => {
            Box::new(widgets::Flex::from_spec(&spec.flex, Axis::Vertical))
        }
        spec::Widget::Row(spec) => Box::new(widgets::Flex::from_spec(&spec.flex, Axis::Horizontal)),
        spec::Widget::Text(spec) => Box::new(widgets::Text::from_spec(spec)),
        spec::Widget::TextInput(spec) => Box::new(widgets::TextInput::from_spec(spec)),
        spec::Widget::Button(spec) => Box::new(widgets::Button::from_spec(spec)),
        spec::Widget::Image(spec) => Box::new(widgets::Image::from_spec(spec)),
        spec::Widget::Container(spec) => Box::new(widgets::Container::from_spec(spec)),
        spec::Widget::ProgressBar(spec) => Box::new(widgets::ProgressBar::from_spec(spec)),
        spec::Widget::Clickable(spec) => Box::new(widgets::Clickable::from_spec(spec)),
        spec::Widget::Slider(spec) => Box::new(widgets::Slider::from_spec(spec)),
        spec::Widget::Table(spec) => Box::new(widgets::Table::from_spec(spec)),
        spec::Widget::Divider(spec) => Box::new(widgets::Divider::from_spec(&spec)),
        spec::Widget::Scrollable(spec) => Box::new(widgets::Scrollable::from_spec(spec)),
        spec::Widget::PickList(spec) => Box::new(widgets::PickList::from_spec(spec)),
        spec::Widget::Tooltip(_spec) => Box::new(widgets::Tooltip::new()),
        spec::Widget::Custom(spec) => ui
            .custom_widget_builders
            .get(&spec.typ)
            .unwrap_or_else(|| panic!("missing custom widget '{}'", spec.typ))(
            &spec.params
        ),
    };

    let mut pod = WidgetPod::new(widget);

    if let Some(base) = spec_widget.base_spec() {
        pod.data_mut().set_flex(base.flex);

        for class in &base.classes {
            pod.data_mut().add_class(class);
        }
    }

    // Ensure `Widget::style_updated` isn't called for initialization.
    pod.data_mut().mark_classes_clean();

    // Inflate children recursively.
    let children = spec_widget.children();

    for child in children {
        let child = instantiate_widget(ui, child, widgets_with_ids);
        pod.data_mut().add_child(child);
    }

    // Special case for Tooltip, because its children
    // cannot be represented as a slice
    if let spec::Widget::Tooltip(s) = spec_widget {
        for child in [&s.child, &s.tooltip] {
            if let Some(child) = child {
                let child = instantiate_widget(ui, child, widgets_with_ids);
                pod.data_mut().add_child(child);
            }
        }
    }

    pod.mount();

    let handle = Rc::new(RefCell::new(pod));

    if let Some(id) = spec_widget.base_spec().and_then(|b| b.id.as_ref()) {
        widgets_with_ids.push((id.clone(), Rc::clone(&handle)));
    }

    handle
}
