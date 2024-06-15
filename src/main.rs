slint::include_modules!();

fn decrement_clamp_and_report(min: u32, max: u32, val: u32) -> (bool, u32) {
    if val == min {
        return (true, max);
    }
    else if val > max {
        return (false, max);
    }
    else {
        return (false, val - 1);
    }
}

fn decrement_seconds(seconds: &mut u32, minutes: &mut u32) -> String {
    let (sec_overflow, n_seconds) = decrement_clamp_and_report(0, 59, *seconds);
    *seconds = n_seconds;
    if sec_overflow {
        let (min_overflow, n_minutes) = decrement_clamp_and_report(0, 59, *minutes);
        *minutes = n_minutes;
        if min_overflow {
            *seconds = 0;
            *minutes = 0;
            return format!("Timer Done");
        }
    }
    
    return format!("{:02}:{:02}", minutes, seconds);
}
    

#[cfg_attr(target_arch = "wasm32",
           wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn main() /* -> Result<(), slint::PlatformError>*/ {
    let ui = AppWindow::new().unwrap();

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    use slint::{Timer, TimerMode};
    let timer = Timer::default();
    let weak_ui = ui.as_weak();
    let mut minutes: u32 = 2;
    let mut seconds: u32 = 0;
    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(1000), move || {
        let strong_ui = weak_ui.upgrade().unwrap();
        strong_ui.invoke_request_increase_value();

        strong_ui.set_timer_string(decrement_seconds(&mut seconds, &mut minutes).into());
    });
    ui.run().unwrap();
}
