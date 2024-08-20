use crate::backend::{FrontendEvent, Screen};
use crate::frontend::easy_mark::easy_mark;
use crate::frontend::ui_kit::{icon, UiKit, DELETE_TOKEN, EDIT_TOKEN};
use crate::frontend::Frontend;
use eframe::emath::Align;
use egui::{Color32, CursorIcon, Layout, ScrollArea, Ui};

impl Frontend {
    pub fn draw_patch_notes(&mut self, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            ui.label(format!("Total: {}", self.backend.patch_note_holder.total));

            if ui.button_s("Create patch note", 0., 0.).clicked() {
                self.emit_event(FrontendEvent::RequestOpenScreen(Screen::EditPatchNote {
                    id: None,
                }));
            }

            ui.separator();

            ScrollArea::vertical()
                .id_source("ptch_notes")
                .show(ui, |ui| {
                    for patch_note in &self.backend.patch_note_holder.patch_notes {
                        ui.add_space(20.);
                        ui.horizontal(|ui| {
                            ui.vertical_centered(|ui| {
                                ui.set_width(width);

                                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                    if ui
                                        .label(
                                            icon(DELETE_TOKEN).size(16.).color(Color32::DARK_RED),
                                        )
                                        .on_hover_cursor(CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        self.emit_event(FrontendEvent::DeletePatchNote {
                                            id: patch_note.id,
                                        })
                                    }

                                    if ui
                                        .label(
                                            icon(EDIT_TOKEN)
                                                .size(16.)
                                                .color(Color32::from_rgb(187, 82, 0)),
                                        )
                                        .on_hover_cursor(CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        self.emit_event(FrontendEvent::RequestOpenScreen(
                                            Screen::EditPatchNote {
                                                id: Some(patch_note.id),
                                            },
                                        ))
                                    }
                                });

                                egui::Frame::default()
                                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                                    .rounding(ui.visuals().widgets.noninteractive.rounding)
                                    .show(ui, |ui| {
                                        egui::Frame {
                                            inner_margin: 6.0.into(),
                                            outer_margin: 0.0.into(),
                                            rounding: 6.0.into(),
                                            shadow: egui::Shadow::NONE,
                                            fill: Default::default(),
                                            stroke: egui::Stroke::new(1.0, Color32::GRAY),
                                        }
                                        .show(ui, |ui| {
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Wrap);
                                            ui.horizontal(|ui| {
                                                easy_mark(ui, &patch_note.data);
                                            })
                                        })
                                    })
                            });

                            ui.add_space(5.);
                        });
                    }
                });
        });
    }
}

impl Frontend {
    pub fn draw_patch_note_editor(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            if self.markup_editor.ui(ui).clicked() {
                self.emit_event(FrontendEvent::SavePatchNote {
                    id: self.markup_editor.edit_id,
                    data: self.markup_editor.code.clone(),
                })
            };
        });
    }
}
