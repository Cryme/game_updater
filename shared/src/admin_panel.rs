use bincode::config;
use bincode::error::{DecodeError, EncodeError};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{BufReader, Cursor, Read};
use strum::{Display, EnumIter};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct TgUser {
    id: u64,
    user_name: String,
}

impl TgUser {
    pub fn test() -> Self {
        Self {
            id: 1488,
            user_name: "Test User".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub created: i64,
    pub modified_at: i64,
    pub updated_by: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FolderInfo {
    pub name: String,
    pub size: u64,
    pub created: i64,
    pub modified_at: i64,
    pub updated_by: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientPacket {
    FileList {
        dir: String,
    },
    RemoveFile {
        dir: String,
        name: String,
    },
    AddFile {
        id: Uuid,
        dir: String,
        name: String,
        file: Vec<u8>,
    },
    PatchNotes,
    EditPatchNote {
        id: u32,
        data: String,
    },
    AddPatchNote {
        data: String,
    },
    CreateFolder {
        dir: String,
        name: String,
    },
    Logs,
}

impl ClientPacket {
    pub fn is_heavy(&self) -> bool {
        match self {
            ClientPacket::AddFile { .. } => true,
            _ => false,
        }
    }

    pub fn from_bin(slice: &[u8]) -> anyhow::Result<ClientPacket> {
        match slice[0] {
            0x0 => Ok(bincode::serde::decode_from_slice::<ClientPacket, _>(
                &slice[1..],
                config::standard(),
            )
            .map(|v| v.0)?),
            0x1 => {
                let mut reader = BufReader::new(Cursor::new(&slice[1..]));

                let dir = read_string(&mut reader)?;
                let name = read_string(&mut reader)?;

                let mut uuid = [0x0u8; 16];

                reader.read_exact(&mut uuid)?;

                let id = Uuid::from_bytes(uuid);

                let mut file = vec![];
                reader.read_to_end(&mut file)?;

                Ok(ClientPacket::AddFile {
                    id,
                    dir,
                    name,
                    file,
                })
            }
            _ => {
                unimplemented!()
            }
        }
    }
    pub fn to_bin(&self) -> Result<Vec<u8>, EncodeError> {
        let mut res = vec![0x0u8];
        match self {
            ClientPacket::AddFile {
                id,
                dir,
                name,
                file,
            } => {
                res[0] = 0x1;

                let b = dir.as_bytes();
                res.extend(b.len().to_le_bytes());
                res.extend(b);

                let b = name.as_bytes();
                res.extend(b.len().to_le_bytes());
                res.extend(b);

                res.extend(id.as_bytes());

                res.extend(file);
            }
            _ => res.extend(bincode::serde::encode_to_vec(self, config::standard())?),
        }

        Ok(res)
    }
}

fn read_string<R: Read>(reader: &mut R) -> anyhow::Result<String> {
    let count = reader.read_u32::<LittleEndian>()?;

    let mut bytes: Vec<u8> = vec![0u8; count as usize];
    reader.read_exact(&mut bytes)?;

    Ok(String::from_utf8(bytes)?)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPacket {
    FileList {
        dir: String,
        files: Vec<FileInfo>,
        folders: Vec<FolderInfo>,
    },
    Logs(Vec<Log>),
    FileUploaded {
        id: Uuid,
    },
}

impl ServerPacket {
    pub fn from_bin(slice: &[u8]) -> Result<ServerPacket, DecodeError> {
        bincode::serde::decode_from_slice::<ServerPacket, _>(slice, config::standard()).map(|v| v.0)
    }
    pub fn to_bin(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::serde::encode_to_vec(self, config::standard())
    }
}

#[derive(Debug)]
pub struct LogHolder {
    pub producers: HashSet<String>,
    pub server_logs: Vec<Log>,
    pub app_logs: Vec<Log>,

    pub producer_filter: String,
    pub(crate) max_log_level: LogLevel,
    pub level_filter: LogLevelFilter,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Display, EnumIter)]
#[repr(u8)]
pub enum LogLevelFilter {
    Debug,
    Info,
    Warning,
    Error,
    All = 255,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Log {
    pub level: LogLevel,
    pub producer: String,
    pub log: String,
    pub time: i64,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogHolder {
    pub const ALL: &'static str = "All";

    pub fn new() -> Self {
        let mut producers = HashSet::new();
        producers.insert(Self::ALL.to_string());

        Self {
            producers,
            server_logs: vec![],
            app_logs: vec![],
            producer_filter: LogHolder::ALL.to_string(),
            max_log_level: LogLevel::Info,
            level_filter: LogLevelFilter::Info,
        }
    }

    pub fn add_server(&mut self, logs: Vec<Log>) {
        for log in logs {
            self.max_log_level = self.max_log_level.max(log.level);

            if !self.producers.contains(&log.producer) {
                self.producers.insert(log.producer.clone());
            }

            self.server_logs.push(log);
        }
    }

    pub fn add_app(&mut self, log: Log) {
        self.app_logs.push(log);
    }
}
