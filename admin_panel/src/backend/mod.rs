use crate::backend::file_info_holder::FileInfoHolder;
use crate::backend::network::Network;
use crate::backend::notification::Notification;
use crate::backend::patch_note::PatchNoteHolder;
use log::{log, Level};
use shared::admin_panel::{ClientPacket, Log, LogHolder, LogLevel, ServerPacket};
use std::sync::mpsc::{channel, Receiver};

pub(crate) mod events;
pub(crate) mod file_info_holder;
pub(crate) mod network;
pub(crate) mod notification;
mod packet_handler;
mod patch_note;

pub enum BackendCommand {
    OpenFileObserve { dir: String },
    OpenLogs,
    OpenPatchNotes,
    OpenPatchNote { id: Option<u32>, data: String },
}

pub enum FrontendEvent {
    CreateFolder {
        dir: String,
        name: String,
    },
    RemoveFolder {
        dir: String,
        name: String,
    },
    UploadFiles {
        dir: String,
        files: Vec<(String, Vec<u8>)>,
    },
    RemoveFile {
        dir: String,
        name: String,
    },
    SkipFileHashCheck {
        dir: String,
        name: String,
    },
    RequestOpenScreen(Screen),
    SavePatchNote {
        id: Option<u32>,
        data: String,
    },
    DeletePatchNote {
        id: u32,
    },
}

#[derive(Default, Eq, PartialEq)]
pub enum Screen {
    #[default]
    Dashboard,
    PatchNotes,
    EditPatchNote {
        id: Option<u32>,
    },
    Files {
        dir: String,
    },
    Logs,
}

pub struct Backend {
    network: Network,
    from_server: Receiver<ServerPacket>,
    from_frontend: Receiver<FrontendEvent>,

    pub(crate) notifications: Vec<Notification>,

    pub(crate) log_holder: LogHolder,
    pub(crate) patch_note_holder: PatchNoteHolder,
    pub(crate) file_info_holder: FileInfoHolder,
}

impl Backend {
    pub fn new(frontend_rx: Receiver<FrontendEvent>) -> Self {
        let (sender, receiver) = channel();
        let mut network = Network::init(sender);

        network.run();

        Self {
            network,
            log_holder: LogHolder::new(),
            patch_note_holder: PatchNoteHolder::default(),
            from_server: receiver,
            from_frontend: frontend_rx,
            file_info_holder: FileInfoHolder::default(),
            notifications: vec![],
        }
    }

    fn send_packet(&self, packet: ClientPacket) {
        self.network.send_packet(packet);
    }

    pub fn debug(&mut self, text: &str) {
        log!(Level::Debug, "{}", text);

        self.log_holder.add_app(Log {
            level: LogLevel::Debug,
            producer: "App".to_string(),
            log: text.to_string(),
            time: chrono::Local::now().timestamp(),
        })
    }

    pub fn on_update(&mut self) -> Vec<BackendCommand> {
        let mut res = self.handle_packets();
        res.append(&mut self.handle_frontend_events());

        res
    }
}
