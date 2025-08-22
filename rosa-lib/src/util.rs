pub fn rank(sq: u8) -> u8 {
    sq / 8
}

pub fn file(sq: u8) -> u8 {
    sq % 8
}

pub fn same_colors(a: i8, b: i8) -> bool {
    a ^ b >= 0
}

pub fn dif_colors(a: i8, b: i8) -> bool {
    a ^ b < 0
}

// This accomodates for knight moves
pub fn no_wrap(a: u8, b: u8) -> bool {
    (a as i16 % 8 - b as i16 % 8).abs() <= 2
}

pub fn is_op_piece(active: i8, p: i8) -> bool {
    active * p < 0
}

pub fn is_self_piece(active: i8, p: i8) -> bool {
    active * p >= 0
}

pub fn square_name(sq: u8) -> String {
    let file = sq % 8;
    let rank = sq / 8;
    let filestr = (b'a' + file) as char;
    let rankstr = (b'1' + rank) as char;
    format!("{}{}", filestr, rankstr)
}

pub fn square_num(sq: &str) -> u8 {
    let file = sq.chars().nth(0).unwrap();
    let rank = sq.chars().nth(1).unwrap();
    let file = file as u8 - b'a';
    let rank = rank.to_digit(10).unwrap() as u8 - 1;
    rank * 8 + file
}
