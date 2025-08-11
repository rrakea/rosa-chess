use rosa_lib::mv::Mv;
/*
    Relativly basic move ordering, it just buffers all non capture moves,
    (and non promotions)
*/

pub fn order_mvs<I>(mv_iter: I) -> MoveOrder<I>
where
    I: Iterator<Item = Mv>,
{
    MoveOrder {
        iter: mv_iter,
        // Chess positions have around 35 moves on average -> we dont spend the time growing
        buffer: Vec::with_capacity(30),
        buf_index: 0,
        exhausted: false,
    }
}

pub struct MoveOrder<I>
where
    I: Iterator<Item = Mv>,
{
    iter: I,
    buffer: Vec<Mv>,
    buf_index: usize,
    exhausted: bool,
}

impl<I> Iterator for MoveOrder<I>
where
    I: Iterator<Item = Mv>,
{
    type Item = Mv;

    fn next(&mut self) -> Option<Mv> {
        loop {
            let mv = self.iter.next();
            if let Some(mv) = mv {
                if mv.is_cap() || mv.is_prom() {
                    return Some(mv);
                } else {
                    self.buffer.push(mv);
                }
            } else {
                self.exhausted = true;
                break;
            }
        }

        if self.exhausted && self.buf_index < self.buffer.len() {
            let next = self.buffer[self.buf_index];
            self.buf_index += 1;
            return Some(next);
        }

        None
    }
}
