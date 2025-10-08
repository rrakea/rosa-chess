use rosa_lib::board::*;

#[test]
fn ones() {
    let mut b = Board::new(0);
    let mut v = vec![0u32, 23, 2, 34, 24, 20, 63];
    b.toggle_all(v.clone());
    let res_vec = b.get_ones();
    assert_eq!(res_vec.len(), v.len());
    for res in res_vec {
        v.remove(res as usize);
    }
    assert_eq!(v.len(), 0);
}

#[test]
fn single() {
    let mut b = Board::new(0);
    let rand = rand::random_range(0..64);
    b.toggle(rand);
    assert_eq!(b.get_ones_single(), rand);
}

#[test]
fn ones2() {
    let mut b = Board::new(0);
    let mut v = vec![3u32, 52, 64, 2, 1, 0, 33, 43];
    let res_vec = b.get_ones();
    b.toggle_all(v.clone());
    assert_eq!(res_vec.len(), v.len());
    for res in res_vec {
        v.remove(res as usize);
    }
    assert_eq!(v.len(), 0);}
