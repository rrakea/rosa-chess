pub fn rank(sq: u8) -> u8 {
    sq / 8
}

pub fn file(sq: u8) -> u8 {
    sq % 8
}

// This accomodates for knight moves
pub fn no_wrap(a: u8, b: u8) -> bool {
    (a as i16 % 8 - b as i16 % 8).abs() <= 2
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
