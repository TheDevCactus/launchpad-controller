use std::sync::mpsc;

use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

pub const CLIENT_NAME: &str = "midi thing";
pub const MIDI_NAME: &str = "Launchpad MK2:Launchpad MK2 MIDI 1 20:0";
pub const IN_PORT_NAME: &str = "port_in_midi_thing";
pub const OUT_PORT_NAME: &str = "port_out_midi_thing";

pub const NOTE_OFF: u8 = 0b10000000;
pub const NOTE_ON: u8 = 0b10010000;

#[derive(Debug)]
pub enum MidiControlMessage {
    LED_ON(u8, u8),
    LED_OFF(u8),
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
            if let Err(_) = port_name {
                return false;
            }
            let port_name = port_name.unwrap();
            return port_name == device_name;
        });
        let output_port = output_ports.iter().find(|port| {
            let port_name = midi_out.port_name(port);
            if let Err(_) = port_name {
                return false;
            }
            let port_name = port_name.unwrap();
            return port_name == device_name;
        });

        if let None = input_port {
            return Err(());
        }
        if let None = output_port {
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

                channel.0.send(MidiIncomingMessage {
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

        return Ok(Midi {
            in_connection: input_connection,
            out_connection: output_connection,
            incoming: channel.1,
        });
    }

    pub fn send(&mut self, msg: MidiControlMessage) {
        match msg {
            MidiControlMessage::LED_ON(note, vel) => {
                let _ = self.out_connection.send(&[NOTE_ON, note, vel]);
            }
            MidiControlMessage::LED_OFF(note) => {
                let _ = self.out_connection.send(&[NOTE_ON, note, 0]);
            }
            _ => {
                let _ = self.out_connection.send(&[NOTE_OFF, 0x0b, 0x7f]);
            }
        };
    }

    pub fn close(&mut self) {}
}
