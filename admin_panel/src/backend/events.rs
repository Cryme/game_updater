use crate::backend::notification::{FileUploadState, Notification};
use crate::backend::{Backend, BackendCommand, FrontendEvent, Screen};
use crate::frontend::easy_mark::DEFAULT_CODE;
use crate::frontend::right_block::RightBlockScreen;
use log::{log, Level};
use shared::admin_panel::ClientPacket;
use uuid::Uuid;

impl Backend {
    pub fn handle_frontend_events(&mut self) -> Vec<BackendCommand> {
        let mut res = vec![];

        for v in self.from_frontend.try_iter() {
            match v {
                FrontendEvent::UploadFiles { dir, files } => {
                    let id = Uuid::new_v4();
                    for f in files {
                        self.notifications.push(Notification::FileUpload {
                            id,
                            dir: dir.clone(),
                            name: f.0.clone(),
                            state: FileUploadState::Uploading,
                        });

                        self.send_packet(ClientPacket::AddFile {
                            id,
                            dir: dir.clone(),
                            name: f.0,
                            file: f.1,
                        });
                    }
                }

                FrontendEvent::DeleteFile { dir, name } => {
                    self.send_packet(ClientPacket::RemoveFile { dir, name })
                }

                FrontendEvent::RequestOpenScreen(new_screen) => match new_screen {
                    Screen::Dashboard => {
                        todo!()
                    }
                    Screen::PatchNotes => {
                        //TODO: add take and skip!
                        self.send_packet(ClientPacket::PatchNotes { take: 10, skip: 0 });
                    }
                    Screen::EditPatchNote { id } => {
                        if let Some(id) = id {
                            self.send_packet(ClientPacket::RequestEditPatchNote { id });
                        } else {
                            res.push(BackendCommand::OpenPatchNote {
                                id: None,
                                data: DEFAULT_CODE.to_string(),
                            })
                        }
                    }
                    Screen::Files { dir } => {
                        log!(Level::Debug, "Navigate to |{}|", dir);

                        self.send_packet(ClientPacket::FileList {
                            dir: if dir.len() == 1 {
                                "".to_string()
                            } else {
                                dir[2..].to_string()
                            },
                        });
                    }
                    Screen::Logs => {
                        self.send_packet(ClientPacket::Logs);
                    }
                },

                FrontendEvent::CreateFolder { dir, name } => {
                    self.send_packet(ClientPacket::CreateFolder {
                        dir: if dir.len() == 1 {
                            "".to_string()
                        } else {
                            dir[2..].to_string()
                        },
                        name,
                    })
                }

                FrontendEvent::SavePatchNote { id, data } => {
                    if let Some(id) = id {
                        self.send_packet(ClientPacket::SavePatchNote { id, data })
                    } else {
                        self.send_packet(ClientPacket::AddPatchNote { data })
                    }
                }

                FrontendEvent::DeletePatchNote { id } => {
                    self.send_packet(ClientPacket::DeletePatchNote { id })
                }

                FrontendEvent::SkipFileHashCheck { dir, name, val } => {
                    self.send_packet(ClientPacket::SkipFileHashCheck { dir, name, val })
                }
            }
        }

        res
    }
}
