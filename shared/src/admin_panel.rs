use bincode::config;
use bincode::error::{DecodeError, EncodeError};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Cursor, Read};

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
                let mut file = vec![];
                reader.read_to_end(&mut file)?;

                Ok(ClientPacket::AddFile { dir, name, file })
            }
            _ => {
                unimplemented!()
            }
        }
    }
    pub fn to_bin(&self) -> Result<Vec<u8>, EncodeError> {
        let mut res = vec![0x0u8];
        match self {
            ClientPacket::AddFile { dir, name, file } => {
                res[0] = 0x1;

                let b = dir.as_bytes();
                res.extend(b.len().to_le_bytes());
                res.extend(b);

                let b = name.as_bytes();
                res.extend(b.len().to_le_bytes());
                res.extend(b);

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
}

impl ServerPacket {
    pub fn from_bin(slice: &[u8]) -> Result<ServerPacket, DecodeError> {
        bincode::serde::decode_from_slice::<ServerPacket, _>(slice, config::standard()).map(|v| v.0)
    }
    pub fn to_bin(&self) -> Result<Vec<u8>, EncodeError> {
        bincode::serde::encode_to_vec(self, config::standard())
    }
}
