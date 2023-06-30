#![allow(unknown_lints)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![warn(unused_extern_crates)]
#![warn(unused_crate_dependencies)]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::path::PathBuf;

mod assets;
mod event_loop;
mod index_html;
mod startup;

#[derive(pretty_error_debug::Debug, thiserror::Error)]
pub enum MainError {
    #[error("could not initialize logger")]
    Log(#[source] log::SetLoggerError),
    #[error("could not create window")]
    Window(#[source] wry::application::error::OsError),
    #[error("could not open WebView")]
    Webview(#[source] wry::Error),
    #[error("execution exited with code {0}")]
    Exitcode(std::num::NonZeroI32),
    #[error("could not create temp dir")]
    Tempdir(#[source] std::io::Error),
    #[error("could not get cache dir")]
    Cachedir,
    #[error("could not change working directory")]
    ChangeWd(#[source] std::io::Error),
    #[error("error in event loop")]
    EventLoop(#[source] crate::event_loop::EvloopError),
    #[error("could not start exit-watchdog thread")]
    ExitWatchdog(#[source] std::io::Error),
}

#[derive(Debug)]
pub enum CustomEvent {
    ExitCtrlC,
    Open(PathBuf),
}

fn main() -> Result<(), MainError> {
    let result = startup::run();
    log::info!("Bye!");
    result
}
