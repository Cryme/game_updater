use crate::log::app_log;
use shared::file::ServerFileInfo;
use shared::file::{ServerFolderInfo, ROOT_FOLDER_INFO_FILE_NAME};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing::log::Level;
use tracing::{debug, error, info};

static INSTANCE: OnceLock<RwLock<FileUpdater>> = OnceLock::new();

pub struct FileUpdater {
    pub root_folder: ServerFolderInfo,
}

impl FileUpdater {
    pub fn file_info(&self, folder_path: &str, file_name: &str) -> Option<&ServerFileInfo> {
        let Some(folder) = self.folder_info(folder_path) else {
            return None;
        };

        folder.files.get(file_name)
    }

    /// returns (mut_ref_to_file_info, was_file_info_just_created)
    pub fn get_or_create_file_info(
        &mut self,
        folder_path: &str,
        file_name: &str,
    ) -> (&mut ServerFileInfo, bool) {
        let mut current_folder = &mut self.root_folder;

        for folder in folder_path.split("/") {
            if !current_folder.folders.contains_key(folder) {
                current_folder
                    .folders
                    .insert(folder.to_string(), ServerFolderInfo::default());
            }

            current_folder = current_folder.folders.get_mut(folder).unwrap();
        }

        let exists = current_folder.files.contains_key(file_name);

        if !exists {
            current_folder
                .files
                .insert(file_name.to_string(), ServerFileInfo::default());
        }

        (current_folder.files.get_mut(file_name).unwrap(), !exists)
    }

    pub fn folder_info_mut(&mut self, path: &str) -> Option<&mut ServerFolderInfo> {
        let mut current_folder = &mut self.root_folder;

        if path.is_empty() {
            return Some(current_folder);
        }

        for f in path.split("/") {
            let Some(p) = current_folder.folders.get_mut(f) else {
                return None;
            };

            current_folder = p;
        }

        Some(current_folder)
    }

    pub fn folder_info(&self, path: &str) -> Option<&ServerFolderInfo> {
        let mut current_folder = &self.root_folder;

        if path.is_empty() {
            return Some(current_folder);
        }

        for f in path.split("/") {
            let Some(p) = current_folder.folders.get(f) else {
                return None;
            };

            current_folder = p;
        }

        Some(current_folder)
    }

    pub async fn instance<'a>() -> RwLockReadGuard<'a, FileUpdater> {
        INSTANCE
            .get_or_init(|| RwLock::new(Self::new()))
            .read()
            .await
    }

    pub async fn instance_mut<'a>() -> RwLockWriteGuard<'a, FileUpdater> {
        INSTANCE
            .get_or_init(|| RwLock::new(Self::new()))
            .write()
            .await
    }

    fn new() -> Self {
        let root_folder = if let Ok(file) = File::open(format!("./{ROOT_FOLDER_INFO_FILE_NAME}")) {
            if let Ok(d) = ron::de::from_reader::<File, ServerFolderInfo>(file) {
                debug!("Files count: {}", d.files_count);

                d
            } else {
                error!("Corrupted root folder info ./{ROOT_FOLDER_INFO_FILE_NAME}");

                ServerFolderInfo::default()
            }
        } else {
            info!("No root folder info was found!");

            ServerFolderInfo::default()
        };

        Self { root_folder }
    }
}

trait AsyncOps {
    /// Writes opcode, which means that **raw file** will be sent
    ///
    /// Flush **is not** called!
    async fn write_file_prefix(&mut self) -> tokio::io::Result<usize>;
    /// Writes opcode, which means that **packet** will be sent
    ///
    /// Flush **is not** called!
    async fn write_packet_prefix(&mut self) -> tokio::io::Result<usize>;
}

impl<W: AsyncWrite + Unpin> AsyncOps for W {
    async fn write_file_prefix(&mut self) -> tokio::io::Result<usize> {
        self.write(&[0x0]).await
    }
    async fn write_packet_prefix(&mut self) -> tokio::io::Result<usize> {
        self.write(&[0x1]).await
    }
}

impl FileUpdater {
    pub async fn add_file(folder_path: &str, file_name: &str, file_hash: String, bytes: &[u8]) {
        let mut lock = Self::instance_mut().await;

        let Ok(mut file) =
            std::fs::File::create(Path::new("./compressed").join(folder_path).join(file_name))
        else {
            app_log(Level::Error, &format!("Can't create file {folder_path}!")).await;

            return;
        };

        let Ok(_) = file.write_all(bytes) else {
            app_log(
                Level::Error,
                &format!("Can't write to created file {folder_path}!"),
            )
            .await;

            return;
        };

        let (file_info, just_created) = lock.get_or_create_file_info(folder_path, file_name);

        if just_created {
            file_info.created_at = chrono::Utc::now().timestamp();
        }

        file_info.hash = file_hash;
        file_info.size = bytes.len() as u64;
        file_info.updated_at = file_info.created_at;
        file_info.deleted = false;

        drop(lock);
    }

    pub async fn send_file<W: AsyncWrite + Unpin>(
        folder_path: &str,
        file_name: &str,
        out_stream: &mut W,
    ) -> bool {
        let lock = Self::instance().await;

        if let Some(info) = lock.file_info(folder_path, file_name) {
            if info.deleted {
                return false;
            }

            let Ok(mut file) =
                tokio::fs::File::open(Path::new("./compressed").join(folder_path).join(file_name))
                    .await
            else {
                app_log(
                    Level::Error,
                    &format!("File {folder_path} is presented in filelist, but not exists!"),
                )
                .await;

                return false;
            };

            let Ok(_) = out_stream.write_file_prefix().await else {
                return false;
            };

            let Ok(_) = tokio::io::copy(&mut file, out_stream).await else {
                return false;
            };

            return true;
        }

        drop(lock);

        false
    }
}
