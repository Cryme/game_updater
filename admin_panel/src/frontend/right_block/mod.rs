mod file_info;
mod logs;
mod patchnotes;

use crate::frontend::Frontend;
use egui::Ui;

const PATCH_NOTE_WIDTH: f32 = 800.;

#[derive(Default, Eq, PartialEq)]
pub enum RightBlockScreen {
    #[default]
    Dashboard,
    PatchNotes,
    EditPatchNote,
    Files,
    Logs,
}

impl Frontend {
    pub fn draw_right_block(&mut self, ui: &mut Ui, width: f32) {
        ui.scope(|ui| {
            ui.set_width(width);

            match &mut self.right_block_screen {
                RightBlockScreen::Dashboard => {}

                RightBlockScreen::PatchNotes => self.draw_patch_notes(ui, PATCH_NOTE_WIDTH),

                RightBlockScreen::EditPatchNote => self.draw_patch_note_editor(ui),

                RightBlockScreen::Files => {
                    self.draw_file_infos(ui, &self.backend.file_info_holder.current_dir.clone())
                }

                RightBlockScreen::Logs => self.draw_logs(ui),
            }
        });
    }
}
