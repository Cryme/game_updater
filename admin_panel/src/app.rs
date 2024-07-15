use crate::backend::{Backend, FrontendEvent};
use crate::frontend::Frontend;
use std::sync::mpsc::Sender;

pub struct App {
    frontend: Frontend,
}

impl App {
    pub fn new(backend: Backend, to_backend: Sender<FrontendEvent>) -> Self {
        Self {
            frontend: Frontend::new(backend, to_backend),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frontend.on_update();

        self.frontend.draw(ctx);
    }
}
