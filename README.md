# Oxidoro

A Programmable Timer for the Web made with Slint and Rust

## About

The goal of this project is to create a way to layout the behavior for a multistage timer/activity: like Pomodoro, Exercise Routines / PT etc.
See Pomodoro Timer

## Usage

Build
`wasm-pack build --release --target web`
Run
`python3 -m http.server`

TODO

## Next Steps

- [ ] The Current Timer Scheme's Pause and Resume is only on the seconds level, so a pause and resume operate at the beginning of each second. A more sophisticated timer model would fix this, which is doable.
- [ ] "Play Sound" and "Display Arbitrary Text" as options in a program
- [ ] Editor for the user to create their own programs
- [ ] Getting the App to take up the whole page of the browser for multiple screen sizes

