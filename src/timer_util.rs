use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};
use slint::Timer;
use std::fs::File;
use std::io::BufReader;

// TODO The Borrow Checker Likely Requires a setup more like this:
// State Transition Scheme that holds:
// an Arc Mutex to the TimerModel Data
// a Slint Timer that calls the step function using that Arc<Mutex>

enum ViewDataUpdate {
    ErrorOccurred(anyhow::Error),
    ProgramDone,
    CurrentTime(u32),
    WaitingForInput,
    Paused,
}

enum ProgramPhases {
    /* Set The Timer To Run for the given number of seconds */
    TimeFor { seconds: u32 },

    /* Wait for User Interaction before Continuing to the Next Phase */
    WaitForInput,

    /* step: Where in the program to repeat from, step must be an earlier phase */
    /* times: How Many Times to Repeat */
    RepeatProgram { step: usize, times: usize },

    /* For Infinite Loops, Place Only At the End of a Program */
    Loop { step: usize },
}

#[derive(Debug, Clone, Copy)]
enum TimerState {
    Idle,
    Running,
    Paused,
}

pub enum TimerTransitions {
    Interact,
    Start,
    Step,
    Stop,
    Pause,
    Unpause,
    Wait,
}

pub struct TimerModel {
    state: TimerState,
    seconds: Option<u32>,
    repetitions: Option<usize>,
    //timer: slint::Timer,
    timer_sound: String,
    report_callback: fn(ViewDataUpdate),
    phase: usize,
    timer_program: Vec<ProgramPhases>,
}

impl TimerModel {
    fn new(
        timer_sound: &str,
        report_callback: fn(ViewDataUpdate),
        timer_program: Vec<ProgramPhases>,
    ) -> TimerModel {
        TimerModel {
            state: TimerState::Idle,
            seconds: None,
            repetitions: None,
            //timer: Timer::default(),
            timer_sound: timer_sound.to_string(),
            report_callback: report_callback,
            phase: 0,
            timer_program: timer_program,
        }
    }

    fn next(&mut self, transition: TimerTransitions) {
        self.state = match (self.state, transition) {
            (_, TimerTransitions::Stop) => {
                //self.timer.stop();
                TimerState::Idle
            },
            (TimerState::Idle, TimerTransitions::Start) => self.next_phase(),
            (TimerState::Idle, _) => self.state,
            (TimerState::Running, TimerTransitions::Step) => {
                match self.seconds {
                    Some(seconds) => {
                        if seconds != 0 {
                            self.seconds = Some(seconds - 1);
                        }
                        else {
                            self.seconds = None;    
                        }
                        self.state
                    },
                    None => self.next_phase(),
                }
            },
            (TimerState::Running, TimerTransitions::Pause) => {
                (self.report_callback)(ViewDataUpdate::Paused);
                TimerState::Paused
            },
            (TimerState::Running, _) => self.state,
            (TimerState::Paused, TimerTransitions::Unpause) => {
                
                TimerState::Running
            },
            (TimerState::Paused, _) => self.state,
        };
    }

    fn next_phase(&mut self) -> TimerState {
        if self.phase >= self.timer_program.len() {
            match play_sound(Some(&self.timer_sound)) {
                Ok((sink, _out_stream)) => sink.sleep_until_end() /* TODO Maybe Put this in it's own thread */,
                Err(e) => (self.report_callback)(ViewDataUpdate::ErrorOccurred(e))
            }
            (self.report_callback)(ViewDataUpdate::ProgramDone);
            return TimerState::Idle;
        }
        self.phase += 1;
        match self.timer_program[self.phase] {
            ProgramPhases::TimeFor { seconds } => {
                self.seconds = Some(seconds);
                (self.report_callback)(ViewDataUpdate::CurrentTime(seconds));
                return TimerState::Running;
            }
            ProgramPhases::WaitForInput => {
                (self.report_callback)(ViewDataUpdate::WaitingForInput);
                return TimerState::Running;
            }
            /* May Want to Report this to the UI */
            ProgramPhases::RepeatProgram { step, times } => {
                match self.repetitions {
                    Some(repeats) => {
                        if repeats != 0 {
                            self.repetitions = Some(repeats - 1);
                        } else {
                            self.repetitions = None;
                        }
                    }
                    None => {
                        self.repetitions = Some(times - 1);
                        self.phase = step - 1;
                    }
                }
                return self.next_phase();
            }
            ProgramPhases::Loop { step } => {
                self.phase = step - 1;
                return self.next_phase();
            }
        }
    }
}

fn play_sound(path: Option<&str>) -> Result<(Sink, OutputStream)> {
    let audio_path: &str = path.unwrap_or("assets/Timer-Done-Sound.mp3");

    // _stream must live as long as the sink
    let (stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let file = BufReader::new(File::open(audio_path)?);

    // Decode that sound file into a source
    let source = Decoder::new(file)?;
    sink.append(source);

    // The sound plays in a separate thread. This call will block the current thread until the sink
    // has finished playing all its queued sounds.
    //sink.sleep_until_end();
    return Ok((sink, stream));
}

#[cfg(test)]
mod timer_util_tests {

    use super::play_sound;
    use std::panic;

    #[test]
    fn play_default_sound() {
        match play_sound(None) {
            Ok((sink, _stream)) => sink.sleep_until_end(),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn play_specified_sound() {
        match play_sound(Some("assets/Timer-Done-Sound.mp3")) {
            Ok((sink, _stream)) => sink.sleep_until_end(),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn err_on_invalid_specified_sound() {
        match play_sound(Some("")) {
            Ok((_sink, _stream)) => panic!("Should Err, Instead was Ok"),
            Err(_) => (),
        }
    }
}
