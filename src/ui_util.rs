use crate::AppWindow;
use crate::ProgramPhase;
use crate::TimerOutput;

use anyhow::Result;

pub fn data_to_ui(output: TimerOutput, ui_handle: &slint::Weak<AppWindow>) {
    let ui = ui_handle.unwrap();
    match output {
        TimerOutput::NoChange => {}
        TimerOutput::ProgramStopped { program_phase } => {
            match program_phase {
                ProgramPhase::TimeFor { duration: _ } => {
                    ui.invoke_stop_timer();
                    ui.set_timer_string("Stopped".into());
                }
                _ => {}
            }
            let _ = play_sound("assets/Program-Stopped-Sound.mp3".into());
            ui.set_timer_string("Ready to Start".into()); // TODO Probably want a Reset UI function to invoke
        },
        TimerOutput::TimerProgress { seconds } => {
            ui.set_timer_string(seconds_to_h_m_s_display_string(seconds).into());
            if seconds == 0 {
                let _ = play_sound("assets/Timer-Done-Sound.mp3".into()); // TODO Error Handling
            }
        }
        TimerOutput::TimerPaused => {
            let _ = play_sound("assets/Pause-Sound.mp3".into());
            ui.set_timer_string(format!("{} ||", ui.get_timer_string().to_string()).into());
        }
        TimerOutput::TimerResumed { seconds } => {
            ui.invoke_start_timer();
            ui.set_timer_string(seconds_to_h_m_s_display_string(seconds).into());
            let _ = play_sound("assets/Resume-Sound.mp3".into()); // TODO Error Handling
        }
        TimerOutput::TimerReset { seconds } => {
            ui.set_timer_string(seconds_to_h_m_s_display_string(seconds).into());
            let _ = play_sound("assets/Reset-Sound.mp3".into()); // TODO Error Handling
        },
        TimerOutput::PhaseChange { prev_phase, next_phase, phase_completed } => {
            match prev_phase {
                ProgramPhase::BeginProgram => {
                    let _ = play_sound("assets/Program-Start-Sound.mp3".into()); // TODO Error Handling
                },
                ProgramPhase::TimeFor { duration: _ } => {
                    if phase_completed {
                        
                    } else {
                        let _ = play_sound("assets/Skip-Sound.mp3".into()); // TODO Error Handling
                    }
                    
                },
                ProgramPhase::EndProgram => {},
                ProgramPhase::ReceiveInput => {
                    let _ = play_sound("assets/Timer-Done-Sound.mp3".into());
                },
                _ => {},
            }
            match next_phase {
                ProgramPhase::TimeFor { duration } => {
                    ui.invoke_start_timer();
                    ui.set_timer_string(seconds_to_h_m_s_display_string(duration).into());
                },
                ProgramPhase::EndProgram => {
                    let _ = play_sound("assets/Program-Done-Sound.mp3".into()); // TODO Error Handling
                    ui.set_timer_string("Ready to Start".into());
                },
                ProgramPhase::BeginProgram => {
                    let _ = play_sound("assets/Program-Done-Sound.mp3".into());
                },
                ProgramPhase::ReceiveInput => {
                    ui.set_timer_string("Input".into());
                },
                _ => {},
            }
        },
    }
}

fn seconds_to_h_m_s_display_string(total_seconds: usize) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds - hours * 3600) / 60;
    let seconds = total_seconds - hours * 3600 - minutes * 60;
    return format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
}

pub fn play_sound(path: Option<&str>) -> Result<()> {
    let audio_path: &str = path.unwrap_or("assets/Timer-Done-Sound.mp3");
    let result = web_sys::HtmlAudioElement::new_with_src(audio_path);
    let _ = result.unwrap().play();

    return Ok(());
}

#[cfg(test)]
mod ui_util_tests {

    use super::seconds_to_h_m_s_display_string;
    // use std::panic;

    #[test]
    fn just_seconds() {
        assert_eq!("00:00:15", seconds_to_h_m_s_display_string(15));
    }

    #[test]
    fn just_minutes() {
        assert_eq!("00:15:00", seconds_to_h_m_s_display_string(15 * 60));
    }

    #[test]
    fn just_hours() {
        assert_eq!("15:00:00", seconds_to_h_m_s_display_string(15 * 3600));
    }

    #[test]
    fn minute_threshold() {
        assert_eq!("00:01:00", seconds_to_h_m_s_display_string(60));
    }

    
    #[test]
    fn hour_threshold() {
        assert_eq!("01:00:00", seconds_to_h_m_s_display_string(3600));
    }
}
