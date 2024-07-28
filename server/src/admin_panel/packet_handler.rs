use crate::file_updater::FileUpdater;
use shared::admin_panel::{
    ClientPacket, FileInfo, FolderInfo, Log, LogLevel, PatchNote, ServerPacket,
};
use std::fs::File;
use tracing::log::debug;

pub(crate) trait HandleClientPacket {
    async fn handle(self) -> Option<ServerPacket>;
}

impl HandleClientPacket for ClientPacket {
    async fn handle(self) -> Option<ServerPacket> {
        match self {
            ClientPacket::FileList { dir } => {
                let Some((folders, files)) = FileUpdater::get_folder_and_file_infos(&dir).await
                else {
                    return None;
                };

                Some(ServerPacket::FileList {
                    dir,
                    files,
                    folders,
                })
            }
            ClientPacket::RemoveFile { dir, name } => {
                debug!("{dir} {name}");

                None
            }
            ClientPacket::AddFile {
                id,
                dir,
                name,
                file,
            } => {
                debug!(">>> Add file to dir: {dir} {name}");

                Some(ServerPacket::FileUploaded { id })
            }
            ClientPacket::PatchNotes { take, skip } => Some(ServerPacket::PatchNotes {
                take: 10,
                skip: 0,
                total: 2,
                patch_notes: vec![
                    PatchNote {
                        id: 1,
                        data: TEMP.to_string(),
                    },
                    PatchNote {
                        id: 2,
                        data: TEMP.to_string(),
                    },
                ],
            }),
            ClientPacket::SavePatchNote { id, data } => {
                debug!(">>> Edit patch note {id}!");

                None
            }
            ClientPacket::DeletePatchNote { id } => {
                debug!(">>> Delete patch note {id}!");

                None
            }
            ClientPacket::AddPatchNote { data } => {
                debug!(">>> Create patch note");

                None
            }
            ClientPacket::CreateFolder { dir, name } => {
                debug!(">>> Create folder {name} in dir: {dir}");

                None
            }
            ClientPacket::Logs => Some(ServerPacket::Logs(vec![Log {
                level: LogLevel::Info,
                producer: "Monitor".to_string(),
                log: "bla bla".to_string(),
                time: chrono::Local::now().timestamp(),
            }])),
            ClientPacket::RequestEditPatchNote { id } => {
                Some(ServerPacket::OpenPatchNote(PatchNote {
                    id,
                    data: TEMP.to_string(),
                }))
            }
            ClientPacket::SkipFileHashCheck { dir, name, val } => {
                debug!(">>> Set hash check to {val} for file {name} in dir: {dir}");

                None
            }
        }
    }
}

impl FileUpdater {
    async fn get_folder_and_file_infos(dir: &str) -> Option<(Vec<FolderInfo>, Vec<FileInfo>)> {
        let instance = Self::instance().await;

        let Some(info) = instance.folder_info(dir) else {
            return None;
        };

        let mut folders = Vec::with_capacity(info.folders.len());
        let mut files = Vec::with_capacity(info.files.len());

        for (k, v) in &info.folders {
            folders.push(FolderInfo {
                name: k.to_string(),
                size: v.size,
                created: v.created_at,
                modified_at: v.updated_at,
                updated_by: 0,
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
