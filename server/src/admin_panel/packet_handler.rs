use shared::admin_panel::{ClientPacket, FileInfo, FolderInfo, Log, LogLevel, ServerPacket};
use std::time::Duration;
use tracing::log::{debug};

pub(crate) trait HandleClientPacket {
    async fn handle(self) -> Option<ServerPacket>;
}

impl HandleClientPacket for ClientPacket {
    async fn handle(self) -> Option<ServerPacket> {
        match self {
            ClientPacket::FileList { dir } => Some(ServerPacket::FileList {
                dir,
                files: (0..10)
                    .map(|v| FileInfo {
                        name: format!("Abrglavitof {v}"),
                        size: 1024 * 1024 * 160 - v * 1024 * 1024,
                        created: chrono::offset::Local::now().timestamp() - 400 - 4 * v as i64,
                        modified_at: chrono::offset::Local::now().timestamp() - 100 - 4 * v as i64,
                        updated_by: Default::default(),
                    })
                    .collect(),
                folders: (0..4)
                    .map(|v| FolderInfo {
                        name: format!("Folder {v}"),
                        size: 1024 * 1024 * 160 - v * 1024 * 1024,
                        created: chrono::offset::Local::now().timestamp() - 400 - 4 * v as i64,
                        modified_at: chrono::offset::Local::now().timestamp() - 100 - 4 * v as i64,
                        updated_by: Default::default(),
                    })
                    .collect(),
            }),
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
                tokio::time::sleep(Duration::from_secs(4)).await;

                Some(ServerPacket::FileUploaded { id })
            }
            ClientPacket::PatchNotes => {
                todo!()
            }
            ClientPacket::EditPatchNote { id, data } => {
                todo!()
            }
            ClientPacket::AddPatchNote { data } => {
                todo!()
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
        }
    }
}
