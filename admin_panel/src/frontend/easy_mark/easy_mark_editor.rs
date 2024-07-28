use crate::frontend::easy_mark::{easy_mark, MemoizedEasymarkHighlighter, DEFAULT_CODE};
use crate::frontend::ui_kit::{icon, UiKit, INFO_TOKEN};
use egui::{text::CCursorRange, *};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct EasyMarkEditor {
    pub code: String,
    pub edit_id: Option<u32>,

    highlight_editor: bool,
    show_rendered: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    highlighter: MemoizedEasymarkHighlighter,
}

impl PartialEq for EasyMarkEditor {
    fn eq(&self, other: &Self) -> bool {
        (&self.code, self.highlight_editor, self.show_rendered)
            == (&other.code, other.highlight_editor, other.show_rendered)
    }
}

impl Default for EasyMarkEditor {
    fn default() -> Self {
        Self {
            code: DEFAULT_CODE.trim().to_owned(),
            edit_id: None,
            highlight_editor: false,
            show_rendered: true,
            highlighter: Default::default(),
        }
    }
}

impl EasyMarkEditor {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Response {
        let resp = ui
            .horizontal(|ui| {
                ui.add(Label::new(icon(INFO_TOKEN)).selectable(false))
                    .on_hover_ui(nested_hotkeys_ui)
                    .on_hover_cursor(CursorIcon::Help);

                ui.button_s("Save", 0., 0.)
            })
            .inner;

        ui.separator();

        if self.show_rendered {
            ui.columns(2, |columns| {
                ScrollArea::vertical()
                    .id_source("source")
                    .show(&mut columns[0], |ui| self.editor_ui(ui));
                ScrollArea::vertical()
                    .id_source("rendered")
                    .show(&mut columns[1], |ui| {
                        // TODO(emilk): we can save some more CPU by caching the rendered output.
                        easy_mark(ui, &self.code);
                    });
            });
        } else {
            ScrollArea::vertical()
                .id_source("source")
                .show(ui, |ui| self.editor_ui(ui));
        }

        resp
    }

    fn editor_ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            code, highlighter, ..
        } = self;

        let response = if self.highlight_editor {
            let mut layouter = |ui: &egui::Ui, easymark: &str, wrap_width: f32| {
                let mut layout_job = highlighter.highlight(ui.style(), easymark);
                layout_job.wrap.max_width = wrap_width;
                ui.fonts(|f| f.layout_job(layout_job))
            };

            ui.add(
                egui::TextEdit::multiline(code)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .layouter(&mut layouter),
            )
        } else {
            ui.add(egui::TextEdit::multiline(code).desired_width(f32::INFINITY))
        };

        if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id) {
            if let Some(mut ccursor_range) = state.cursor.char_range() {
                let any_change = shortcuts(ui, code, &mut ccursor_range);
                if any_change {
                    state.cursor.set_char_range(Some(ccursor_range));
                    state.store(ui.ctx(), response.id);
                }
            }
        }
    }
}

pub const SHORTCUT_BOLD: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::B);
pub const SHORTCUT_CODE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
pub const SHORTCUT_ITALICS: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::I);
pub const SHORTCUT_STRIKETHROUGH: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::Q);
pub const SHORTCUT_INDENT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::E);

fn nested_hotkeys_ui(ui: &mut egui::Ui) {
    egui::Grid::new("shortcuts").striped(true).show(ui, |ui| {
        let mut label = |shortcut, what| {
            ui.label(what);
            ui.weak(ui.ctx().format_shortcut(&shortcut));
            ui.end_row();
        };

        label(SHORTCUT_BOLD, "**bold**");
        label(SHORTCUT_CODE, "`code`");
        label(SHORTCUT_ITALICS, "*italics*");
        label(SHORTCUT_STRIKETHROUGH, "~~strikethrough~~");
        label(SHORTCUT_INDENT, "two spaces"); // Placeholder for tab indent
    });
}

fn shortcuts(ui: &Ui, code: &mut dyn TextBuffer, ccursor_range: &mut CCursorRange) -> bool {
    let mut any_change = false;

    if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_INDENT)) {
        // This is a placeholder till we can indent the active line
        any_change = true;
        let [primary, _secondary] = ccursor_range.sorted();

        let advance = code.insert_text("  ", primary.index);
        ccursor_range.primary.index += advance;
        ccursor_range.secondary.index += advance;
    }

    for (shortcut, surrounding) in [
        (SHORTCUT_BOLD, "**"),
        (SHORTCUT_CODE, "`"),
        (SHORTCUT_ITALICS, "*"),
        (SHORTCUT_STRIKETHROUGH, "~~"),
    ] {
        if ui.input_mut(|i| i.consume_shortcut(&shortcut)) {
            any_change = true;
            toggle_surrounding(code, ccursor_range, surrounding);
        };
    }

    any_change
}

/// E.g. toggle *strong* with `toggle_surrounding(&mut text, &mut cursor, "*")`
fn toggle_surrounding(
    code: &mut dyn TextBuffer,
    ccursor_range: &mut CCursorRange,
    surrounding: &str,
) {
    let [primary, secondary] = ccursor_range.sorted();

    let surrounding_ccount = surrounding.chars().count();

    let prefix_crange = primary.index.saturating_sub(surrounding_ccount)..primary.index;
    let suffix_crange = secondary.index..secondary.index.saturating_add(surrounding_ccount);
    let already_surrounded = code.char_range(prefix_crange.clone()) == surrounding
        && code.char_range(suffix_crange.clone()) == surrounding;

    if already_surrounded {
        code.delete_char_range(suffix_crange);
        code.delete_char_range(prefix_crange);
        ccursor_range.primary.index -= surrounding_ccount;
        ccursor_range.secondary.index -= surrounding_ccount;
    } else {
        code.insert_text(surrounding, secondary.index);
        let advance = code.insert_text(surrounding, primary.index);

        ccursor_range.primary.index += advance;
        ccursor_range.secondary.index += advance;
    }
}
