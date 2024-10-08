mod app;
mod backend;
mod frontend;

use crate::app::App;
use crate::backend::Backend;

use crate::frontend::setup_custom_fonts;
pub use eframe::{WebLogger, WebOptions, WebRunner};
use gloo_timers::future::TimeoutFuture;
use std::sync::mpsc::channel;
use wasm_bindgen_futures::spawn_local;

const WS_SERVER: &str = "ws://127.0.0.1:3000/ws";

/**
TODO:
 - Real work with files
*/

fn main() {
    WebLogger::init(log::LevelFilter::Debug).ok();
    console_error_panic_hook::set_once();

    let web_options = WebOptions::default();

    spawn_local(async {
        let start_result = WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| {
                    setup_custom_fonts(&cc.egui_ctx);

                    let ctx = cc.egui_ctx.clone();

                    spawn_local(async move {
                        loop {
                            ctx.request_repaint();

                            TimeoutFuture::new(1_000).await;
                        }
                    });

                    let (sender, receiver) = channel();
                    let mut backend = Backend::new(receiver);

                    backend.debug("Started...");

                    Ok(Box::new(App::new(backend, sender)))
                }),
            )
            .await;

        // Remove the loading text and spinner:
        let loading_text = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"));

        if let Some(loading_text) = loading_text {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
