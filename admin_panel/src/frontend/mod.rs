use crate::backend::{Backend, BackendCommand, FrontendEvent};
use crate::frontend::dialog::Dialog;
use crate::frontend::easy_mark::EasyMarkEditor;
use crate::frontend::left_block::LeftBlockScreen;
use crate::frontend::right_block::RightBlockScreen;
use crate::frontend::ui_kit::UiKit;
use egui::{Align2, Vec2};
use std::sync::mpsc::Sender;

mod dialog;
pub mod easy_mark;
mod left_block;
mod notification;
pub(crate) mod right_block;
mod ui_kit;

pub struct Frontend {
    left_block_screen: LeftBlockScreen,
    right_block_screen: RightBlockScreen,

    show_deleted_files: bool,

    markup_editor: EasyMarkEditor,

    to_backend: Sender<FrontendEvent>,
    dialog: Dialog,
    pub(crate) backend: Backend,
}

impl Frontend {
    pub fn emit_event(&self, e: FrontendEvent) {
        self.to_backend.send(e).unwrap()
    }
    pub fn new(backend: Backend, to_backend: Sender<FrontendEvent>) -> Self {
        Self {
            left_block_screen: Default::default(),
            right_block_screen: Default::default(),
            dialog: Dialog::None,
            show_deleted_files: false,
            to_backend,
            backend,
            markup_editor: Default::default(),
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        const LEFT_BLOCK_SIZE: f32 = 120.;
        const NOTIFICATIONS_BLOCK: f32 = 160.;

        egui::CentralPanel::default().show(ctx, |ui| {
            let inner_rect = ui.min_rect();

            self.draw_dialog(ctx, inner_rect.size());

            ui.separator();

            ui.horizontal(|ui| {
                ui.set_height(inner_rect.height());

                ui.separator();

                self.draw_left_block(ui, LEFT_BLOCK_SIZE);

                ui.separator();

                self.draw_right_block(
                    ui,
                    inner_rect.width() - LEFT_BLOCK_SIZE - NOTIFICATIONS_BLOCK - 80.,
                );

                ui.separator();

                self.draw_notifications(ui, NOTIFICATIONS_BLOCK);

                ui.separator();
            })
        });
    }

    fn draw_dialog(&mut self, ctx: &egui::Context, rect: Vec2) {
        let mut close = false;

        match &mut self.dialog {
            Dialog::CreateFolder { dir, name } => {
                egui::Window::new("Create folder")
                    .id(egui::Id::new("_warn_"))
                    .collapsible(false)
                    .resizable(false)
                    .default_pos([rect.x / 3.0, rect.y / 3.0])
                    .pivot(Align2::CENTER_CENTER)
                    .show(ctx, |ui| {
                        ui.set_width(200.);
                        ui.vertical_centered(|ui| {
                            ui.label(format!("In directory {dir}"));

                            ui.text_edit_singleline(name);

                            ui.horizontal(|ui| {
                                if ui.button_s("Create", 60., 1.).clicked() {
                                    self.to_backend
                                        .send(FrontendEvent::CreateFolder {
                                            dir: dir.to_string(),
                                            name: name.to_string(),
                                        })
                                        .unwrap();
                                    close = true;
                                }

                                ui.add_space(72.);

                                if ui.button_s("Cancel", 60., 1.).clicked() {
                                    close = true;
                                }
                            });
                        })
                    });
            }

            Dialog::None => {}
        }

        if close {
            self.dialog = Dialog::None;
        }
    }

    pub fn on_update(&mut self) {
        for v in self.backend.on_update() {
            match v {
                BackendCommand::OpenFileObserve { dir } => {
                    self.right_block_screen = RightBlockScreen::Files;
                }

                BackendCommand::OpenLogs => self.right_block_screen = RightBlockScreen::Logs,

                BackendCommand::OpenPatchNotes => {
                    self.right_block_screen = RightBlockScreen::PatchNotes
                }

                BackendCommand::OpenPatchNote { id, data } => {
                    self.markup_editor.code = data;
                    self.markup_editor.edit_id = id;

                    self.right_block_screen = RightBlockScreen::EditPatchNote;
                }
            }
        }
    }
}

pub fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../../files/Nunito-Black.ttf")),
    );
    fonts.font_data.insert(
        "my_icons".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../files/Font Awesome 6 Free-Regular-400.otf"
        )),
    );
    fonts.font_data.insert(
        "my_icons2".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../files/Font Awesome 6 Free-Solid-900.otf"
        )),
    );
    fonts.font_data.insert(
        "my_brands".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../files/Font Awesome 6 Brands-Regular-400.otf"
        )),
    );

    fonts
        .families
        .entry(egui::FontFamily::Name("icons".into()))
        .or_default()
        .push("my_icons".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Name("icons".into()))
        .or_default()
        .push("my_icons2".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Name("icons".into()))
        .or_default()
        .push("my_brands".to_owned());

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}
