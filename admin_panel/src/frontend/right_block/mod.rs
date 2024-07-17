mod file_info;
mod logs;

use crate::frontend::Frontend;
use egui::Ui;

#[derive(Default, Eq, PartialEq)]
pub enum RightBlockScreen {
    #[default]
    Dashboard,
    PatchNotes,
    EditPatchNote {
        id: u32,
    },
    Files {
        dir: String,
    },
    Logs,
}

impl Frontend {
    pub fn draw_right_block(&mut self, ui: &mut Ui, width: f32) {
        ui.scope(|ui| {
            ui.set_width(width);

            match &mut self.right_block_screen {
                RightBlockScreen::Dashboard => {}
                RightBlockScreen::PatchNotes => {}
                RightBlockScreen::EditPatchNote { id } => {}
                RightBlockScreen::Files { dir } => {
                    let s = dir.clone();

                    self.draw_file_infos(ui, &s);
                }
                RightBlockScreen::Logs => self.draw_logs(ui),
            }
        });
    }
}
