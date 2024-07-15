use crate::backend::{Backend, BackendCommand, FrontendEvent};
use crate::frontend::right_block::RightBlockScreen;
use log::{log, Level};
use shared::admin_panel::ClientPacket;

impl Backend {
    pub fn handle_frontend_events(&mut self) -> Vec<BackendCommand> {
        let mut res = vec![];

        for v in self.from_frontend.try_iter() {
            log!(Level::Debug, "handle event");

            match v {
                FrontendEvent::UploadFiles { dir, files } => {
                    res.push(BackendCommand::ShowFileUploading {
                        files: files.iter().map(|v| v.0.clone()).collect(),
                    });
                    for f in files {
                        self.send_packet(ClientPacket::AddFile {
                            dir: dir.clone(),
                            name: f.0,
                            file: f.1,
                        });
                    }
                }

                FrontendEvent::DeleteFile { dir, name } => {
                    self.send_packet(ClientPacket::RemoveFile { dir, name })
                }

                FrontendEvent::ChangeScreen(new_screen) => match new_screen {
                    RightBlockScreen::Dashboard => {}
                    RightBlockScreen::PatchNotes => {}
                    RightBlockScreen::EditPatchNote { id } => {}
                    RightBlockScreen::Files { dir } => {
                        self.send_packet(ClientPacket::FileList { dir: dir.clone() });
                    }
                    RightBlockScreen::Logs => {}
                },
            }
        }

        res
    }
}
