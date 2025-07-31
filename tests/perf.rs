use rosa::mv;
use rosa::*;

fn init() {
    table::init_zobrist_keys();
    mv::magic_init::init_magics();
}

const START_POS: [u64; 9] = [
    1,
    20,
    400,
    8902,
    197281,
    4865609,
    119060324,
    3195901860,
    84998978956,
];

#[test]
fn starting_pos() {
    init();
    let pos = fen::starting_pos();
    check_perft_res(&pos, &START_POS);
}

fn check_perft_res(p: &pos::Pos, res: &[u64]) {
    for (i, r) in res.iter().enumerate() {
        assert_eq!(counting_search(p, i, 0, "".to_string()), *r);
    }
}

fn counting_search(p: &pos::Pos, depth: usize, do_div_before: usize, mv_stack: String) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut count: u64 = 0;
    let mv_iter = mv::mv_gen::gen_mvs(p).filter(|mv| !mv.is_null());
    for mv in mv_iter {
        let npos = mv::mv_apply::apply(p, &mv);
        let npos = match npos {
            Some(n) => n,
            None => continue,
        };
        let next_stack = if depth < do_div_before {
            mv_stack.clone() + mv.notation().as_str()
        } else {
            "".to_string()
        };
        count += counting_search(&npos, depth - 1, do_div_before, next_stack);
    }
    if depth < do_div_before {
        println!("Depth left: {depth}, Mv stack: {mv_stack}, Count: {count}");
    }
    count
}

#[test]
fn division_perf() {
    init();
    let pos = fen::starting_pos();
    for (i, r) in START_POS.iter().enumerate() {
        let res = counting_search(&pos, i, i - 3, "".to_string());
        println!("Depth: {i}, Got: {res}, Expected: {r}")
    }
}
