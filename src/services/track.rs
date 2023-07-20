use std::process::Command;

#[derive(Debug)]
pub enum PlayState {
    Playing,
    Stopped,
}

const PLAYING_STATUS: &str = "Playing";
const STOPPED_STATUS: &str = "Stopped";

#[derive(Debug)]
pub enum LoopState {
    None,
    Track,
    Playlist,
}

impl Into<String> for LoopState {
    fn into(self) -> String {
        match self {
            LoopState::None => NONE_STATUS.to_string(),
            LoopState::Playlist => PLAYLIST_STATUS.to_string(),
            LoopState::Track => TRACK_STATUS.to_string(),
        }
    }
}

const NONE_STATUS: &str = "None";
const TRACK_STATUS: &str = "Track";
const PLAYLIST_STATUS: &str = "Playlist";

pub fn get_play_state() -> PlayState {
    let raw_output = Command::new("playerctl")
        .args(["-a", "status"])
        .output()
        .unwrap()
        .stdout;
    let stringified_output = String::from_utf8_lossy(&raw_output);
    let is_playing = stringified_output
        .split('\n')
        .any(|status| status == PLAYING_STATUS);

    match is_playing {
        true => PlayState::Playing,
        false => PlayState::Stopped,
    }
}

pub fn toggle_play_state() {
    let _res = Command::new("playerctl")
        .args(["-a", "play-pause"])
        .output();
}

pub fn pause() {
    let _res = Command::new("playerctl").args(["-a", "pause"]).output();
}

pub fn play() {
    let _res = Command::new("playerctl").args(["-a", "play"]).output();
}

pub fn get_loop_state() -> LoopState {
    let raw_output = Command::new("playerctl")
        .args(["-a", "loop"])
        .output()
        .unwrap()
        .stdout;
    let stringified_output = String::from_utf8_lossy(&raw_output).to_string();
    let cleaned_output = stringified_output.trim_end_matches("\n");

    match cleaned_output {
        PLAYLIST_STATUS => LoopState::Playlist,
        TRACK_STATUS => LoopState::Track,
        _ => LoopState::None,
    }
}

pub fn toggle_loop_state() {
    let current_loop_state = get_loop_state();
    match current_loop_state {
        LoopState::None => set_loop_state(LoopState::Playlist),
        LoopState::Playlist => set_loop_state(LoopState::Track),
        LoopState::Track => set_loop_state(LoopState::None),
    }
}

pub fn set_loop_state(new_loop_state: LoopState) {
    let str_loop_state: String = new_loop_state.into();
    let _res = Command::new("playerctl")
        .args(["-a", "loop", str_loop_state.as_str()])
        .output();
}

pub fn next() {
    let _res = Command::new("playerctl").args(["-a", "next"]).output();
}

pub fn previous() {
    let _res = Command::new("playerctl").args(["-a", "previous"]).output();
}
