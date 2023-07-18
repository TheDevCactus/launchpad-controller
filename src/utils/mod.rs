pub fn coord_to_note(x_dimen: u8, y_dimen: u8, x: u8, y: u8) -> u8 {
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

pub fn note_to_coord(note: u8, x_dimen: usize, y_dimen: usize) -> (u8, u8) {
    let mut base_note = note - 0x0b;
    let mut cols = 0;
    while base_note > x_dimen as u8 {
        base_note -= x_dimen as u8;
        base_note -= 1;
        cols += 1;
    }
    return (base_note, cols);
}
