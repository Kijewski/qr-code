use std::process::abort;
use std::sync::atomic;

use wry::application::event::{Event, StartCause, WindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoopProxy};

use crate::startup::ExitWatchdog;
use crate::CustomEvent;

#[derive(Debug, thiserror::Error)]
pub enum EvloopError {}

fn setup_ctrl_c(proxy: &EventLoopProxy<CustomEvent>, exit_watchdog: &ExitWatchdog) {
    let proxy = proxy.clone();
    let exit_watchdog = exit_watchdog.clone();
    let once = atomic::AtomicUsize::new(0);

    ctrlc::set_handler(move || {
        let count = once.fetch_add(1, atomic::Ordering::SeqCst);
        log::warn!("Ctrl+C trapped (repeat #{count})");
        match count {
            0 => {
                exit_watchdog.trip();
                let _: Result<_, _> = proxy.send_event(CustomEvent::ExitCtrlC);
            },
            1 => panic!("Told to shut down twice."),
            _ => abort(),
        }
    })
    .expect("could not set ctrl+c handler");
}

pub fn handle_evloop_event(
    control_flow: &mut ControlFlow,
    event: Event<'_, CustomEvent>,
    proxy: &EventLoopProxy<CustomEvent>,
    exit_watchdog: &ExitWatchdog,
) -> Result<(), EvloopError> {
    *control_flow = ControlFlow::Wait;
    match event {
        Event::NewEvents(StartCause::Init) => {
            setup_ctrl_c(&proxy, &exit_watchdog);
        },
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                exit_watchdog.trip();
                *control_flow = ControlFlow::ExitWithCode(0);
            },
            _ => (),
        },
        _ => (),
    }
    Ok(())
}
