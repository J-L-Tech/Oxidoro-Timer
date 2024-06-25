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
        ProgramPhase::ReceiveInput,
        ProgramPhase::TimeFor { duration: 5 },
        ProgramPhase::TimeFor { duration: 5 },
        ProgramPhase::TimeFor { duration: 5 },
    ];
    let model: Arc<Mutex<TimerFSM>> = Arc::new(Mutex::new(TimerFSM::new(program, None)));
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

    // ui.on_request_increase_value({
    //     let ui_handle = ui.as_weak();
    //     move || {
    //         let ui = ui_handle.unwrap();
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });

    ui.run().unwrap();
}
