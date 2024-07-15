use crate::backend::{Backend, BackendCommand, FrontendEvent};
use crate::frontend::left_block::LeftBlockScreen;
use crate::frontend::right_block::RightBlockScreen;
use std::sync::mpsc::Sender;

mod left_block;
pub(crate) mod right_block;
mod ui_kit;

pub struct Frontend {
    left_block_screen: LeftBlockScreen,
    right_block_screen: RightBlockScreen,

    to_backend: Sender<FrontendEvent>,
    pub(crate) backend: Backend,
}

impl Frontend {
    pub fn emit_event(&self, e: FrontendEvent) {
        self.to_backend.send(e).unwrap()
    }
    pub fn new(backend: Backend, to_backend: Sender<FrontendEvent>) -> Self {
        Self {
            left_block_screen: Default::default(),
            right_block_screen: Default::default(),
            to_backend,
            backend,
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let inner_rect = ui.min_rect();

            ui.separator();

            ui.horizontal(|ui| {
                ui.set_height(inner_rect.height());

                ui.separator();

                self.draw_left_block(ui);

                ui.separator();

                self.draw_right_block(ui);
            })
        });
    }

    pub fn on_update(&mut self) {
        for v in self.backend.on_update() {
            match v {
                BackendCommand::OpenFileObserve { dir } => {
                    self.right_block_screen = RightBlockScreen::Files { dir }
                }

                BackendCommand::ShowFileUploading { files } => {}
            }
        }
    }
}
