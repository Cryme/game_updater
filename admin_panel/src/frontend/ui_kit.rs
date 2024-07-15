use crate::backend::log_holder::LogLevel;

use egui::{
    Button, Color32, CursorIcon, Label, Response, RichText, Sense, TextWrapMode, Ui, Vec2,
    WidgetText,
};
use std::fmt::Display;
use strum::IntoEnumIterator;

pub const DELETE_ICON: &str = "🗑";

pub trait Draw<F> {
    fn draw(&self, ui: &mut Ui, callback: F);
}

pub trait UiKit {
    fn left_menu_button(&mut self, text: impl Into<String>, selected: bool) -> Response;
    fn label_u(&mut self, text: impl Into<WidgetText>) -> Response;
}

impl UiKit for Ui {
    fn left_menu_button(&mut self, text: impl Into<String>, selected: bool) -> Response {
        let b = Button::new(RichText::new(text).size(16.))
            .selected(selected)
            .min_size(Vec2::new(120., 20.));

        self.add(b).on_hover_cursor(CursorIcon::PointingHand)
    }

    fn label_u(&mut self, text: impl Into<WidgetText>) -> Response {
        self.add(Label::new(text).selectable(false).sense(Sense::click()))
    }
}

pub fn combo_box_row<T: Display + PartialEq + Copy + IntoEnumIterator>(
    ui: &mut Ui,
    val: &mut T,
    label: &str,
) {
    ui.horizontal(|ui| {
        if !label.is_empty() {
            ui.add(egui::Label::new(label));
        }
        egui::ComboBox::from_id_source(ui.next_auto_id())
            .selected_text(format!("{}", val))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                ui.set_min_width(20.0);

                for t in T::iter() {
                    ui.selectable_value(val, t, format!("{t}"));
                }
            });
    });
}

impl From<LogLevel> for Color32 {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Debug => Color32::from_rgb(118, 171, 204),
            LogLevel::Info => Color32::from_rgb(196, 210, 221),
            LogLevel::Warning => Color32::from_rgb(238, 146, 62),
            LogLevel::Error => Color32::from_rgb(238, 62, 62),
        }
    }
}
