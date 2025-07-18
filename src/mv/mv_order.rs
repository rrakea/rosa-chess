use super::mv::Mv;

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
        buffer: Vec::new(),
        buf_index: 0,
    }
}

pub struct MoveOrder<I>
where
    I: Iterator<Item = Mv>,
{
    iter: I,
    buffer: Vec<Mv>,
    buf_index: u8,
}

impl<I> Iterator for MoveOrder<I>
where
    I: Iterator<Item = Mv>,
{
    type Item = Mv;

    fn next(&mut self) -> Option<Mv> {
        for mv in &mut self.iter {
            if mv.is_cap() || mv.is_prom() {
                return Some(mv);
            }
            self.buffer.push(mv)
        }

        if self.buf_index as usize == self.buffer.len() {
            return None;
        }

        Some(self.buffer[self.buf_index as usize])
    }
}
