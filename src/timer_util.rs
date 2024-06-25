use crate::TimerInput;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerOutput {
    NoChange,
    ProgramStopped { program_phase: ProgramPhase },
    PhaseChange {prev_phase: ProgramPhase, next_phase: ProgramPhase, phase_completed: bool},
    TimerProgress { seconds: usize },
    TimerPaused,
    TimerReset {seconds: usize},
    TimerResumed { seconds: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgramPhase {
    BeginProgram,
    TimeFor { duration: usize },
    ReceiveInput,
    Repeat {to_phase: usize, var_index: usize},
    OffsetVariable{var_index: usize, offset: i8},
    EndProgram,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TimerState {
    Idle,
    Timer {
        progress: usize,
        duration: usize,
        paused: bool,
    },
    Input,
}

pub struct TimerFSM {
    program: Vec<ProgramPhase>,
    variables: Vec<i8>,
    state: TimerState,
    phase: usize,
}

/**
 * @return the State the Timer Should Be In After Transitioning to the Given Phase
 */
fn phase_transition(phase: &mut usize, state: &mut TimerState, program: &Vec<ProgramPhase>) {
    return match program.get(*phase) {
        Some(program_phase) => {
            match program_phase {
                ProgramPhase::TimeFor { duration } => {
                    *state = TimerState::Timer {
                        progress: *duration,
                        duration: *duration,
                        paused: false,
                    }
                }
                ProgramPhase::BeginProgram | ProgramPhase::EndProgram => {
                    *state = TimerState::Idle; 
                    *phase = 0;
                },
                ProgramPhase::ReceiveInput => {
                    *state = TimerState::Input;
                },
                ProgramPhase::Repeat { to_phase, var_index } => {
                    todo!(); // Check if variable is zero
                    *phase = *to_phase;
                    phase_transition(phase, state, program);
                },
                ProgramPhase::OffsetVariable { var_index, offset } => {
                    todo!(); // Edit Variable
                    *phase += 1;
                    phase_transition(phase, state, program);
                },
            }
        }
        None => {
            *state = TimerState::Idle; 
            //*phase = 0;
        }
    };
}

impl TimerFSM {

    pub fn new(program: Vec<ProgramPhase>, variables: Option<Vec<i8>>) -> TimerFSM {
        TimerFSM {
            program: program,
            state: TimerState::Idle,
            phase: 0,
            variables: variables.unwrap_or(vec![]),
        }
    }

    pub fn input(&mut self, input: TimerInput) -> TimerOutput {
        let mut output: TimerOutput = TimerOutput::NoChange;
        match (&mut self.state, input) {
            (_, TimerInput::Stop) => {
                output = TimerOutput::ProgramStopped {
                    program_phase: *self.program.get(self.phase).unwrap_or(&ProgramPhase::EndProgram),
                };
                self.phase = 0;
                phase_transition( &mut self.program.len(), &mut self.state, &self.program);
            }
            (TimerState::Idle, TimerInput::Start) => {
                output = TimerOutput::PhaseChange { 
                    prev_phase: ProgramPhase::BeginProgram, 
                    next_phase: self.program[0], 
                    phase_completed: true,
                };
                phase_transition( &mut self.phase, &mut self.state, &self.program);
            }
            (TimerState::Idle, _) => {},
            (_, TimerInput::Start) => {},
            (TimerState::Timer {..} | TimerState::Input, TimerInput::Skip,) => {
                output = self.next_phase(false);
            }
            (
                TimerState::Timer {
                    progress,
                    duration: _,
                    paused,
                },
                TimerInput::Step,
            ) => {
                if *paused {
                    output = TimerOutput::NoChange;
                } else if *progress > 0 {
                    *progress -= 1;
                    output = TimerOutput::TimerProgress { seconds: *progress };
                } else {
                    output = self.next_phase(true);
                }
            }
            (
                TimerState::Timer {
                    progress,
                    duration,
                    paused: _,
                },
                TimerInput::Reset,
            ) => {
                *progress = *duration;
                output = TimerOutput::TimerReset { seconds: *progress };
            }
            (
                TimerState::Timer {
                    progress: _,
                    duration: _,
                    paused,
                },
                TimerInput::Pause,
            ) => {
                *paused = true;
                output = TimerOutput::TimerPaused;
            }
            (
                TimerState::Timer {
                    progress,
                    duration: _,
                    paused,
                },
                TimerInput::Resume,
            ) => {
                *paused = false;
                output = TimerOutput::TimerResumed { seconds: *progress };
            }
            (TimerState::Timer {..}, _,) => {},
            (TimerState::Input, TimerInput::Input) => {
                output = self.next_phase(true);
            },
            (TimerState::Input, _) => {},
        }
        return output;
    }


    fn next_phase(&mut self, prev_completed: bool) -> TimerOutput {
        let prev_phase: ProgramPhase = self.program[self.phase];
        self.phase += 1;
        phase_transition( &mut self.phase, &mut self.state, &self.program);
        return TimerOutput::PhaseChange { prev_phase: prev_phase, next_phase: *self.program.get(self.phase).unwrap_or(&ProgramPhase::EndProgram), phase_completed: prev_completed };
    }

    // fn to_phase(&mut self, phase_offset: usize) -> ProgramPhase {
    //     self.phase += phase_offset;
    //     match self.program.get(self.phase) {
    //         Some(phase_ref) => {
    //             let program_phase: ProgramPhase = *phase_ref;
    //             match program_phase {
    //                 ProgramPhase::TimeFor { duration } => {
    //                     self.state = TimerState::Timer {
    //                         progress: duration,
    //                         duration: duration,
    //                         paused: false,
    //                     };
    //                 }
    //                 ProgramPhase::BeginProgram => self.to_default(),
    //                 ProgramPhase::EndProgram => self.to_default(),
    //                 ProgramPhase::ReceiveInput => {
    //                     self.state = TimerState::Input;
    //                 },
    //                 ProgramPhase::Repeat { to_phase } => {

    //                 },
    //             }
    //             return program_phase;
    //         },
    //         None => {
    //             self.to_default();
    //             return ProgramPhase::EndProgram;
    //         },
    //     }
    // }
}

#[cfg(test)]
mod timer_util_tests {

    use super::{ProgramPhase, TimerFSM, TimerInput, TimerOutput, TimerState};
    use ProgramPhase::*;
    use TimerInput::*;
    use TimerOutput::*;
    use TimerState::*;

    #[test]
    fn timer_program() {
        let seconds: usize = 3;
        let mut model: TimerFSM = TimerFSM::new(vec![TimeFor { duration: seconds }], None);
        assert_eq!(Idle, model.state);
        assert_eq!(
            PhaseChange { prev_phase: BeginProgram, next_phase: TimeFor { duration: seconds }, phase_completed: true },
            model.input(Start)
        );
        for i in 1..seconds {
            assert_eq!(
                TimerProgress {
                    seconds: seconds - i
                },
                model.input(Step)
            );
        }
        assert_eq!(TimerProgress { seconds: 0 }, model.input(Step));
        assert_eq!(PhaseChange {prev_phase:TimeFor{duration:seconds}, next_phase:EndProgram, phase_completed: true }, model.input(Step));
        assert_eq!(Idle, model.state);
    }

    #[test]
    fn input_program() {
        let mut model: TimerFSM = TimerFSM::new(vec![ReceiveInput], None);
        assert_eq!(Idle, model.state);
        assert_eq!(PhaseChange {prev_phase: BeginProgram, next_phase: ReceiveInput, phase_completed: true }, model.input(TimerInput::Start));
        assert_eq!(PhaseChange {prev_phase: ReceiveInput, next_phase:EndProgram, phase_completed: true }, model.input(TimerInput::Input));
        assert_eq!(Idle, model.state);
    }

    #[test]
    fn skip_input_program() {
        let mut model: TimerFSM = TimerFSM::new(vec![ReceiveInput], None);
        assert_eq!(Idle, model.state);
        assert_eq!(PhaseChange {prev_phase: BeginProgram, next_phase: ReceiveInput, phase_completed: true }, model.input(TimerInput::Start));
        assert_eq!(PhaseChange {prev_phase: ReceiveInput, next_phase:EndProgram, phase_completed: false }, model.input(TimerInput::Skip));
        assert_eq!(Idle, model.state);
    }

    #[test]
    fn stop_timer_program() {
        let seconds: usize = 3;
        let mut model: TimerFSM = TimerFSM::new(vec![TimeFor { duration: seconds }], None);
        assert_eq!(Idle, model.state);
        assert_eq!(
            PhaseChange { prev_phase: BeginProgram, next_phase: TimeFor { duration: seconds }, phase_completed: true },
            model.input(Start)
        );
        assert_eq!(
            Timer {
                progress: seconds,
                duration: seconds,
                paused: false
            },
            model.state
        );
        for i in 1..seconds {
            assert_eq!(
                TimerProgress {
                    seconds: seconds - i
                },
                model.input(Step)
            );
        }
        assert_eq!(
            ProgramStopped {
                program_phase: TimeFor { duration: seconds }
            },
            model.input(Stop)
        );
        assert_eq!(Idle, model.state);
        assert_eq!(0, model.phase);
    }

    #[test]
    fn reset_timer_program() {
        let seconds: usize = 3;
        let mut model: TimerFSM = TimerFSM::new(vec![TimeFor { duration: seconds }], None);
        assert_eq!(Idle, model.state);
        assert_eq!(
            PhaseChange { prev_phase: BeginProgram, next_phase: TimeFor { duration: seconds }, phase_completed: true },
            model.input(Start)
        );
        assert_eq!(
            Timer {
                progress: seconds,
                duration: seconds,
                paused: false
            },
            model.state
        );
        for i in 1..seconds {
            assert_eq!(
                TimerProgress {
                    seconds: seconds - i
                },
                model.input(Step)
            );
        }
        assert_eq!(TimerReset { seconds: seconds }, model.input(Reset));
        assert_eq!(
            Timer {
                progress: seconds,
                duration: seconds,
                paused: false
            },
            model.state
        );
        assert_eq!(0, model.phase);
    }

    #[test]
    fn skip_timer_program() {
        let seconds: usize = 3;
        let mut model: TimerFSM = TimerFSM::new(vec![
            TimeFor { duration: seconds },
            TimeFor {
                duration: seconds + 1,
            },
        ], None);
        assert_eq!(Idle, model.state);
        assert_eq!(
            PhaseChange {prev_phase:BeginProgram,next_phase:TimeFor{duration:seconds}, phase_completed: true },
            model.input(Start)
        );
        assert_eq!(
            Timer {
                progress: seconds,
                duration: seconds,
                paused: false
            },
            model.state
        );
        assert_eq!(PhaseChange { prev_phase: TimeFor { duration: seconds }, next_phase: TimeFor { duration: seconds + 1 }, phase_completed: false }, model.input(Skip));
        assert_eq!(1, model.phase);
        assert_eq!(
            Timer {
                progress: seconds + 1,
                duration: seconds + 1,
                paused: false
            },
            model.state
        );
    }

    #[test]
    fn pause_timer_program() {
        let seconds: usize = 3;
        let mut model: TimerFSM = TimerFSM::new(vec![TimeFor { duration: seconds }], None);
        assert_eq!(Idle, model.state);
        assert_eq!(
            PhaseChange { prev_phase: BeginProgram, next_phase: TimeFor { duration: seconds }, phase_completed: true },
            model.input(Start)
        );
        assert_eq!(
            Timer {
                progress: seconds,
                duration: seconds,
                paused: false
            },
            model.state
        );
        assert_eq!(TimerPaused, model.input(Pause));
        assert_eq!(0, model.phase);
        assert_eq!(
            Timer {
                progress: seconds,
                duration: seconds,
                paused: true
            },
            model.state
        );
        assert_eq!(NoChange, model.input(Step));
        assert_eq!(
            Timer {
                progress: seconds,
                duration: seconds,
                paused: true
            },
            model.state
        );
    }

    #[test]
    fn resume_timer_program() {
        let seconds: usize = 3;
        let mut model: TimerFSM = TimerFSM::new(vec![TimeFor { duration: seconds }], None);
        assert_eq!(Idle, model.state);
        assert_eq!(
            PhaseChange {prev_phase:BeginProgram,next_phase:TimeFor{duration:seconds}, phase_completed: true },
            model.input(Start)
        );
        assert_eq!(
            Timer {
                progress: seconds,
                duration: seconds,
                paused: false
            },
            model.state
        );
        assert_eq!(TimerPaused, model.input(Pause));
        assert_eq!(
            Timer {
                progress: seconds,
                duration: seconds,
                paused: true
            },
            model.state
        );
        assert_eq!(TimerResumed { seconds: seconds }, model.input(Resume));
        assert_eq!(
            TimerProgress {
                seconds: seconds - 1
            },
            model.input(Step)
        );
        assert_eq!(
            Timer {
                progress: seconds - 1,
                duration: seconds,
                paused: false
            },
            model.state
        );
        assert_eq!(0, model.phase);
    }

    #[test]
    fn multi_timer_program() {
        let seconds: usize = 1;
        let mut model: TimerFSM = TimerFSM::new(vec![
            TimeFor { duration: seconds },
            TimeFor {
                duration: seconds + 1,
            },
            TimeFor {
                duration: seconds + 2,
            },
        ], None);
        assert_eq!(Idle, model.state);
        assert_eq!(
            PhaseChange {prev_phase:BeginProgram,next_phase:TimeFor{duration:seconds}, phase_completed: true },
            model.input(Start)
        );
        for i in 1..seconds {
            assert_eq!(
                TimerProgress {
                    seconds: seconds - i
                },
                model.input(Step)
            );
        }
        model.input(Step);
        assert_eq!(
            PhaseChange {prev_phase:TimeFor{duration:seconds}, next_phase:TimeFor{duration:seconds+1}, phase_completed: true },
            model.input(Step)
        );
        for i in 1..(seconds + 1) {
            assert_eq!(
                TimerProgress {
                    seconds: seconds + 1 - i
                },
                model.input(Step)
            );
        }
        assert_eq!(
            TimerProgress { seconds: 0 },
            model.input(Step)
        );
        assert_eq!(
            PhaseChange {prev_phase:TimeFor{duration:seconds + 1},next_phase:TimeFor{duration:seconds + 2}, phase_completed: true },
            model.input(Step)
        );
        for i in 1..(seconds + 2) {
            assert_eq!(
                TimerProgress {
                    seconds: seconds + 2 - i
                },
                model.input(Step)
            );
        }
        assert_eq!(
            TimerProgress { seconds: 0 },
            model.input(Step)
        );
        assert_eq!(
            PhaseChange {prev_phase:TimeFor{duration:seconds + 2}, next_phase:EndProgram, phase_completed: true },
            model.input(Step)
        );
        assert_eq!(Idle, model.state);
    }
}
