use crate::backend::notification::{FileUploadState, Notification};
use crate::backend::{Backend, BackendCommand};
use shared::admin_panel::ServerPacket;

impl Backend {
    pub(crate) fn handle_packets(&mut self) -> Vec<BackendCommand> {
        let mut res = vec![];

        while let Ok(v) = self.from_server.try_recv() {
            self.debug(&format!("{:?}", v));

            match v {
                ServerPacket::FileList {
                    dir,
                    files,
                    folders,
                } => {
                    self.file_info_holder.set(files, folders);
                    res.push(BackendCommand::OpenFileObserve {
                        dir: if dir.is_empty() {
                            ".".to_string()
                        } else {
                            "./".to_owned() + &*dir
                        },
                    });
                }

                ServerPacket::Logs(logs) => {
                    self.log_holder.add_server(logs);

                    res.push(BackendCommand::OpenLogs);
                }

                ServerPacket::FileUploaded { id } => {
                    self.debug(&format!("{:#?}", self.notifications));

                    for v in &mut self.notifications {
                        match v {
                            Notification::FileUpload {
                                id: v_id, state, ..
                            } => {
                                if id.eq(v_id) {
                                    *state = FileUploadState::Processing;
                                }
                            }
                        }
                    }
                }

                ServerPacket::PatchNotes {
                    total,
                    patch_notes,
                    take,
                    skip,
                } => {
                    self.patch_note_holder.patch_notes = patch_notes;
                    self.patch_note_holder.total = total;
                    self.patch_note_holder.take = take;
                    self.patch_note_holder.skip = skip;

                    res.push(BackendCommand::OpenPatchNotes);
                }

                ServerPacket::OpenPatchNote(patch_note) => {
                    res.push(BackendCommand::OpenPatchNote {
                        id: Some(patch_note.id),
                        data: patch_note.data,
                    })
                }
            }
        }

        res
    }
}
