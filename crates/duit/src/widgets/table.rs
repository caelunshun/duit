use std::{cell::Cell, iter, mem, rc::Rc};

use ahash::AHashMap;
use duit_core::spec::widgets::TableSpec;
use glam::{vec2, Vec2};

use crate::{widget::Context, Color, Widget, WidgetData, WidgetPodHandle};

type ColumnName = Rc<str>;

/// A tabular representation of data.
///
/// Has a fixed list of columns, each of which has a name.
/// Programmaticaly, you should add data by calling [`add_row`].
/// Each row consists of a widget for each column.
pub struct Table {
    columns: Vec<ColumnName>,
    column_widths: AHashMap<ColumnName, f32>,
    column_offsets: AHashMap<ColumnName, f32>,
    rows: Vec<Row>,
    empty_rows: u32,
    the_empty_row: Row,
}

#[derive(Default)]
struct Row {
    widgets: AHashMap<ColumnName, WidgetPodHandle>,
    height: Cell<f32>,
}

impl Table {
    pub fn from_spec(spec: &TableSpec) -> Self {
        Self {
            columns: spec.columns.iter().map(|c| c.as_str().into()).collect(),
            column_widths: AHashMap::new(),
            column_offsets: AHashMap::new(),
            rows: Vec::new(),
            empty_rows: spec.empty_rows,
            the_empty_row: Row::default(),
        }
    }

    fn find_column_name(&self, name: &str) -> ColumnName {
        self.columns
            .iter()
            .find(|c| c.as_ref() == name)
            .unwrap_or_else(|| panic!("unknown table column '{}'", name))
            .clone()
    }

    pub fn add_row<'a>(&mut self, widgets: impl IntoIterator<Item = (&'a str, WidgetPodHandle)>) {
        let mut row = Row::default();
        for (name, widget) in widgets {
            let name = self.find_column_name(name);
            row.widgets.insert(name, widget);
        }
        self.rows.push(row);
    }

    pub fn clear_rows(&mut self) {
        self.rows.clear();
    }

    pub fn add_column(&mut self, name: &str) {
        self.columns.push(name.into());
    }

    pub fn remove_column(&mut self, name: &str) {
        let pos = self.columns.iter().position(|c| c.as_ref() == name);
        if let Some(pos) = pos {
            self.columns.remove(pos);
        }
    }

    fn rows(&self) -> impl Iterator<Item = &Row> + '_ {
        self.rows
            .iter()
            .chain(iter::repeat(&self.the_empty_row).take(self.empty_rows as usize))
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Style {
    background_color_a: Color,
    background_color_b: Color,
    border_color: Color,
    border_width: f32,
    min_row_height: f32,
    min_column_width: f32,
    cell_padding: f32,
}

impl Widget for Table {
    type Style = Style;

    fn base_class(&self) -> &str {
        "table"
    }

    fn layout(
        &mut self,
        style: &Self::Style,
        data: &mut WidgetData,
        mut cx: Context,
        max_size: Vec2,
    ) {
        let mut column_widths = mem::take(&mut self.column_widths);
        column_widths.clear();

        let mut size = vec2(0., 0.);
        for row in self.rows() {
            // Compute the height of this row,
            // which is the maximum of all its widgets'
            // heights, plus cell padding.
            //
            // Similarly, column width is the maximum of its widgets'
            // widths, plus cell padding.
            let mut row_height = 0.0f32;
            for (column_name, widget) in &row.widgets {
                let mut widget = widget.borrow_mut();
                widget.layout(&mut cx, max_size);

                widget
                    .data_mut()
                    .set_origin(vec2(0., size.y + style.cell_padding));

                let size = widget.data().size();
                row_height = row_height.max(size.y);

                let column_width = column_widths.entry(column_name.clone()).or_default();
                *column_width = column_width.max(size.x);
            }

            row_height = row_height.max(style.min_row_height);
            row_height += 2. * style.cell_padding;

            row.height.set(row_height);
            size.y += row_height;
        }

        for column in &self.columns {
            let mut column_width = column_widths.get(column).copied().unwrap_or_default();
            column_width = column_width.max(style.min_column_width);
            column_width += 2. * style.cell_padding;

            column_widths.insert(column.clone(), column_width);
            size.x += column_width;
        }

        data.set_size(size);

        // Set widget X positions
        let mut cursor = 0.;
        for column in &self.columns {
            let column_width = column_widths[column];

            for row in self.rows() {
                if let Some(widget) = row.widgets.get(column) {
                    let mut widget = widget.borrow_mut();
                    let mut origin = widget.data().origin();
                    origin.x = cursor + style.cell_padding;
                    widget.data_mut().set_origin(origin);
                }
            }

            self.column_offsets.insert(column.clone(), cursor);
            cursor += column_width;
        }

        self.column_widths = column_widths;
    }

    fn paint(&mut self, style: &Self::Style, _data: &mut WidgetData, mut cx: Context) {
        let mut cursor_y = 0.0f32;
        for (i, row) in self.rows().enumerate() {
            for column in &self.columns {
                // Background / border
                let pos = vec2(self.column_offsets[column], cursor_y);
                let size = vec2(self.column_widths[column], row.height.get());

                let color = if i % 2 == 0 {
                    style.background_color_a
                } else {
                    style.background_color_b
                };

                cx.canvas
                    .begin_path()
                    .rect(pos, size)
                    .solid_color(color.into())
                    .fill();
                cx.canvas
                    .stroke_width(style.border_width)
                    .solid_color(style.border_color.into())
                    .stroke();
            }

            for widget in row.widgets.values() {
                widget.borrow_mut().paint(&mut cx);
            }

            cursor_y += row.height.get();
        }
    }

    fn handle_event(&mut self, _data: &mut WidgetData, mut cx: Context, event: &crate::Event) {
        for row in self.rows() {
            for widget in row.widgets.values() {
                widget.borrow_mut().handle_event(&mut cx, event);
            }
        }
    }

    fn paint_overlay(&mut self, _style: &Self::Style, _data: &mut WidgetData, mut cx: Context) {
        for row in self.rows() {
            for widget in row.widgets.values() {
                widget.borrow_mut().paint_overlay(&mut cx);
            }
        }
    }
}
