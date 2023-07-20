use crate::{
    midi::{Midi, MidiControlMessage, CLIENT_NAME, IN_PORT_NAME, MIDI_NAME, OUT_PORT_NAME},
    services::log::{log, Log},
    services::volume::get_current_volume,
};
use midi::{Brightness, Color, Colors, MidiIncomingMessage};
use services::{track, volume};
use utils::{coord_to_key, key_to_coord};

mod midi;
mod services;
mod utils;

const CONTROL_SECTION: u8 = 176;
const MAIN_SECTION: u8 = 144;
const BUTTON_UP_VELOCITY: u8 = 0;

const NUDGE_UP_NOTE: u8 = 104;
const NUDGE_DOWN_NOTE: u8 = 105;
const PREVIOUS_TRACK_NOTE: u8 = 106;
const NEXT_TRACK_NOTE: u8 = 107;

const STOP_NOTE: u8 = 39;
const MUTE_NOTE: u8 = 29;
const LOOP_NOTE: u8 = 19;

const VOLUME_COLUMN_LEFT: u8 = 0;
const VOLUME_COLUMN_RIGHT: u8 = 1;

#[derive(Debug)]
enum HandleMessageError {
    UnboundInput,
}

fn handle_control_message(message: MidiIncomingMessage) -> Result<(), HandleMessageError> {
    match message.data[1] {
        NUDGE_UP_NOTE => {
            volume::nudge(5, true);
        }
        NUDGE_DOWN_NOTE => {
            volume::nudge(5, false);
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
    Ok(())
}

fn handle_main_message(message: MidiIncomingMessage) -> Result<(), HandleMessageError> {
    let (x, y) = key_to_coord(message.data[1]);

    if x == VOLUME_COLUMN_LEFT || x == VOLUME_COLUMN_RIGHT {
        volume::set_volume((y + 1) * 10);
    }

    match message.data[1] {
        STOP_NOTE => track::toggle_play_state(),
        MUTE_NOTE => volume::set_volume(0),
        LOOP_NOTE => track::toggle_loop_state(),
        _ => {
            return Err(HandleMessageError::UnboundInput);
        }
    };
    Ok(())
}

fn handle_message(message: MidiIncomingMessage) -> Result<(), HandleMessageError> {
    log(Log {
        msg: String::from_utf8_lossy(&message.data).to_string(),
    });
    if message.data[2] == BUTTON_UP_VELOCITY {
        return Ok(());
    }

    match message.data[0] {
        CONTROL_SECTION => handle_control_message(message),
        MAIN_SECTION => handle_main_message(message),
        _ => Err(HandleMessageError::UnboundInput),
    }
}

fn paint_volume(midi: &mut Midi, _delta_t: u128) {
    let (left, _) = get_current_volume();
    let l_val = left / 10;
    for y in 0..9 {
        if y < l_val {
            midi.send(MidiControlMessage::LED_ON(
                coord_to_key(VOLUME_COLUMN_LEFT, y),
                Color::new(Colors::Green, y.into()),
            ));
            midi.send(MidiControlMessage::LED_ON(
                coord_to_key(VOLUME_COLUMN_RIGHT, y),
                Color::new(Colors::Green, y.into()),
            ));
        } else {
            midi.send(MidiControlMessage::LED_OFF(coord_to_key(
                VOLUME_COLUMN_LEFT,
                y,
            )));
            midi.send(MidiControlMessage::LED_OFF(coord_to_key(
                VOLUME_COLUMN_RIGHT,
                y,
            )));
        }
    }
}

fn paint_control_buttons(midi: &mut Midi, _delta_t: u128) {
    let play_state = track::get_play_state();
    let play_state_color = match play_state {
        track::PlayState::Playing => Color::new(Colors::Green, Brightness::High),
        track::PlayState::Stopped => Color::new(Colors::Red, Brightness::High),
    };

    let (volume, _) = volume::get_current_volume();
    let mute_state_color = Color::new(Colors::Green, (volume / 10).into());

    let loop_state = track::get_loop_state();
    let loop_state_color = match loop_state {
        track::LoopState::None => Color::new(Colors::Red, Brightness::High),
        track::LoopState::Playlist => Color::new(Colors::Purple, Brightness::High),
        track::LoopState::Track => Color::new(Colors::Green, Brightness::High),
    };

    midi.send(MidiControlMessage::LED_ON(STOP_NOTE, play_state_color));
    midi.send(MidiControlMessage::LED_ON(MUTE_NOTE, mute_state_color));
    midi.send(MidiControlMessage::LED_ON(LOOP_NOTE, loop_state_color));

    midi.send(MidiControlMessage::CONTROL_LED_ON(
        NUDGE_UP_NOTE,
        Color::new(Colors::Red, Brightness::High),
    ));
    midi.send(MidiControlMessage::CONTROL_LED_ON(
        NUDGE_DOWN_NOTE,
        Color::new(Colors::Red, Brightness::Low),
    ));
    midi.send(MidiControlMessage::CONTROL_LED_ON(
        PREVIOUS_TRACK_NOTE,
        Color::new(Colors::Blue, Brightness::High),
    ));
    midi.send(MidiControlMessage::CONTROL_LED_ON(
        NEXT_TRACK_NOTE,
        Color::new(Colors::Blue, Brightness::Low),
    ));
}
pub fn paint(midi: &mut Midi, delta_t: u128) {
    paint_volume(midi, delta_t);
    paint_control_buttons(midi, delta_t);
}

fn main() {
    let midi = Midi::new(CLIENT_NAME, MIDI_NAME, IN_PORT_NAME, OUT_PORT_NAME);
    if midi.is_err() {
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
        } else {
            // only sleep if there is an empty buffer.
            // i.e. no user input
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let current_time = std::time::SystemTime::now();
        let delta_time = current_time.duration_since(start_time).unwrap().as_millis();
        start_time = current_time;

        paint(&mut midi, delta_time);
    }
}
