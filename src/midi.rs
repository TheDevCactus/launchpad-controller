use std::sync::mpsc;

use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

pub const CLIENT_NAME: &str = "midi thing";
pub const MIDI_NAME: &str = "Launchpad MK2:Launchpad MK2 MIDI 1 20:0";
pub const IN_PORT_NAME: &str = "port_in_midi_thing";
pub const OUT_PORT_NAME: &str = "port_out_midi_thing";

pub const LED_SECTION: u8 = 0b10010000;
pub const CONTROL_SECTION: u8 = 176;

#[derive(Debug)]
pub enum Colors {
    Off,
    Green,
    Blue,
    Red,
    Purple,
}

#[derive(Debug)]
pub enum Brightness {
    Low,
    Medium,
    High,
}

impl From<u8> for Brightness {
    fn from(x: u8) -> Self {
        if x <= 3 {
            return Brightness::Low;
        }
        if x >= 7 {
            return Brightness::High;
        }
        return Brightness::Medium;
    }
}

#[derive(Debug)]
pub struct Color {
    color: Colors,
    brightness: Brightness,
}

impl Color {
    pub fn new(color: Colors, brightness: Brightness) -> Self {
        Color {
            color: color,
            brightness: brightness,
        }
    }
}

#[derive(Debug)]
pub enum MidiControlMessage {
    CONTROL_LED_ON(u8, Color),
    CONTROL_LED_OFF(u8),
    LED_ON(u8, Color),
    LED_OFF(u8),
}

impl Into<u8> for Color {
    fn into(self) -> u8 {
        match self.color {
            Colors::Blue => match self.brightness {
                Brightness::High => 41,
                Brightness::Low => 42,
                Brightness::Medium => 40,
            },
            Colors::Green => match self.brightness {
                Brightness::High => 17,
                Brightness::Low => 16,
                Brightness::Medium => 18,
            },
            Colors::Off => match self.brightness {
                Brightness::High => 0,
                Brightness::Low => 0,
                Brightness::Medium => 0,
            },
            Colors::Purple => match self.brightness {
                Brightness::High => 49,
                Brightness::Low => 50,
                Brightness::Medium => 48,
            },
            Colors::Red => match self.brightness {
                Brightness::High => 5,
                Brightness::Low => 6,
                Brightness::Medium => 60,
            },
        }
    }
}

pub struct MidiIncomingMessage {
    pub time: u64,
    pub data: [u8; 3],
}

pub struct Midi {
    in_connection: MidiInputConnection<()>,
    out_connection: MidiOutputConnection,
    pub incoming: mpsc::Receiver<MidiIncomingMessage>,
}

impl Midi {
    pub fn new(
        client_name: &str,
        device_name: &str,
        input_port_name: &str,
        output_port_name: &str,
    ) -> Result<Self, ()> {
        let midi_in = MidiInput::new(client_name);
        let midi_out = MidiOutput::new(client_name);
        if let Err(_e) = midi_in {
            return Err(());
        }
        if let Err(_e) = midi_out {
            return Err(());
        }

        let midi_in = midi_in.unwrap();
        let midi_out = midi_out.unwrap();

        let input_ports = midi_in.ports();
        let output_ports = midi_out.ports();

        let input_port = input_ports.iter().find(|port| {
            let port_name = midi_in.port_name(port);
            if port_name.is_err() {
                return false;
            }
            let port_name = port_name.unwrap();
            port_name == device_name
        });
        let output_port = output_ports.iter().find(|port| {
            let port_name = midi_out.port_name(port);
            if port_name.is_err() {
                return false;
            }
            let port_name = port_name.unwrap();
            port_name == device_name
        });

        if input_port.is_none() {
            return Err(());
        }
        if output_port.is_none() {
            return Err(());
        }

        let input_port = input_port.unwrap();
        let output_port = output_port.unwrap();

        let channel = mpsc::channel();
        let input_connection_result = midi_in.connect(
            input_port,
            input_port_name,
            move |stamp, msg, _| {
                #[cfg(debug_assertions)]
                println!("----\nReceived Message\nSection: {:?}\nNote/Button: {:?}\nVelocity: {:?}\n----", msg[0], msg[1], msg[2]);

                let _ = channel.0.send(MidiIncomingMessage {
                    time: stamp,
                    data: [msg[0], msg[1], msg[2]],
                });
            },
            (),
        );
        if let Err(_e) = input_connection_result {
            return Err(());
        }
        let input_connection = input_connection_result.unwrap();

        let output_connection_result = midi_out.connect(output_port, output_port_name);
        if let Err(_e) = output_connection_result {
            return Err(());
        }
        let output_connection = output_connection_result.unwrap();

        Ok(Midi {
            in_connection: input_connection,
            out_connection: output_connection,
            incoming: channel.1,
        })
    }

    pub fn send(&mut self, msg: MidiControlMessage) {
        match msg {
            MidiControlMessage::CONTROL_LED_OFF(note) => {
                let _ = self.out_connection.send(&[CONTROL_SECTION, note, 0]);
            }
            MidiControlMessage::CONTROL_LED_ON(note, color) => {
                let _ = self
                    .out_connection
                    .send(&[CONTROL_SECTION, note, color.into()]);
            }
            MidiControlMessage::LED_ON(note, color) => {
                let _ = self.out_connection.send(&[LED_SECTION, note, color.into()]);
            }
            MidiControlMessage::LED_OFF(note) => {
                let _ = self.out_connection.send(&[LED_SECTION, note, 0]);
            }
        };
    }

    pub fn close(&mut self) {}
}
