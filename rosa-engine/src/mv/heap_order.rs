use rosa_lib::long_mv::LongMv;

use std::collections::BinaryHeap;

pub struct HeapOrder<I>
where
    I: Iterator<Item = LongMv>,
{
    iter: I,
    heap: BinaryHeap<LongMv>,
}

pub fn order<I>(mv_iter: I) -> HeapOrder<I>
where
    I: Iterator<Item = LongMv>,
{
    HeapOrder {
        iter: LongMv_iter,
        heap: BinaryHeap::with_capacity(35),
    }
}

impl<I> Iterator for HeapOrder<I>
where
    I: Iterator<Item = LongMv>,
{
    type Item = LongMv;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let LongMv = self.iter.next();
            match LongMv {
                Some(m) => {
                    if m.cutoff() {
                        return Some(m);
                    } else {
                        self.heap.push(m);
                        continue;
                    }
                }
                None => break,
            }
        }
        self.heap.pop()
    }
}
