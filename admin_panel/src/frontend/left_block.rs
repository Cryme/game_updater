use crate::backend::{FrontendEvent, Screen};
use crate::frontend::right_block::RightBlockScreen;
use crate::frontend::ui_kit::UiKit;
use crate::frontend::Frontend;
use egui::Ui;

#[derive(Default)]
pub enum LeftBlockScreen {
    #[default]
    CommonMenu,
}

impl Frontend {
    pub fn draw_left_block(&mut self, ui: &mut Ui, width: f32) {
        ui.vertical(|ui| {
            if ui
                .left_menu_button(
                    "Dashboard",
                    self.right_block_screen == RightBlockScreen::Dashboard,
                    width,
                )
                .clicked()
            {
                self.to_backend
                    .send(FrontendEvent::RequestOpenScreen(Screen::Dashboard))
                    .unwrap();
            }

            if ui
                .left_menu_button(
                    "Patch notes",
                    matches!(
                        self.right_block_screen,
                        RightBlockScreen::PatchNotes { .. }
                    ),
                    width,
                )
                .clicked()
            {
                self.to_backend
                    .send(FrontendEvent::RequestOpenScreen(Screen::PatchNotes))
                    .unwrap();
            }

            if ui
                .left_menu_button(
                    "Files",
                    matches!(self.right_block_screen, RightBlockScreen::Files { .. }),
                    width,
                )
                .clicked()
            {
                self.to_backend
                    .send(FrontendEvent::RequestOpenScreen(Screen::Files {
                        dir: "./".to_string(),
                    }))
                    .unwrap();
            }

            if ui
                .left_menu_button(
                    "Logs",
                    self.right_block_screen == RightBlockScreen::Logs,
                    width,
                )
                .clicked()
            {
                self.to_backend
                    .send(FrontendEvent::RequestOpenScreen(Screen::Logs))
                    .unwrap();
            }
        });
    }
}
