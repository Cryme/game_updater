use strum::Display;
use uuid::Uuid;

#[derive(Display, Debug)]
pub enum FileUploadState {
    Uploading,
    Processing,
    Completed,
}

#[derive(Display, Debug)]
pub enum Notification {
    FileUpload {
        id: Uuid,
        dir: String,
        name: String,
        state: FileUploadState,
    },
}

impl PartialEq<Self> for Notification {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Notification::FileUpload { id, .. } => {
                let Notification::FileUpload { id: other_id, .. } = other;

                id == other_id
            }
        }
    }
}
