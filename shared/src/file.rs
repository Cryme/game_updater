use chrono::Utc;
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hasher;
use std::io::Write;

pub static COMPRESSED_FOLDER_NAME: &str = "compressed";
pub static ROOT_FOLDER_INFO_FILE_NAME: &str = "root_folder_server_info.ron";

#[derive(Debug, Deserialize, Serialize)]
pub struct FileList {
    pub files: HashMap<String, FileInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileInfo {
    hash: String,
    last_updated: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ServerFolderInfo {
    pub size: u64,
    pub files_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted: bool,
    pub files: HashMap<String, ServerFileInfo>,
    pub folders: HashMap<String, ServerFolderInfo>,
}

impl ServerFolderInfo {
    pub fn calc_size(&mut self) -> (u64, u32) {
        self.size = 0;
        self.files_count = 0;

        self.files.values().for_each(|v| {
            self.size += v.size;
            self.files_count += 1;
        });

        self.folders.values_mut().for_each(|v| {
            let (size, files_count) = v.calc_size();
            self.size += size;
            self.files_count += files_count;
        });

        (self.size, self.files_count)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ServerFileInfo {
    pub hash: String,
    pub size: u64,
    pub created_at: i64,
    pub updated_at: i64,
    /// Для некоторых файлов, например *.ini*, в которых хранятся настройки пользователей, мы не
    /// хотим проверять хэш - только наличие файла
    pub skip_hash_check: bool,
    pub deleted: bool,
}

pub fn hash_of(bytes_too_hash: &[u8]) -> String {
    let mut hasher = SeaHasher::default();

    hasher.write(bytes_too_hash);

    format!("{:016x}", hasher.finish())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn compress_in_mem<W: Write>(bytes_to_compress: &[u8], buff: &mut W) -> anyhow::Result<()> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;

    let mut e = ZlibEncoder::new(buff, Compression::best());

    e.write_all(bytes_to_compress)?;
    e.finish()?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn decompress_in_mem<W: Write>(bytes_to_decompress: &[u8], buff: &mut W) -> anyhow::Result<()> {
    use flate2::write::ZlibDecoder;

    let mut e = ZlibDecoder::new(buff);

    e.write_all(bytes_to_decompress)?;
    e.finish()?;

    Ok(())
}
