use crate::{
    midi::{Midi, MidiControlMessage, CLIENT_NAME, IN_PORT_NAME, MIDI_NAME, OUT_PORT_NAME},
    services::volume::get_current_volume,
};
use midi::MidiIncomingMessage;
use services::{track, volume};
use utils::coord_to_note;

mod midi;
mod services;
mod utils;

/*
Key Mapping
    bit 1 == section
    bit 2 == note/button
    bit 3 == velocity

Sections
    176 == Top row
    144 == Everything but top row

Notes/Buttons
    Contained within section: 176
        104-111
    Contained within section: 144
        11-89
*/

const CONTROL_SECTION: u8 = 176;
const MAIN_SECTION: u8 = 144;
const BUTTON_UP_VELOCITY: u8 = 0;
const BUTTON_DOWN_VELOCITY: u8 = 127;

// CONTROL SECTION NOTES

const NUDGE_UP_NOTE: u8 = 104;
const NUDGE_DOWN_NOTE: u8 = 105;
const PREVIOUS_TRACK_NOTE: u8 = 106;
const NEXT_TRACK_NOTE: u8 = 107;

// MAIN SECTION NOTES

const STOP_NOTE: u8 = 49;
const MUTE_NOTE: u8 = 39;

#[derive(Debug)]
enum HandleMessageError {
    UnboundInput,
}

fn handle_control_message(message: MidiIncomingMessage) -> Result<(), HandleMessageError> {
    match message.data[1] {
        NUDGE_UP_NOTE => {
            volume::nudge(1, true);
        }
        NUDGE_DOWN_NOTE => {
            volume::nudge(1, false);
        }
        PREVIOUS_TRACK_NOTE => {
            track::previous();
        }
        NEXT_TRACK_NOTE => {
            track::next();
        }
        _ => {
            return Err(HandleMessageError::UnboundInput);
        }
    };

    return Ok(());
}
fn handle_main_message(message: MidiIncomingMessage) -> Result<(), HandleMessageError> {
    match message.data[1] {
        STOP_NOTE => {
            track::toggle_play_state();
        }
        MUTE_NOTE => {
            volume::set_volume(0);
        }
        _ => {
            return Err(HandleMessageError::UnboundInput);
        }
    };
    return Ok(());
}

fn handle_message(message: MidiIncomingMessage) -> Result<(), HandleMessageError> {
    if message.data[2] == BUTTON_UP_VELOCITY {
        return Ok(());
    }

    let handle_result = match message.data[0] {
        CONTROL_SECTION => handle_control_message(message),
        MAIN_SECTION => handle_main_message(message),
        _ => Err(HandleMessageError::UnboundInput),
    };

    return handle_result;
}

pub fn paint(midi: &mut Midi, _delta_t: u128) {
    let (left, right) = get_current_volume();
    for y in 0..8 {
        let r_val = left / 10;
        if y < r_val {
            midi.send(MidiControlMessage::LED_ON(coord_to_note(9, 8, 0, y), 2));
        } else {
            midi.send(MidiControlMessage::LED_OFF(coord_to_note(9, 8, 0, y)));
        }
    }
    for y in 0..8 {
        let r_val = right / 10;
        if y < r_val {
            midi.send(MidiControlMessage::LED_ON(coord_to_note(9, 8, 1, y), right));
        } else {
            midi.send(MidiControlMessage::LED_OFF(coord_to_note(9, 8, 1, y)));
        }
    }
}

fn main() {
    let midi = Midi::new(CLIENT_NAME, MIDI_NAME, IN_PORT_NAME, OUT_PORT_NAME);
    if let Err(_) = midi {
        println!("Failed to initialize midi");
        return;
    }
    let mut midi = midi.unwrap();

    let mut start_time = std::time::SystemTime::now();
    loop {
        let recv_res = midi.incoming.try_recv();
        if let Ok(message) = recv_res {
            let res = handle_message(message);
            if let Err(e) = res {
                println!("Encountered error while handling message: {:?}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
        let current_time = std::time::SystemTime::now();
        let delta_time = current_time.duration_since(start_time).unwrap().as_millis();
        start_time = current_time;

        paint(&mut midi, delta_time);
    }
}
