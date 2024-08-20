use crate::log::app_log;
use shared::file::{compress_in_mem, hash_of, ServerFileInfo, COMPRESSED_FOLDER_NAME};
use shared::file::{ServerFolderInfo, ROOT_FOLDER_INFO_FILE_NAME};
use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::task::spawn_blocking;
use tracing::log::Level;
use tracing::{error, info};

static INSTANCE: OnceLock<RwLock<FileHolder>> = OnceLock::new();

pub struct FileHolder {
    pub root_folder: ServerFolderInfo,
}

impl FileHolder {
    pub fn dump(&self) {
        let mut file = match std::fs::File::create(format!("./database/{ROOT_FOLDER_INFO_FILE_NAME}")) {
            Ok(v) => v,
            Err(e) => {
                error!("Can't create file: {e}");

                return;
            }
        };

        if let Err(e) = file.write_all(
            ron::ser::to_string_pretty(&self.root_folder, ron::ser::PrettyConfig::default())
                .unwrap()
                .as_bytes(),
        ) {
            error!("Can't write updater metadata: {e}");
        }
    }

    fn file_info(&self, folder_path: &str, file_name: &str) -> Option<&ServerFileInfo> {
        let Some(folder) = self.folder_info(folder_path) else {
            return None;
        };

        folder.files.get(file_name)
    }

    fn file_info_mut(&mut self, folder_path: &str, file_name: &str) -> Option<&mut ServerFileInfo> {
        let Some(folder) = self.folder_info_mut(folder_path) else {
            return None;
        };

        folder.files.get_mut(file_name)
    }

    /// returns (mut_ref_to_file_info, was_file_info_just_created)
    fn get_or_create_file_info(
        &mut self,
        folder_path: &str,
        file_name: &str,
    ) -> (&mut ServerFileInfo, bool) {
        let mut current_folder = &mut self.root_folder;
        if !folder_path.is_empty() {
            for folder in folder_path.split('/') {
                if !current_folder.folders.contains_key(folder) {
                    current_folder
                        .folders
                        .insert(folder.to_string(), ServerFolderInfo::default());
                }

                current_folder = current_folder.folders.get_mut(folder).unwrap();
            }
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

        for f in path.split('/') {
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

        for f in path.split('/') {
            let Some(p) = current_folder.folders.get(f) else {
                return None;
            };

            current_folder = p;
        }

        Some(current_folder)
    }

    pub async fn instance<'a>() -> RwLockReadGuard<'a, FileHolder> {
        INSTANCE
            .get_or_init(|| RwLock::new(Self::new()))
            .read()
            .await
    }

    pub async fn instance_mut<'a>() -> RwLockWriteGuard<'a, FileHolder> {
        INSTANCE
            .get_or_init(|| RwLock::new(Self::new()))
            .write()
            .await
    }

    pub fn info(&self) -> String {
        format!("File Holder:\n\tTotal files: {}\n", self.root_folder.files_count)
    }

    fn new() -> Self {
        let root_folder =
            if let Ok(file) = std::fs::File::open(format!("./database/{ROOT_FOLDER_INFO_FILE_NAME}")) {
                if let Ok(d) = ron::de::from_reader::<std::fs::File, ServerFolderInfo>(file) {
                    d
                } else {
                    error!("Corrupted root folder info: ./database/{ROOT_FOLDER_INFO_FILE_NAME}");

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

impl FileHolder {
    pub async fn create_folder(parent_folder_path: &str, new_folder_name: &str) -> bool {
        let Ok(_) = std::fs::create_dir(Path::new(COMPRESSED_FOLDER_NAME).join(parent_folder_path).join(new_folder_name)) else {
            return false;
        };

        let mut instance = Self::instance_mut().await;

        let Some(f) = instance.folder_info_mut(parent_folder_path) else {
            return false;
        };

        f.folders
            .insert(new_folder_name.to_string(), ServerFolderInfo::default());

        instance.dump();

        true
    }

    pub async fn add_file(folder_path: &str, file_name: &str, bytes: Vec<u8>) -> bool {
        let initial_size = bytes.len();

        let Ok((hash, compressed_bytes)) = spawn_blocking(move || {
            let mut out = vec![];
            compress_in_mem(&bytes, &mut out).unwrap();
            (hash_of(&bytes), out)
        })
        .await
        else {
            return false;
        };

        let mut instance = Self::instance_mut().await;

        let Ok(mut file) =
            std::fs::File::create(Path::new("./compressed").join(folder_path).join(file_name))
        else {
            app_log(Level::Error, &format!("Can't create file {folder_path}")).await;

            return false;
        };

        let Ok(_) = file.write_all(&compressed_bytes) else {
            app_log(
                Level::Error,
                &format!("Can't write to created file {folder_path}"),
            )
            .await;

            return false;
        };

        let (file_info, just_created) = instance.get_or_create_file_info(folder_path, file_name);

        if just_created {
            file_info.created_at = chrono::Utc::now().timestamp();
        }

        file_info.hash = hash;
        file_info.size = initial_size as u64;
        file_info.updated_at = chrono::Utc::now().timestamp();
        file_info.deleted = false;

        instance.root_folder.calc_size();

        instance.dump();

        true
    }

    pub async fn delete_file(folder_path: &str, file_name: &str) -> bool {
        let mut instance = Self::instance_mut().await;

        let Some(file_info) = instance.file_info_mut(folder_path, file_name) else {
            return false;
        };

        file_info.deleted = !file_info.deleted;

        instance.dump();

        true
    }

    pub async fn toggle_hash_check(folder_path: &str, file_name: &str) -> bool {
        let mut instance = Self::instance_mut().await;

        let Some(file_info) = instance.file_info_mut(folder_path, file_name) else {
            return false;
        };

        file_info.skip_hash_check = !file_info.skip_hash_check;

        instance.dump();

        true
    }

    pub async fn delete_folder(folder_path: &str) -> bool {
        let mut instance = Self::instance_mut().await;

        let Some(folder_info) = instance.folder_info_mut(folder_path) else {
            return false;
        };

        folder_info.deleted = !folder_info.deleted;

        instance.dump();

        true
    }

    pub async fn send_file<W: AsyncWrite + Unpin>(
        folder_path: &str,
        file_name: &str,
        out_stream: &mut W,
    ) -> bool {
        let instance = Self::instance().await;

        if let Some(info) = instance.file_info(folder_path, file_name) {
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

        drop(instance);

        false
    }
}
