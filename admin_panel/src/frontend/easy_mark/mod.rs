//! Experimental markup language

mod easy_mark_editor;
mod easy_mark_highlighter;
pub mod easy_mark_parser;
mod easy_mark_viewer;

pub use easy_mark_editor::EasyMarkEditor;
pub use easy_mark_highlighter::MemoizedEasymarkHighlighter;
pub use easy_mark_parser as parser;
pub use easy_mark_viewer::easy_mark;

/// Create a [`Hyperlink`](egui::Hyperlink) to this egui source code file on github.
#[macro_export]
macro_rules! egui_github_link_file {
    () => {
        $crate::egui_github_link_file!("(source code)")
    };
    ($label: expr) => {
        egui::github_link_file!(
            "https://github.com/emilk/egui/blob/master/",
            egui::RichText::new($label).small()
        )
    };
}

// ----------------------------------------------------------------------------

pub const DEFAULT_CODE: &str = r#"
# [MarkupLanguage](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax)
----------------

# At a glance
- inline text:
  - normal, `code`, **strong**, ~~strikethrough~~, *italics*, ***strong italics***
  - `\` escapes the next character
  - [hyperlink](https://github.com/emilk/egui)
  - Embedded URL: <https://github.com/emilk/egui>
- `# ` header
- `---` separator (horizontal line)
- `> ` quote
- `- ` bullet list
- `1. ` numbered list
- \`\`\` code fence
"#;
