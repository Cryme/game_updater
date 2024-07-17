use crate::backend::notification::Notification;
use crate::frontend::ui_kit::{icon, CLOSE_TOKEN};
use crate::frontend::Frontend;
use egui::{Align, Color32, CursorIcon, Label, Layout, RichText, Ui};

impl Frontend {
    pub(crate) fn draw_notifications(&mut self, ui: &mut Ui, width: f32) {
        let mut remove = None;
        ui.vertical(|ui| {
            ui.set_width(width);

            for (i, v) in self.backend.notifications.iter().enumerate() {
                v.draw(ui, width, || remove = Some(i));
                ui.add_space(10.);
            }
        });

        if let Some(i) = remove {
            self.backend.notifications.remove(i);
        }
    }
}

impl Notification {
    pub fn draw(&self, ui: &mut Ui, width: f32, callback: impl FnOnce()) {
        let (text, label) = match self {
            Notification::FileUpload {
                id,
                dir,
                name,
                state,
            } => (
                format!("Status: {state}\nFile: {name}\nDir: {dir}"),
                "File Upload",
            ),
        };

        ui.vertical_centered(|ui| {
            ui.set_width(width);

            egui::Frame::default()
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .rounding(ui.visuals().widgets.noninteractive.rounding)
                .show(ui, |ui| {
                    egui::Frame {
                        inner_margin: 6.0.into(),
                        outer_margin: 0.0.into(),
                        rounding: 6.0.into(),
                        shadow: egui::Shadow::NONE,
                        fill: Color32::from_rgb(73, 84, 77),
                        stroke: egui::Stroke::new(1.0, Color32::GRAY),
                    }
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        ui.horizontal(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                ui.set_width(width - 30.);

                                if ui
                                    .label(icon(CLOSE_TOKEN).size(16.).color(Color32::LIGHT_GRAY))
                                    .on_hover_cursor(CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    callback();
                                };

                                ui.vertical_centered_justified(|ui| {
                                    ui.add(
                                        Label::new(
                                            RichText::new(label).size(16.).color(Color32::WHITE),
                                        )
                                        .selectable(false),
                                    );
                                });

                                ui.add(
                                    Label::new(
                                        icon(CLOSE_TOKEN).size(16.).color(Color32::TRANSPARENT),
                                    )
                                    .selectable(false),
                                )
                                .on_hover_cursor(CursorIcon::Default);
                            });
                        });
                        ui.label(RichText::new(text).size(13.).color(Color32::WHITE));
                    });
                });
        });
    }
}
