use crate::file_updater::FileHolder;
use shared::admin_panel::{
    ClientPacket, FileInfo, FolderInfo, Log, LogLevel, PatchNote, ServerPacket,
};
use tokio::sync::mpsc::Sender;
use tracing::log::debug;
use crate::db::Database;

pub(crate) trait HandleClientPacket {
    async fn handle(self, to_client: Sender<ServerPacket>);
}

impl HandleClientPacket for ClientPacket {
    async fn handle(self, to_client: Sender<ServerPacket>) {
        match self {
            ClientPacket::FileList { dir } => {
                let Some((folders, files)) = FileHolder::get_folder_and_file_infos(&dir).await
                else {
                    return;
                };

                let _ = to_client
                    .send(ServerPacket::FileList {
                        dir,
                        files,
                        folders,
                    })
                    .await;
            }

            ClientPacket::CreateFolder { dir, name } => {
                FileHolder::create_folder(&dir, &name).await;

                debug!(">>> Created folder {name} in dir: {dir}");

                let Some((folders, files)) = FileHolder::get_folder_and_file_infos(&dir).await
                else {
                    return;
                };

                let _ = to_client
                    .send(ServerPacket::FileList {
                        dir,
                        files,
                        folders,
                    })
                    .await;
            }

            ClientPacket::RemoveFile { dir, name } => {
                FileHolder::delete_file(&dir, &name).await;

                debug!(">>> Deleted file {name} in dir: {dir}");

                let Some((folders, files)) = FileHolder::get_folder_and_file_infos(&dir).await
                else {
                    return;
                };

                let _ = to_client
                    .send(ServerPacket::FileList {
                        dir,
                        files,
                        folders,
                    })
                    .await;
            }

            ClientPacket::RemoveFolder { dir, name } => {
                if dir.is_empty() {
                    FileHolder::delete_folder(&name).await;
                } else {
                    FileHolder::delete_folder(&format!("{dir}/{name}")).await;
                }

                debug!(">>> Deleted folder {name} in dir {dir}");

                let Some((folders, files)) = FileHolder::get_folder_and_file_infos(&dir).await
                else {
                    return;
                };

                let _ = to_client
                    .send(ServerPacket::FileList {
                        dir,
                        files,
                        folders,
                    })
                    .await;
            }

            ClientPacket::AddFile {
                id,
                dir,
                name,
                file,
            } => {
                let _ = to_client.send(ServerPacket::FileUploaded { id }).await;

                FileHolder::add_file(&dir, &name, file).await;

                debug!(">>> File {name} added to dir {dir}");

                let _ = to_client.send(ServerPacket::FileProceeded { id }).await;
            }

            ClientPacket::PatchNotes { take, skip } => {
                let patch_notes = Database::instance().patch_notes().await;

                let _ = to_client
                    .send(ServerPacket::PatchNotes {
                        take: patch_notes.len() as u32,
                        skip: 0,
                        total: patch_notes.len() as u32,
                        patch_notes,
                    })
                    .await;
            }

            ClientPacket::SavePatchNote { id, data } => {
                Database::instance().update_patch_note(id, data).await;
                debug!(">>> Edit patch note {id}!");
            }

            ClientPacket::DeletePatchNote { id } => {
                debug!(">>> Delete patch note {id}!");
            }

            ClientPacket::AddPatchNote { data } => {
                debug!(">>> Create patch note");
                let patch_note = Database::instance().add_patch_note(data).await;

                let _ = to_client
                    .send(ServerPacket::OpenPatchNote(patch_note))
                    .await;
            }

            ClientPacket::Logs => {
                let _ = to_client
                    .send(ServerPacket::Logs(vec![Log {
                        level: LogLevel::Info,
                        producer: "Monitor".to_string(),
                        log: "bla bla".to_string(),
                        time: chrono::Local::now().timestamp(),
                    }]))
                    .await;
            }

            ClientPacket::RequestEditPatchNote { id } => {
                let _ = to_client
                    .send(ServerPacket::OpenPatchNote(PatchNote {
                        id,
                        data: TEMP.to_string(),
                    }))
                    .await;
            }

            ClientPacket::SkipFileHashCheck { dir, name } => {
                FileHolder::toggle_hash_check(&dir, &name).await;

                debug!(">>> Toggled hash check for file {name} in dir: {dir}");

                let Some((folders, files)) = FileHolder::get_folder_and_file_infos(&dir).await
                else {
                    return;
                };

                let _ = to_client
                    .send(ServerPacket::FileList {
                        dir,
                        files,
                        folders,
                    })
                    .await;
            }
        }
    }
}

impl FileHolder {
    async fn get_folder_and_file_infos(dir: &str) -> Option<(Vec<FolderInfo>, Vec<FileInfo>)> {
        let instance = Self::instance().await;

        let info = instance.folder_info(dir)?;

        let mut folders = Vec::with_capacity(info.folders.len());
        let mut files = Vec::with_capacity(info.files.len());

        for (k, v) in &info.folders {
            folders.push(FolderInfo {
                name: k.to_string(),
                size: v.size,
                created: v.created_at,
                modified_at: v.updated_at,
                updated_by: 0,
                deleted: v.deleted,
            })
        }

        for (k, v) in &info.files {
            files.push(FileInfo {
                name: k.to_string(),
                size: v.size,
                created: v.created_at,
                modified_at: v.updated_at,
                updated_by: 0,
                skip_hash_check: v.skip_hash_check,
                deleted: v.deleted,
            })
        }

        Some((folders, files))
    }
}

static TEMP: &str = r#"
# [MarkupLanguage](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax)
----------------

# At a glance
- inline text:
  - normal, `code`, **strong**, ~~strikethrough~~, *italics*, ***strong italics***
  - `\` escapes the next character
  - [hyperlink](https://github.com/emilk/egui)
  - Embedded URL: <https://github.com/emilk/egui>
- `# ` header
- `---` separator (horizontal line)
- `> ` quote
- `- ` bullet list
- `1. ` numbered list
- \`\`\` code fence
"#;
