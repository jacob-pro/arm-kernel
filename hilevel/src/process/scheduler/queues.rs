use core::ops;
use core::cell::RefCell;
use crate::process::{ProcessControlBlock, StrongPcbRef};
use alloc::rc::{Weak, Rc};
use alloc::collections::VecDeque;
use alloc::vec::Vec;

pub type StrongQueueRef = Rc<RefCell<QueueLevel>>;
type QueueInternal = VecDeque<StrongPcbRef>;

const QUEUE_QUANTUM: &[u32] = &[2, 4, 8, 16];

// Both above and below can't be strong otherwise there would be a reference cycle
pub struct QueueLevel {
    above: Option<Weak<RefCell<QueueLevel>>>,
    internal: QueueInternal,
    below: Option<StrongQueueRef>,
    quantum: u32,
}

pub trait LinkedQueues {
    fn below(&self) -> Option<StrongQueueRef>;
    fn above(&self) -> Option<StrongQueueRef>;
}

impl LinkedQueues for StrongQueueRef {

    fn below(&self) -> Option<StrongQueueRef> {
        self.borrow().below.as_ref().map(|x| Rc::clone(x))
    }

    fn above(&self) -> Option<StrongQueueRef> {
        self.borrow().above.as_ref().map(|x| Weak::upgrade(x).unwrap())
    }

}

impl ops::Deref for QueueLevel {
    type Target = QueueInternal;
    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl ops::DerefMut for QueueLevel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.internal
    }
}

impl QueueLevel {
    pub fn quantum(&self) -> u32 {
        self.quantum
    }
}

pub struct MultiLevelQueue {
    top: StrongQueueRef
}

impl MultiLevelQueue {

    pub fn new(mut quantums: Vec<u32>) -> Self {
        assert!(quantums.len() > 0);
        // Create top queue
        let top = Rc::new(RefCell::new(QueueLevel {
            above: None,
            internal: Default::default(),
            below: None,
            quantum: quantums.remove(0),
        }));
        let mut queues = vec![top];
        // Add queues below, linking each up above
        for quantum in quantums.into_iter() {
            Rc::new(RefCell::new(QueueLevel {
                above: Some(Rc::downgrade(queues.last().unwrap())),
                internal: Default::default(),
                below: None,
                quantum,
            }));
        }
        // Link the queues to the ones below, by iterating upwards
        let mut i = Rc::clone(queues.last().unwrap());
        while i.borrow().above.is_some() {
            let above = i.borrow().above.as_ref().map(|x| Weak::upgrade(x).unwrap()).unwrap();
            above.borrow_mut().below = Some(Rc::clone(&i));
            i = above;
        }
        MultiLevelQueue { top: queues.remove(0) }
    }

    pub fn top_queue(&self) -> StrongQueueRef {
        Rc::clone(&self.top)
    }

    fn iter(&self) -> MultiLevelQueueIterator {
        MultiLevelQueueIterator {start: Rc::clone(&self.top), current: None }
    }

    // If a given process is contained in any queue
    pub fn contains(&self, process: &StrongPcbRef) -> bool {
        for queue in self.iter() {
            let borrow = queue.borrow();
            if borrow.iter().any(|x| Rc::ptr_eq(process, x)) {return true}
        }
        false
    }

    // Search queues for first matching process
    pub fn pop_process<F>(&mut self, filter: F) -> Option<(StrongPcbRef, StrongQueueRef)>
        where F: Fn(&ProcessControlBlock)->bool
    {
        for queue in self.iter() {
            let mut borrowed = queue.borrow_mut();
            for (i, item) in borrowed.iter().enumerate() {
                if filter(& (*item).borrow()) {
                    let popped = borrowed.remove(i).unwrap();
                    return Some((popped, Rc::clone(&queue)))
                }
            }
        }
        None
    }

    // Moves all processes to the top queue
    pub fn boost(&mut self) {
        for queue in self.iter().skip(1) {
            let x = &mut queue.borrow_mut().internal;
            self.top.borrow_mut().internal.append(x)
        }
    }

    // Removes a process if it is found in any queue
    pub fn remove_process(&mut self, process: &StrongPcbRef) -> Option<StrongPcbRef> {
        for queue in self.iter() {
            let mut borrow = queue.borrow_mut();
            for (i, item) in borrow.iter().enumerate() {
                if Rc::ptr_eq(process, item) {
                    return borrow.remove(i)
                }
            }
        }
        None
    }

}

impl Default for MultiLevelQueue {
    fn default() -> Self {
        MultiLevelQueue::new(QUEUE_QUANTUM.to_vec())
    }
}

struct MultiLevelQueueIterator {
    start: StrongQueueRef,
    current: Option<StrongQueueRef>,
}

// Iterates through queues downwards
impl Iterator for MultiLevelQueueIterator {
    type Item = StrongQueueRef;

    fn next(&mut self) -> Option<Self::Item> {
        self.current = match &self.current {
            None => {
                Some(Rc::clone(&self.start))
            },
            Some(x) => {
                Rc::clone(x).borrow().below.clone()
            },
        };
        self.current.clone()
    }
}

#[cfg(test)]
mod tests {


    #[test]
    fn new_test() {

    }

    #[test]
    fn iter_test() {

    }

    #[test]
    fn contains_test() {

    }

    #[test]
    fn boost_test() {

    }

    #[test]
    fn remove_test() {

    }

}
