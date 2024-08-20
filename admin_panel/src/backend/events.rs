use crate::backend::notification::{FileUploadState, Notification};
use crate::backend::{Backend, BackendCommand, FrontendEvent, Screen};
use crate::frontend::easy_mark::DEFAULT_CODE;
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
                            dir: remove_leading_dot(&dir),
                            name: f.0,
                            file: f.1,
                        });
                    }
                }

                FrontendEvent::RemoveFile { dir, name } => {
                    self.send_packet(ClientPacket::RemoveFile {
                        dir: remove_leading_dot(&dir),
                        name,
                    })
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
                            dir: remove_leading_dot(&dir),
                        });
                    }
                    Screen::Logs => {
                        self.send_packet(ClientPacket::Logs);
                    }
                },

                FrontendEvent::CreateFolder { dir, name } => {
                    self.send_packet(ClientPacket::CreateFolder {
                        dir: remove_leading_dot(&dir),
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

                FrontendEvent::SkipFileHashCheck { dir, name } => {
                    self.send_packet(ClientPacket::SkipFileHashCheck {
                        dir: remove_leading_dot(&dir),
                        name,
                    })
                }
                FrontendEvent::RemoveFolder { dir, name } => {
                    self.send_packet(ClientPacket::RemoveFolder {
                        dir: remove_leading_dot(&dir),
                        name,
                    })
                }
            }
        }

        res
    }
}

pub fn remove_leading_dot(dir: &str) -> String {
    log!(Level::Debug, "|{dir}|");

    if dir.len() == 1 {
        "".to_string()
    } else {
        dir[2..].to_string()
    }
}
