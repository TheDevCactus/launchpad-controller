use std::process::Command;

use regex::Regex;

pub fn get_current_volume() -> (u8, u8) {
    let reg = Regex::new("\\[(.*?)\\%\\]").unwrap();
    let volume = Command::new("amixer")
        .args(["-D", "pulse", "sget", "Master"])
        .output()
        .unwrap();
    let volume_output = String::from_utf8_lossy(volume.stdout.as_slice());
    let volume_output: Vec<&str> = volume_output.split('\n').collect();
    let left = volume_output[5];
    let right = volume_output[6];
    let left = reg.find(left).unwrap();
    let right = reg.find(right).unwrap();
    let mut left = left.as_str();
    let mut right = right.as_str();
    left = left.trim_end_matches("%]");
    right = right.trim_end_matches("%]");
    left = left.trim_start_matches('[');
    right = right.trim_start_matches('[');
    let parse_result = left.parse::<u8>();
    if let Err(_e) = parse_result {
        return (0, 0);
    }
    let left = parse_result.unwrap();

    let parse_result = right.parse::<u8>();
    if let Err(_e) = parse_result {
        return (0, 0);
    }
    let right = parse_result.unwrap();

    (left, right)
}

pub fn set_volume(new_value: u8) {
    if new_value > 100 {
        // !TODO throw an error here
        return;
    }
    let _res = Command::new("amixer")
        .args([
            "-D",
            "pulse",
            "sset",
            "Master",
            format!("{}%", new_value).as_str(),
        ])
        .output()
        .unwrap();
}

pub fn nudge(by: u8, nudge_up: bool) {
    let nudge_symbol = if nudge_up { "+" } else { "-" };
    let _res = Command::new("amixer")
        .args([
            "-D",
            "pulse",
            "sset",
            "Master",
            format!("{}%{}", by, nudge_symbol).as_str(),
        ])
        .output()
        .unwrap();
}
