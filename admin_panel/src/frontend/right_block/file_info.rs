use crate::backend::file_info_holder::{FileInfoHolder, FileSortBy, SortDir};
use crate::backend::FrontendEvent;
use crate::frontend::dialog::Dialog;
use crate::frontend::right_block::RightBlockScreen;
use crate::frontend::ui_kit::{DrawCb, UiKit, DELETE_ICON};
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
                        .label_u(RichText::new(v).underline().color(Color32::WHITE))
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        self.to_backend
                            .send(FrontendEvent::ChangeScreen(RightBlockScreen::Files {
                                dir: prev.clone(),
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

                                log!(Level::Debug, "{}", f.file_name());
                            }

                            t.send(FrontendEvent::UploadFiles { dir, files }).unwrap();
                        }
                    })
                }
            });

            ui.separator();
            ui.separator();

            self.backend.file_info_holder.draw_sort(ui);

            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                for f in self.backend.file_info_holder.folders() {
                    let f1 = || {
                        self.emit_event(FrontendEvent::DeleteFile {
                            dir: dir.to_string(),
                            name: f.name.clone(),
                        })
                    };
                    let f2 = || {
                        self.emit_event(FrontendEvent::ChangeScreen(RightBlockScreen::Files {
                            dir: dir.to_string() + "/" + &f.name,
                        }))
                    };
                    f.draw_cb(ui, (f1, f2));

                    ui.separator();
                }

                for f in self.backend.file_info_holder.files() {
                    f.draw_cb(ui, || {
                        self.emit_event(FrontendEvent::DeleteFile {
                            dir: dir.to_string(),
                            name: f.name.clone(),
                        })
                    });

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

impl<F: FnOnce()> DrawCb<F> for FileInfo {
    fn draw_cb(&self, ui: &mut Ui, callback: F) {
        ui.horizontal(|ui| {
            ui.set_height(ROW_HEIGHT);

            ui.scope(|ui| {
                ui.set_width(FileSortBy::Name.width());

                ui.label(RichText::new(&self.name).color(Color32::WHITE));
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

            if ui
                .add(Button::new(DELETE_ICON).fill(Color32::DARK_RED))
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                callback();
            }
        });
    }
}
impl<F1: FnOnce(), F2: FnOnce()> DrawCb<(F1, F2)> for FolderInfo {
    fn draw_cb(&self, ui: &mut Ui, callback: (F1, F2)) {
        ui.horizontal(|ui| {
            ui.set_height(ROW_HEIGHT);

            ui.scope(|ui| {
                ui.set_width(FileSortBy::Name.width());

                if ui
                    .label_u(
                        RichText::new(&self.name)
                            .color(Color32::LIGHT_BLUE)
                            .underline(),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
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

            if ui
                .add(Button::new(DELETE_ICON).fill(Color32::DARK_RED))
                .on_hover_cursor(CursorIcon::PointingHand)
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
        }
    }
}
