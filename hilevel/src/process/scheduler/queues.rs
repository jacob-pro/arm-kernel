use core::ops;
use crate::util::WeakQueue;
use core::cell::RefCell;
use crate::process::{ProcessControlBlock, StrongPcbRef};
use alloc::rc::{Weak, Rc};

pub type StrongQueueRef = Rc<RefCell<Queue>>;
type QueueInternal = WeakQueue<RefCell<ProcessControlBlock>>;

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

    pub fn top_queue(&self) -> StrongQueueRef {
        Rc::clone(&self.top)
    }

    pub fn iter(&self) -> MultiLevelQueueIterator {
        MultiLevelQueueIterator {start: Rc::clone(&self.top), current: None }
    }

    // Search queues for process
    pub fn first_process<F>(&mut self, filter: F) -> Option<(StrongPcbRef, StrongQueueRef)>
        where F: Fn(&ProcessControlBlock)->bool
    {
        // Iterate from High to Lower queues


        let mut queue_ref = self.top_queue();
        loop {
            let mut queue = queue_ref.borrow_mut();
            for _ in 0..queue.len() {
                let popped = queue.pop_front().unwrap();
                if filter(&popped.borrow()) {
                    return Some((popped, Rc::clone(&queue_ref)))
                } else {
                    queue.push_back(Rc::downgrade(&popped));
                }
            }
            drop(queue);
            let below = LinkedQueues::below(&queue_ref);
            match below {
                Some(x) => {queue_ref = x},
                None => {return None},  // There are no lower queues to search
            }
        }
    }

    // Moves all processes to the top queue
    pub fn boost(&mut self) {
        for queue in self.iter().skip(1) {
            let x = &mut queue.borrow_mut().internal;
            self.top.borrow_mut().internal.append(x)
        }
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
