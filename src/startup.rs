use std::borrow::Cow;
use std::env::{current_exe, set_current_dir, var};
use std::fs::create_dir_all;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::abort;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use dirs::cache_dir;
use parking_lot::Mutex;
use pretty_env_logger::formatted_timed_builder;
use tinyfiledialogs::save_file_dialog_with_filter;
use wry::application::dpi::{LogicalSize, Size};
use wry::application::event_loop::{ControlFlow, EventLoop};
use wry::application::platform::run_return::EventLoopExtRunReturn;
use wry::application::window::{Theme, WindowBuilder};
use wry::webview::{WebContext, WebViewBuilder};

use crate::assets::asset_handler;
use crate::event_loop::handle_evloop_event;
use crate::{CustomEvent, MainError};

pub(crate) fn run() -> Result<(), MainError> {
    setup_logger()?;
    change_working_directory().map_err(MainError::ChangeWd)?;
    let exit_watchdog = setup_exit_watchdog()?;

    let result = run_window(exit_watchdog.clone());
    log::info!("Shutting down ...");

    exit_watchdog.trip();

    result
}

fn setup_exit_watchdog() -> Result<ExitWatchdog, MainError> {
    let (tx, rx) = channel();
    thread::Builder::new()
        .name("exit-watchdog".to_owned())
        .spawn(move || {
            let _: Result<_, _> = rx.recv();
            log::warn!("Awaiting graceful exit. Aborting application in 10 seconds!");
            thread::sleep(Duration::from_secs(10));
            abort();
        })
        .map_err(MainError::ExitWatchdog)?;
    Ok(ExitWatchdog::new(tx))
}

#[derive(Debug, Clone)]
pub struct ExitWatchdog(Arc<Mutex<Option<Sender<()>>>>);

impl ExitWatchdog {
    fn new(tx: Sender<()>) -> Self {
        Self(Arc::new(Mutex::new(Some(tx))))
    }

    pub fn trip(&self) {
        let tx = self
            .0
            .try_lock_for(Duration::from_millis(4))
            .expect("could not lock exit-watchdog's channel")
            .take();
        if let Some(tx) = tx {
            tx.send(())
                .expect("could not write to exit-watchdog's channel");
        }
    }
}

fn setup_logger() -> Result<(), MainError> {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::System::Console;
        let _ = Console::AttachConsole(Console::ATTACH_PARENT_PROCESS);
    }

    let mut builder = formatted_timed_builder();
    builder.parse_filters(&var("RUST_LOG").map_or(Cow::Borrowed("info"), Cow::Owned));
    builder.parse_filters(&var("RUST_LOG_STYLE").map_or(Cow::Borrowed("always"), Cow::Owned));
    builder.format_timestamp_nanos();
    builder.format_indent(Some(4));
    builder.format(format_record);
    builder.try_init().map_err(MainError::Log)?;
    Ok(())
}

fn format_record(
    buf: &mut pretty_env_logger::env_logger::fmt::Formatter,
    record: &log::Record<'_>,
) -> Result<(), std::io::Error> {
    let (Some(file), Some(line)) = (record.file(), record.line()) else { return Ok(()) };
    let mut msg = Vec::with_capacity(80);
    write!(
        msg,
        "[{level}] [{now}] [{file:?}:{line}]: {msg}\r\n",
        level = record.level(),
        now = tz::UtcDateTime::now().expect("could not even get current time"),
        msg = record.args(),
    )?;
    buf.write_all(&msg)
}

#[cfg(not(target_os = "windows"))]
fn change_working_directory() -> std::io::Result<()> {
    let exe_path = current_exe()?.canonicalize()?;
    if let Some(parent) = exe_path.parent() {
        set_current_dir(parent)
    } else {
        set_current_dir("/")
    }
}

#[cfg(target_os = "windows")]
fn change_working_directory() -> std::io::Result<()> {
    let exe_path = current_exe()?.canonicalize()?;
    let parent = if let Some(parent) = exe_path.parent() {
        parent.as_os_str()
    } else {
        return Ok(());
    };

    if parent.to_string_lossy().starts_with(r"\\") {
        set_current_dir(parent)
    } else {
        let mut unc_path = std::ffi::OsString::default();
        unc_path.reserve_exact(4 + parent.len() + 1);
        unc_path.push(r"\\?\");
        unc_path.push(parent);
        set_current_dir(unc_path)
    }
}

fn run_window(exit_watchdog: ExitWatchdog) -> Result<(), MainError> {
    let main_err = Mutex::new(None);

    let tempdir = new_tempdir()?;
    let mut event_loop = EventLoop::<CustomEvent>::with_user_event();
    let mut ctx = WebContext::new(Some(tempdir.path().to_owned()));
    let size = Size::Logical(LogicalSize::new(800.0, 440.0));
    let window = WindowBuilder::new()
        .with_theme(Some(Theme::Light))
        .with_title("QR-Code-Generierer")
        .with_min_inner_size(size)
        .with_inner_size(size)
        .build(&event_loop)
        .map_err(MainError::Window)?;
    let webview = WebViewBuilder::new(window)
        .map_err(MainError::Webview)?
        .with_web_context(&mut ctx)
        .with_background_color((0x11, 0x11, 0x11, 0xff))
        .with_accept_first_mouse(true)
        .with_autoplay(true)
        .with_download_started_handler(|src, dest| {
            let Some((name, _)) = src.split_once('?') else { return false };
            let Some((_, name)) = name.rsplit_once('/') else { return false };
            let Some((_, ext)) = name.rsplit_once('.') else { return false };
            let Some(mut download_path) = dirs::download_dir() else { return false };
            download_path.push(name);
            let Some(download_path_str) = download_path.to_str() else { return false };

            let Some(path) = save_file_dialog_with_filter(
                "Speichern unter",
                download_path_str,
                &[&format!("*.{ext}")],
                "",
            ) else { return false };

            *dest = PathBuf::from(path);
            true
        })
        .with_download_completed_handler(|_, path, success| {
            let (Some(path), true) = (path, success) else { return };
            if let Err(err) = open::that_detached(&path) {
                log::error!("could not open {path:?}: {err}");
            }
        })
        .with_custom_protocol("wry".to_owned(), asset_handler)
        .with_url("wry://app.local/index.html")
        .unwrap()
        .build()
        .map_err(MainError::Webview)?;

    let webview = Mutex::new(Some(webview));
    let exitcode = event_loop.run_return({
        let proxy = event_loop.create_proxy();
        let webview = &webview;
        let main_err = &main_err;
        move |event, _, control_flow| {
            let guard = webview.lock();
            let Some(webview) = &*guard else { return };
            if let Err(err) = handle_evloop_event(control_flow, event, &proxy, &exit_watchdog) {
                exit_watchdog.trip();
                webview.window().set_visible(false);
                *main_err.lock() = Some(MainError::EventLoop(err));
                *control_flow = ControlFlow::ExitWithCode(1);
            }
        }
    });

    let webview = webview.lock().take().unwrap();
    let _: Result<(), wry::Error> = webview.clear_all_browsing_data();
    drop(webview);
    drop(event_loop);
    thread::sleep(Duration::from_millis(2_000));
    if let Err(err) = tempdir.close() {
        log::error!("could not remove tempdir: {err}");
    };

    if let Some(err) = main_err.lock().take() {
        return Err(err);
    }
    match exitcode.try_into() {
        Ok(exitcode) => Err(MainError::Exitcode(exitcode)),
        Err(_) => Ok(()),
    }
}

fn new_tempdir() -> Result<tempfile::TempDir, MainError> {
    let mut cachedir = cache_dir().ok_or_else(|| MainError::Cachedir)?;
    #[cfg(target_os = "windows")]
    {
        let parent = cachedir.as_os_str().to_str().ok_or(MainError::Cachedir)?;
        if !parent.starts_with(r"\\") {
            let mut unc_path = std::ffi::OsString::default();
            unc_path.reserve_exact(4 + parent.len() + 1 + 6 + 1 + 7 + 1);
            unc_path.push(r"\\?\");
            unc_path.push(parent);
            cachedir = std::path::PathBuf::from(unc_path);
        }
    }
    cachedir.push("k6i.de");
    cachedir.push("qr-code");

    create_dir_all(&cachedir).map_err(MainError::Tempdir)?;

    let tempdir = tempfile::Builder::new()
        .prefix(&format!("wry-temp-{}-", std::process::id()))
        .tempdir_in(cachedir)
        .map_err(MainError::Tempdir)?;
    log::info!("Tempdir: {:?}", tempdir.path());
    Ok(tempdir)
}
