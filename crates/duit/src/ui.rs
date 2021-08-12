use std::{any::Any, cell::RefCell, collections::VecDeque, rc::Rc};

use ahash::AHashMap;
use duit_core::spec::{self, Spec};
use dume_renderer::Canvas;
use glam::Vec2;
use winit::event::WindowEvent;

use crate::{
    event::EventTracker,
    spec::InstanceHandle,
    style::{StyleEngine, StyleError},
    widget::{DynWidget, WidgetPod, WidgetPodHandle},
    widgets::{self, flex::Axis},
    window::{Window, WindowPositioner},
};

/// Contains the entire UI state, including all windows and their widget trees.
#[derive(Default)]
pub struct Ui {
    windows: Vec<Window>,
    specs: AHashMap<String, Spec>,
    style_engine: StyleEngine,
    event_tracker: EventTracker,
    messages: VecDeque<Box<dyn Any>>,
}

impl Ui {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_spec(&mut self, spec: Spec) -> &mut Self {
        self.specs.insert(spec.name.clone(), spec);
        self
    }

    pub fn add_stylesheet(&mut self, stylesheet_bytes: &[u8]) -> Result<&mut Self, StyleError> {
        self.style_engine.append_sheet(stylesheet_bytes)?;
        Ok(self)
    }

    pub fn create_spec_instance<S: InstanceHandle>(&mut self) -> (S, WidgetPodHandle) {
        let spec = self
            .specs
            .get(S::name())
            .unwrap_or_else(|| panic!("spec '{}' is not registered with the UI", S::name()));

        let mut widgets_with_ids = Vec::new();

        // Instantiate the widget tree.
        let root = instantiate_widget(&spec.child, &mut widgets_with_ids);

        let spec_handle = S::init(widgets_with_ids);
        (spec_handle, root)
    }

    pub fn create_window(
        &mut self,
        root: WidgetPodHandle,
        positioner: impl WindowPositioner,
        z_index: u64,
    ) {
        self.windows.push(Window::new(root, positioner, z_index));
        self.sort_windows();
    }

    pub fn render(&mut self, canvas: &mut Canvas, window_logical_size: Vec2) {
        for window in &mut self.windows {
            window.render(
                canvas,
                &mut self.style_engine,
                &mut self.messages,
                window_logical_size,
            );
        }
    }

    pub fn handle_window_event(
        &mut self,
        canvas: &mut Canvas,
        event: &WindowEvent,
        window_scale_factor: f64,
    ) {
        if let Some(event) = self.event_tracker.handle_event(event, window_scale_factor) {
            for window in &mut self.windows {
                window.handle_event(canvas, &mut self.style_engine, &mut self.messages, &event);
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

    fn sort_windows(&mut self) {
        self.windows.sort_by_key(|w| w.z_index)
    }
}

fn instantiate_widget(
    spec_widget: &spec::Widget,
    widgets_with_ids: &mut Vec<(String, WidgetPodHandle)>,
) -> WidgetPodHandle {
    let widget: Box<dyn DynWidget> = match &spec_widget {
        spec::Widget::Column(spec) => {
            Box::new(widgets::Flex::from_spec(&spec.flex, Axis::Vertical))
        }
        spec::Widget::Row(spec) => Box::new(widgets::Flex::from_spec(&spec.flex, Axis::Horizontal)),
        spec::Widget::Text(spec) => Box::new(widgets::Text::from_spec(spec)),
        spec::Widget::TextInput(_) => todo!(),
        spec::Widget::Button(spec) => Box::new(widgets::Button::from_spec(spec)),
        spec::Widget::Image(spec) => Box::new(widgets::Image::from_spec(spec)),
    };

    let mut pod = WidgetPod::new(widget);

    let base_class = pod.widget.base_class().to_owned();
    pod.data_mut().add_class(&base_class);

    if let Some(base) = spec_widget.base_spec() {
        pod.data_mut().set_flex(base.flex);

        for class in &base.classes {
            pod.data_mut().add_class(class);
        }
    }

    // Ensure `Widget::style_updated` isn't called for initialization.
    pod.data_mut().mark_classes_clean();

    // Inflate children recursively.
    let children = match spec_widget {
        spec::Widget::Column(s) => s.flex.children.as_slice(),
        spec::Widget::Row(s) => s.flex.children.as_slice(),
        spec::Widget::Button(s) => std::slice::from_ref(&*s.child),
        spec::Widget::Image(s) => match &s.child {
            Some(c) => std::slice::from_ref(&**c),
            None => &[],
        },
        _ => &[],
    };

    for child in children {
        let child = instantiate_widget(child, widgets_with_ids);
        pod.data_mut().add_child(child);
    }

    pod.mount();

    let handle = Rc::new(RefCell::new(pod));

    if let Some(id) = spec_widget.base_spec().and_then(|b| b.id.as_ref()) {
        widgets_with_ids.push((id.clone(), Rc::clone(&handle)));
    }

    handle
}
