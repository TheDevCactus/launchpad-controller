use crate::midi::{Midi, MidiControlMessage, CLIENT_NAME, IN_PORT_NAME, MIDI_NAME, OUT_PORT_NAME};

mod midi;

// 11-88 are the "id's" of the keys on the launchpad.

/*
RULES --

Any live cell with fewer than two live neighbours dies, as if by underpopulation.
Any live cell with two or three live neighbours lives on to the next generation.
Any live cell with more than three live neighbours dies, as if by overpopulation.
Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
*/

struct Conway {
    pub running: bool,
    pub board: Vec<Vec<u8>>,
}

impl Conway {
    pub fn new(x_size: u8, y_size: u8) -> Conway {
        let mut board: Vec<Vec<u8>> = Vec::new();

        for _ in 0..x_size {
            let mut row: Vec<u8> = Vec::new();
            for _ in 0..y_size {
                row.push(0);
            }
            board.push(row);
        }

        return Conway {
            running: false,
            board: board,
        };
    }

    pub fn tick(&mut self) {
        for x in 0..self.board.len() {
            for y in 0..self.board[x].len() {
                let neighbor_count = self.check_neighbors(y, x);
                let is_alive = self.board[x][y] == 1;

                if !is_alive && neighbor_count == 3 {
                    self.board[x][y] = 1;
                    continue;
                }
                if is_alive && neighbor_count != 2 && neighbor_count != 3 {
                    self.board[x][y] = 0;
                }
            }
        }
    }

    // THIS ISN'T RIGHT AT ALL LOL
    pub fn check_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut neighbor_count = 0;

        for x_offset in -1..2 {
            for y_offset in -1..2 {
                if x_offset == 0 && y_offset == 0 {
                    continue;
                }
                let final_x: isize = x as isize + x_offset;
                let final_y: isize = y as isize + y_offset;
                if final_x < 0 {
                    continue;
                }
                if final_y < 0 {
                    continue;
                }
                if final_x > self.board.len() as isize - 1 {
                    continue;
                }
                if final_y > self.board[0].len() as isize - 1 {
                    continue;
                }

                if self.board[final_x as usize][final_y as usize] == 1 {
                    neighbor_count += 1;
                }
            }
        }

        return neighbor_count;
    }

    pub fn toggle_cell_state(&mut self, x: u8, y: u8) {
        if self.board[x as usize][y as usize] == 0 {
            self.set_cell_state(x, y, true);
            return;
        }
        self.set_cell_state(x, y, false);
    }

    pub fn set_cell_state(&mut self, x: u8, y: u8, is_alive: bool) {
        let mut new_cell_state = 0;
        if is_alive {
            new_cell_state = 1;
        }
        self.board[x as usize][y as usize] = new_cell_state;
    }
}

fn draw_conway_to_midi(midi: &mut Midi, conway: &mut Conway) {
    for x in 0..conway.board.len() {
        for y in 0..conway.board[x].len() {
            let is_alive = conway.board[x][y] == 1;
            let note = coord_to_note(
                conway.board.len() as u8,
                conway.board.len() as u8,
                x as u8,
                y as u8,
            );

            if is_alive {
                midi.send(MidiControlMessage::LED_ON(note, 100));
            } else {
                midi.send(MidiControlMessage::LED_OFF(note, 0));
            }
        }
    }
}

fn coord_to_note(x_dimen: u8, y_dimen: u8, x: u8, y: u8) -> u8 {
    let mut out = 0x0b;
    out += x;
    // you add y here because the guy at novation decided
    // each row of buttons should be offset from the last a single extra note.
    // i.e. the right most button of the bottom row on the pad has the id 9,
    // the left most button of the second to bottom row on the pad has the id 11
    // when we would otherwise expect 10
    out += y;
    out += y * x_dimen;
    return out;
}

fn note_to_coord(note: u8, x_dimen: usize, y_dimen: usize) -> (u8, u8) {
    let mut base_note = note - 0x0b;
    let mut cols = 0;
    while base_note > x_dimen as u8 {
        base_note -= x_dimen as u8;
        base_note -= 1;
        cols += 1;
    }
    // println!("X: {}, Y: {}", base_note, cols);
    return (base_note, cols);
}

fn main() {
    let midi = Midi::new(CLIENT_NAME, MIDI_NAME, IN_PORT_NAME, OUT_PORT_NAME);
    if let Err(_) = midi {
        println!("Failed to initialize midi");
        return;
    }

    let mut midi = midi.unwrap();
    let mut conway = Conway::new(9, 8);
    draw_conway_to_midi(&mut midi, &mut conway);

    loop {
        if conway.running {
            conway.tick();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        draw_conway_to_midi(&mut midi, &mut conway);

        let res = midi.incoming.try_recv();
        if let Err(_) = res {
            continue;
        }
        let res = res.unwrap();

        // is state control button
        if res.data[0] == 176 {
            if res.data[1] == 104 {
                conway.running = true;
            }
            if res.data[1] == 105 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        if res.data[2] == 0 {
            continue;
        }
        // is cell button
        let coord = note_to_coord(res.data[1], conway.board.len(), conway.board[0].len());
        conway.toggle_cell_state(coord.0, coord.1);
    }

    println!("Thanks for playing!");
}
