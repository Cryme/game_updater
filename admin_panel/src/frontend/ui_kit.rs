use egui::{
    Button, Color32, CursorIcon, FontFamily, Label, Response, RichText, Sense, TextWrapMode, Ui,
    Vec2, WidgetText,
};
use shared::admin_panel::LogLevel;
use std::fmt::Display;
use strum::IntoEnumIterator;

pub const DELETE_ICON: &str = "🗑";
pub const CLOSE_TOKEN: &str = "\u{f00d}";

pub trait DrawCb<F> {
    fn draw_cb(&self, ui: &mut Ui, callback: F);
}

pub trait UiKit {
    fn left_menu_button(&mut self, text: impl Into<String>, selected: bool, width: f32)
        -> Response;
    fn label_u(&mut self, text: impl Into<WidgetText>) -> Response;
    fn button_s(&mut self, text: impl Into<WidgetText>, width: f32, height: f32) -> Response;
}

impl UiKit for Ui {
    fn left_menu_button(
        &mut self,
        text: impl Into<String>,
        selected: bool,
        width: f32,
    ) -> Response {
        let b = Button::new(RichText::new(text).size(16.))
            .selected(selected)
            .min_size(Vec2::new(width, 20.));

        self.add(b).on_hover_cursor(CursorIcon::PointingHand)
    }

    fn label_u(&mut self, text: impl Into<WidgetText>) -> Response {
        self.add(Label::new(text).selectable(false).sense(Sense::click()))
    }

    fn button_s(&mut self, text: impl Into<WidgetText>, width: f32, height: f32) -> Response {
        self.add(Button::new(text).min_size(Vec2::new(width, height)))
            .on_hover_cursor(CursorIcon::PointingHand)
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

pub trait AsColor {
    fn as_color(&self) -> Color32;
}

impl AsColor for LogLevel {
    fn as_color(&self) -> Color32 {
        match self {
            LogLevel::Debug => Color32::from_rgb(118, 171, 204),
            LogLevel::Info => Color32::from_rgb(196, 210, 221),
            LogLevel::Warning => Color32::from_rgb(238, 146, 62),
            LogLevel::Error => Color32::from_rgb(238, 62, 62),
        }
    }
}

pub fn icon(s: &str) -> RichText {
    RichText::new(s).family(FontFamily::Name("icons".into()))
}
