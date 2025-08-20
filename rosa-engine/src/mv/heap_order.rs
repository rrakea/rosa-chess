use rosa_lib::mv::Mv;

use std::collections::BinaryHeap;

pub struct HeapOrder<I>
where
    I: Iterator<Item = Mv>,
{
    iter: I,
    heap: BinaryHeap<Mv>,
}

pub fn order<I>(mv_iter: I) -> HeapOrder<I>
where
    I: Iterator<Item = Mv>,
{
    HeapOrder {
        iter: mv_iter,
        heap: BinaryHeap::with_capacity(35),
    }
}

impl<I> Iterator for HeapOrder<I>
where
    I: Iterator<Item = Mv>,
{
    type Item = Mv;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Mv = self.iter.next();
            match Mv {
                Some(m) => {
                    self.heap.push(m);
                }
                None => break,
            }
        }
        self.heap.pop()
    }
}
