use crate::frontend::ui_kit::{combo_box_row, AsColor};
use crate::frontend::Frontend;
use eframe::epaint::text::TextWrapMode;
use eframe::epaint::Color32;
use egui::{RichText, ScrollArea, Ui};
use shared::admin_panel::{LogHolder, LogLevelFilter};

impl Frontend {
    pub(crate) fn draw_logs(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.separator();

            ui.horizontal(|ui| {
                combo_box_row(ui, &mut self.backend.log_holder.level_filter, "Level");
                ui.label("Producer");
                egui::ComboBox::from_id_source(ui.next_auto_id())
                    .selected_text(&self.backend.log_holder.producer_filter)
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        ui.set_min_width(20.0);

                        let mut c = self
                            .backend
                            .log_holder
                            .producers
                            .iter()
                            .collect::<Vec<&String>>();
                        c.sort();
                        for t in c {
                            ui.selectable_value(
                                &mut self.backend.log_holder.producer_filter,
                                t.clone(),
                                t,
                            );
                        }
                    });
            });

            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    for log in self.backend.log_holder.server_logs.iter().filter(|v| {
                        let a = self.backend.log_holder.producer_filter == LogHolder::ALL
                            || self.backend.log_holder.producer_filter == v.producer;

                        let b = self.backend.log_holder.level_filter == LogLevelFilter::All
                            || self.backend.log_holder.level_filter as u8 == v.level as u8;

                        a && b
                    }) {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "{}",
                                chrono::DateTime::from_timestamp(log.time, 0)
                                    .unwrap()
                                    .format("%d %b %H:%M")
                            ));
                            ui.label(RichText::new(&log.producer).color(Color32::WHITE));
                            ui.label(RichText::new(&log.log).color(log.level.as_color()));
                        });

                        ui.add_space(5.0);
                    }
                });
            });

            ui.separator();
        });
    }
}
