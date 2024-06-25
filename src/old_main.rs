slint::include_modules!();

mod timer_util;
mod ui_util;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use timer_util::*;
use ui_util::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn main() /* -> Result<(), slint::PlatformError>*/
{
    let ui = AppWindow::new().unwrap();

    let program: Vec<ProgramPhase> = vec![
        // TAA
        ProgramPhase::ReceiveInput,
        ProgramPhase::TimeFor { duration: 10 },
        ProgramPhase::OffsetVariable {
            var_index: 0,
            offset: -1,
        },
        ProgramPhase::Repeat {
            to_phase: 0,
            var_index: 0,
        },
        // SKtC
        ProgramPhase::ReceiveInput,
        ProgramPhase::TimeFor { duration: 20 },
        ProgramPhase::OffsetVariable {
            var_index: 1,
            offset: -1,
        },
        ProgramPhase::Repeat {
            to_phase: 4,
            var_index: 1,
        },
        // LTR
        ProgramPhase::ReceiveInput,
        ProgramPhase::TimeFor { duration: 20 },
        ProgramPhase::OffsetVariable {
            var_index: 2,
            offset: -1,
        },
        ProgramPhase::Repeat {
            to_phase: 8,
            var_index: 2,
        },
        // SLB
        ProgramPhase::ReceiveInput,
        ProgramPhase::TimeFor { duration: 10 },
        ProgramPhase::OffsetVariable {
            var_index: 3,
            offset: -1,
        },
        ProgramPhase::Repeat {
            to_phase: 12,
            var_index: 3,
        },
        // DB
        ProgramPhase::ReceiveInput,
        ProgramPhase::TimeFor { duration: 5 },
        ProgramPhase::OffsetVariable {
            var_index: 4,
            offset: -1,
        },
        ProgramPhase::Repeat {
            to_phase: 16,
            var_index: 4,
        },
        // B
        // Set 1
        ProgramPhase::ReceiveInput,
        ProgramPhase::OffsetVariable {
            var_index: 5,
            offset: -1,
        },
        ProgramPhase::Repeat {
            to_phase: 10,
            var_index: 5,
        },
        // Set 2
        ProgramPhase::ReceiveInput,
        ProgramPhase::OffsetVariable {
            var_index: 6,
            offset: -1,
        },
        ProgramPhase::Repeat {
            to_phase: 10,
            var_index: 6,
        },
    ];
    let variables: Vec<i8> = vec![10, 8, 8, 10, 16, 10, 10];
    let model: Arc<Mutex<TimerFSM>> =
        Arc::new(Mutex::new(TimerFSM::new(program, variables.into())));
    let timer: Arc<Mutex<slint::Timer>> = Arc::new(Mutex::new(slint::Timer::default()));

    ui.on_play_sound({
        || {
            let _ = ui_util::play_sound(None); // TODO Error Handling
        }
    });

    ui.on_button_pressed({
        let ui_handle = ui.as_weak();
        let model_handle: Arc<Mutex<TimerFSM>> = model.clone();
        move |input| {
            data_to_ui(model_handle.lock().unwrap().input(input), &ui_handle);
        }
    });

    ui.on_step({
        let ui_handle = ui.as_weak();
        let model_handle = model.clone();
        move || {
            data_to_ui(
                model_handle.lock().unwrap().input(TimerInput::Step),
                &ui_handle,
            );
        }
    });

    ui.on_start_timer({
        let ui_handle = ui.as_weak();
        let timer_handle: Arc<Mutex<slint::Timer>> = timer.clone();
        move || {
            let ui = ui_handle.unwrap();
            let timer = timer_handle.clone();
            timer.lock().unwrap().start(
                slint::TimerMode::Repeated,
                Duration::new(1, 0),
                move || {
                    ui.invoke_step();
                },
            );
        }
    });

    ui.on_stop_timer({
        let timer_handle: Arc<Mutex<slint::Timer>> = timer.clone();
        move || {
            let timer = timer_handle.clone();
            timer.lock().unwrap().stop();
        }
    });

    ui.run().unwrap();
}
