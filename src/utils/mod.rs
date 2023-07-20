pub fn coord_to_key(x: u8, y: u8) -> u8 {
    return (y * 10) + 1 + x;
}

pub fn key_to_coord(key: u8) -> (u8, u8) {
    return ((key % 10) - 1, key / 10);
}
