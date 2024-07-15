use crate::backend::FrontendEvent;
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
    pub fn draw_left_block(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            if ui
                .left_menu_button(
                    "Dashboard",
                    self.right_block_screen == RightBlockScreen::Dashboard,
                )
                .clicked()
            {
                self.to_backend
                    .send(FrontendEvent::ChangeScreen(RightBlockScreen::Dashboard))
                    .unwrap();
            }

            if ui
                .left_menu_button(
                    "Patch notes",
                    self.right_block_screen == RightBlockScreen::PatchNotes,
                )
                .clicked()
            {
                self.to_backend
                    .send(FrontendEvent::ChangeScreen(RightBlockScreen::PatchNotes))
                    .unwrap();
            }

            if ui
                .left_menu_button(
                    "Files",
                    if matches!(self.right_block_screen, RightBlockScreen::Files { .. }) {
                        true
                    } else {
                        false
                    },
                )
                .clicked()
            {
                self.to_backend
                    .send(FrontendEvent::ChangeScreen(RightBlockScreen::Files {
                        dir: "system/info".to_string(),
                    }))
                    .unwrap();
            }

            if ui
                .left_menu_button("Logs", self.right_block_screen == RightBlockScreen::Logs)
                .clicked()
            {
                self.to_backend
                    .send(FrontendEvent::ChangeScreen(RightBlockScreen::Logs))
                    .unwrap();
            }
        });
    }
}
