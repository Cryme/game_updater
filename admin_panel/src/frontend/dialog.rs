use crate::frontend::Frontend;

#[derive(Eq, PartialEq)]
pub enum Dialog {
    None,
    CreateFolder { dir: String, name: String },
}

impl Frontend {
    pub fn show_dialog(&mut self, dialog: Dialog) {
        self.dialog = dialog;
    }
}
