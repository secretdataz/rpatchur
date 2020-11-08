#![windows_subsystem = "windows"]

mod patcher;
mod ui;

use std::io;

use patcher::{patcher_thread_routine, retrieve_patcher_configuration, PatcherCommand};
use simple_logger::SimpleLogger;
use tokio::{runtime, sync::mpsc};
use ui::{UIController, WebViewUserData};

fn main() {
    SimpleLogger::new()
        .init()
        .expect("Failed to initalize the logger");
    let mut tokio_rt = build_tokio_runtime().expect("Failed to build a tokio runtime");
    let config = match retrieve_patcher_configuration() {
        None => {
            log::error!("Failed to retrieve the patcher's configuration");
            ui::msg_box(
                "Error",
                "<b>Error:</b> Configuration file is invalid or doesn't exist.",
            );
            return;
        }
        Some(v) => v,
    };
    // Create a channel to allow the webview's thread to communicate with the patching thread
    let (tx, rx) = mpsc::channel::<PatcherCommand>(8);
    let webview = ui::build_webview("RPatchur", WebViewUserData::new(config.clone(), tx))
        .expect("Failed to build a web view");
    let patching_task = tokio_rt.spawn(patcher_thread_routine(
        UIController::new(&webview),
        config,
        rx,
    ));
    webview.run().unwrap();
    // Join the patching task from our synchronous function
    tokio_rt.block_on(async {
        if let Err(e) = patching_task.await {
            log::error!("Failed to join patching thread: {}", e);
        }
    });
}

/// Builds a tokio runtime with a threaded scheduler and a reactor
fn build_tokio_runtime() -> io::Result<runtime::Runtime> {
    runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
}
