use core::ops;
use core::cell::RefCell;
use crate::process::{ProcessControlBlock, StrongPcbRef};
use alloc::rc::{Weak, Rc};
use alloc::collections::VecDeque;

pub type StrongQueueRef = Rc<RefCell<Queue>>;
type QueueInternal = VecDeque<StrongPcbRef>;

// Both above and below can't be strong otherwise there would be a reference cycle
pub struct Queue {
    above: Option<Weak<RefCell<Queue>>>,
    internal: QueueInternal,
    below: Option<Rc<RefCell<Queue>>>,
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

impl ops::Deref for Queue {
    type Target = QueueInternal;
    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl ops::DerefMut for Queue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.internal
    }
}

impl Queue {
    pub fn quantum(&self) -> u32 {
        self.quantum
    }
}

pub struct MultiLevelQueue {
    top: StrongQueueRef
}

impl MultiLevelQueue {

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

    // Inserts a new process to the front of the first queue
    pub fn insert_process(&mut self, process: StrongPcbRef) {
        if self.contains(&process) { panic!("Process already in queue") }
        self.top.borrow_mut().push_front(process)
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

        // Create 4 queues
        // Each of them have references to the queue above
        let top = Rc::new(RefCell::new(Queue {
            above: None,
            internal: Default::default(),
            below: None,
            quantum: 2
        }));
        let three = Rc::new(RefCell::new(Queue {
            above: Some(Rc::downgrade(&top)),
            internal: Default::default(),
            below: None,
            quantum: 4
        }));
        let two = Rc::new(RefCell::new(Queue {
            above: Some(Rc::downgrade(&three)),
            internal: Default::default(),
            below: None,
            quantum: 8
        }));
        let bottom = Rc::new(RefCell::new(Queue {
            above: Some(Rc::downgrade(&two)),
            internal: Default::default(),
            below: None,
            quantum: 16
        }));

        // Link the queues to the ones below, by iterating upwards
        let mut i = bottom;
        while i.borrow_mut().above.is_some() {
            let above = i.borrow().above.as_ref().map(|x| Weak::upgrade(x).unwrap()).unwrap();
            above.borrow_mut().below = Some(Rc::clone(&i));
            i = above;
        }

        MultiLevelQueue { top }
    }
}


pub struct MultiLevelQueueIterator {
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
