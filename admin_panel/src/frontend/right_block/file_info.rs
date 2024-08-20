use crate::backend::file_info_holder::{FileInfoHolder, FileSortBy, SortDir};
use crate::backend::{FrontendEvent, Screen};
use crate::frontend::dialog::Dialog;
use crate::frontend::ui_kit::{icon, DrawCb, UiKit, DELETE_TOKEN, RESTORE_TOKEN};
use crate::frontend::Frontend;
use bytesize::ByteSize;
use eframe::epaint::Color32;
use egui::{Button, CursorIcon, RichText, ScrollArea, Stroke, Ui, Vec2};
use log::{log, Level};
use shared::admin_panel::{FileInfo, FolderInfo};
use strum::IntoEnumIterator;
use wasm_bindgen_futures::spawn_local;

const ROW_HEIGHT: f32 = 15.;

impl Frontend {
    pub(crate) fn draw_file_infos(&mut self, ui: &mut Ui, dir: &str) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Folder: ");

                let mut prev = "".to_string();

                let dirs = dir.split('/');

                for v in dirs {
                    prev += v;

                    if ui
                        .clickable_label(RichText::new(v).underline().color(Color32::WHITE))
                        .clicked()
                    {
                        self.to_backend
                            .send(FrontendEvent::RequestOpenScreen(Screen::Files {
                                dir: prev.to_string(),
                            }))
                            .unwrap();
                    }

                    ui.label("/");

                    prev += "/";
                }
            });

            ui.horizontal(|ui| {
                ui.scope(|ui| {
                    ui.set_width(110.);

                    ui.label(format!(
                        "Folders: {}",
                        self.backend.file_info_holder.folders_count()
                    ));
                });

                if ui.button_s("Create Folder", 100., 1.).clicked() {
                    self.show_dialog(Dialog::CreateFolder {
                        dir: dir.to_string(),
                        name: "New Folder".to_string(),
                    })
                }
            });

            ui.horizontal(|ui| {
                ui.scope(|ui| {
                    ui.set_width(110.);

                    ui.label(format!(
                        "Files: {}",
                        self.backend.file_info_holder.files_count()
                    ));
                });

                if ui.button_s("Upload Files", 100., 1.).clicked() {
                    let t = self.to_backend.clone();
                    let dir = dir.to_string();

                    spawn_local(async move {
                        if let Some(entries) = rfd::AsyncFileDialog::new().pick_files().await {
                            if entries.is_empty() {
                                return;
                            }

                            let mut files = vec![];
                            for f in entries {
                                let content = f.read().await;
                                files.push((f.file_name(), content));

                                log!(Level::Debug, "Uploading {}", f.file_name());
                            }

                            t.send(FrontendEvent::UploadFiles { dir, files }).unwrap();
                        }
                    })
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_deleted_files, "Show deleted");
            });

            ui.separator();

            self.backend.file_info_holder.draw_sort(ui);

            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                for f in self.backend.file_info_holder.folders() {
                    let f1 = || {
                        self.emit_event(FrontendEvent::RemoveFolder {
                            dir: dir.to_string(),
                            name: f.name.to_string(),
                        })
                    };

                    let f2 = || {
                        self.emit_event(FrontendEvent::RequestOpenScreen(Screen::Files {
                            dir: format!("{dir}/{}", f.name),
                        }))
                    };
                    f.draw_cb(ui, (f1, f2));

                    ui.separator();
                }

                for f in self.backend.file_info_holder.files() {
                    if !self.show_deleted_files && f.deleted {
                        continue
                    }

                    f.draw_cb(
                        ui,
                        (
                            || {
                                self.emit_event(FrontendEvent::RemoveFile {
                                    dir: dir.to_string(),
                                    name: f.name.clone(),
                                })
                            },
                            || {
                                self.emit_event(FrontendEvent::SkipFileHashCheck {
                                    dir: dir.to_string(),
                                    name: f.name.clone(),
                                })
                            },
                        ),
                    );

                    ui.separator();
                }
            });
        });
    }
}

impl FileInfoHolder {
    fn draw_sort(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let sp = ui.style().spacing.item_spacing.x;

            for (i, v) in FileSortBy::iter().enumerate() {
                ui.style_mut().spacing.item_spacing.x = 0.;

                if ui
                    .add(
                        Button::new(RichText::new(format!("{v}")).color(Color32::WHITE))
                            .min_size(Vec2::new(
                                v.width() + sp * (i.min(1) + 1) as f32,
                                ROW_HEIGHT,
                            ))
                            .stroke(Stroke::NONE)
                            .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    if v == self.sort_by {
                        self.sort_dir = !self.sort_dir;
                        self.sort();
                    } else {
                        self.sort_by = v;
                        self.sort_dir = SortDir::Desc;
                        self.sort();
                    }
                }

                ui.separator();
            }
        });
    }
}

impl<F1: FnOnce(), F2: FnOnce()> DrawCb<(F1, F2)> for FileInfo {
    fn draw_cb(&self, ui: &mut Ui, callback: (F1, F2)) {
        ui.horizontal(|ui| {
            ui.set_height(ROW_HEIGHT);

            ui.scope(|ui| {
                let (text, tooltip) = if self.name.len() > 25 {
                    (&format!("{}...", &self.name[0..25]), true)
                } else {
                    (&self.name, false)
                };

                ui.set_width(FileSortBy::Name.width());

                let mut l = ui.label(if self.deleted {
                    RichText::new(text).color(Color32::GRAY).strikethrough()
                } else {
                    RichText::new(text).color(Color32::WHITE)
                });

                if tooltip {
                    l = l.on_hover_ui(|ui| {
                        ui.label(&self.name);
                    });
                }
            });

            ui.separator();

            ui.scope(|ui| {
                ui.set_width(FileSortBy::Size.width());
                ui.set_height(ROW_HEIGHT);

                ui.label(ByteSize::b(self.size).to_string());
            });

            ui.separator();

            ui.scope(|ui| {
                ui.set_width(FileSortBy::CreatedAt.width());
                ui.set_height(ROW_HEIGHT);

                ui.label(format!(
                    "{}",
                    chrono::DateTime::from_timestamp(self.created, 0)
                        .unwrap()
                        .format("%d/%m/%y %H:%M")
                ));
            });

            ui.separator();

            ui.scope(|ui| {
                ui.set_width(FileSortBy::ModifiedAt.width());
                ui.set_height(ROW_HEIGHT);

                ui.label(format!(
                    "{}",
                    chrono::DateTime::from_timestamp(self.modified_at, 0)
                        .unwrap()
                        .format("%d/%m/%y %H:%M")
                ));
            });

            ui.separator();

            ui.scope(|ui| {
                ui.set_width(FileSortBy::CheckHash.width());
                ui.set_height(ROW_HEIGHT);

                let mut v = !self.skip_hash_check;

                if ui.checkbox(&mut v, "").changed() {
                    callback.1();
                }
            });

            ui.separator();

            if ui
                .clickable_label(if self.deleted {
                    icon(RESTORE_TOKEN).size(16.).color(Color32::DARK_GREEN)
                } else {
                    icon(DELETE_TOKEN).size(16.).color(Color32::DARK_RED)
                })
                .clicked()
            {
                callback.0();
            }
        });
    }
}
impl<F1: FnOnce(), F2: FnOnce()> DrawCb<(F1, F2)> for FolderInfo {
    fn draw_cb(&self, ui: &mut Ui, callback: (F1, F2)) {
        ui.horizontal(|ui| {
            ui.set_height(ROW_HEIGHT);

            ui.scope(|ui| {
                let (text, tooltip) = if self.name.len() > 25 {
                    (&format!("{}...", &self.name[0..25]), true)
                } else {
                    (&self.name, false)
                };

                ui.set_width(FileSortBy::Name.width());

                let mut l = ui.clickable_label(if self.deleted {
                    RichText::new(text)
                        .color(Color32::GRAY)
                        .strikethrough()
                        .underline()
                } else {
                    RichText::new(&self.name)
                        .color(Color32::LIGHT_BLUE)
                        .underline()
                });

                if tooltip {
                    l = l.on_hover_ui(|ui| {
                        ui.label(&self.name);
                    });
                }

                if l.clicked() {
                    callback.1()
                }
            });

            ui.separator();

            ui.scope(|ui| {
                ui.set_width(FileSortBy::Size.width());
                ui.set_height(ROW_HEIGHT);

                ui.label(ByteSize::b(self.size).to_string());
            });

            ui.separator();

            ui.scope(|ui| {
                ui.set_width(FileSortBy::CreatedAt.width());
                ui.set_height(ROW_HEIGHT);

                ui.label(format!(
                    "{}",
                    chrono::DateTime::from_timestamp(self.created, 0)
                        .unwrap()
                        .format("%d/%m/%y %H:%M")
                ));
            });

            ui.separator();

            ui.scope(|ui| {
                ui.set_width(FileSortBy::ModifiedAt.width());
                ui.set_height(ROW_HEIGHT);

                ui.label(format!(
                    "{}",
                    chrono::DateTime::from_timestamp(self.modified_at, 0)
                        .unwrap()
                        .format("%d/%m/%y %H:%M")
                ));
            });

            ui.separator();

            ui.scope(|ui| {
                ui.set_width(FileSortBy::CheckHash.width());
                ui.set_height(ROW_HEIGHT);
            });

            ui.separator();

            if ui
                .clickable_label(if self.deleted {
                    icon(RESTORE_TOKEN).size(16.).color(Color32::DARK_GREEN)
                } else {
                    icon(DELETE_TOKEN).size(16.).color(Color32::DARK_RED)
                })
                .clicked()
            {
                callback.0();
            }
        });
    }
}

impl FileSortBy {
    fn width(&self) -> f32 {
        match self {
            FileSortBy::Name => 200.,
            FileSortBy::Size => 70.,
            FileSortBy::ModifiedAt | FileSortBy::CreatedAt => 160.,
            FileSortBy::CheckHash => 100.,
        }
    }
}
