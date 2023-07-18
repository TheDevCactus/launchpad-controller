use std::process::Command;

pub enum PlayState {
    Playing,
    Stopped,
}

const PLAYING_STATUS: &str = "Playing";
const STOPPED_STATUS: &str = "Stopped";

pub fn get_play_state() -> PlayState {
    let raw_output = Command::new("playerctl")
        .args(["-a", "status"])
        .output()
        .unwrap()
        .stdout;
    let stringified_output = String::from_utf8_lossy(&raw_output);
    let is_playing = stringified_output
        .split("\n")
        .any(|status| status == PLAYING_STATUS);

    return match is_playing {
        true => PlayState::Playing,
        false => PlayState::Stopped,
    };
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

pub fn next() {
    let _res = Command::new("playerctl").args(["-a", "next"]).output();
}

pub fn previous() {
    let _res = Command::new("playerctl").args(["-a", "previous"]).output();
}
