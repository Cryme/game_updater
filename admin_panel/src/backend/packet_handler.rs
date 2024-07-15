use crate::backend::{Backend, BackendCommand};
use shared::admin_panel::ServerPacket;

impl Backend {
    pub(crate) fn handle_packets(&mut self) -> Vec<BackendCommand> {
        let mut res = vec![];
        while let Ok(v) = self.from_server.try_recv() {
            self.debug(&format!("{:?}", v));

            match v {
                ServerPacket::FileList {
                    dir,
                    files,
                    folders,
                } => {
                    self.file_info_holder.set(files, folders);
                    res.push(BackendCommand::OpenFileObserve { dir });
                }
            }
        }

        res
    }
}
