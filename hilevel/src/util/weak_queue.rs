use alloc::collections::LinkedList;
use alloc::rc::{Weak, Rc};

// A queue that uses weak references, i.e. doesn't have ownership of its items
pub struct WeakQueue<T> (LinkedList<Weak<T>>);

impl<T> WeakQueue<T> {

    pub fn push_back(&mut self, item: Weak<T>) {
        self.0.push_back(item);
    }

    pub fn pop_front(&mut self) -> Option<Rc<T>> {
        while !self.0.is_empty() {
            let popped = self.0.pop_front().unwrap();
            match Weak::upgrade(&popped) {
                Some(strong) => {return Some(strong)}
                _ => {}
            }
        }
        None
    }

    pub fn is_empty(&self) -> bool{
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

}

impl<T> Default for WeakQueue<T> {
    fn default() -> Self {
        WeakQueue{ 0: LinkedList::default() }
    }
}