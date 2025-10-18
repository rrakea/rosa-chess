use rosa_lib::board::*;

use std::collections::HashSet;

fn check_board(nums: Vec<u8>) {
    let mut b = Board::new();
    b.toggle_all(nums.clone());
    let res_set: HashSet<_> = b.get_ones().into_iter().collect();
    let num_set: HashSet<_> = nums.into_iter().collect();
    assert_eq!(res_set, num_set);
    assert_eq!(res_set.len(), num_set.len());
}

#[test]
fn ones() {
    let v = vec![0, 23, 2, 34, 24, 20, 63];
    check_board(v);
}

#[test]
fn ones2() {
    let v = vec![3, 52, 63, 2, 1, 0, 33, 43];
    check_board(v);
}

#[test]
fn single() {
    let mut b = Board::new();
    let rand = rand::random_range(0..64);
    b.toggle(rand);
    assert_eq!(b.get_ones_single(), rand);
}

