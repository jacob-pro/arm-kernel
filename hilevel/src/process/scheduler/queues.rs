use core::ops;
use crate::util::WeakQueue;
use core::cell::RefCell;
use crate::process::ProcessControlBlock;
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
    top: Rc<RefCell<Queue>>
}

impl MultiLevelQueue {
    pub fn top_queue(&self) -> StrongQueueRef {
        Rc::clone(&self.top)
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

        // Link the queues to the ones below
        let mut i = bottom;
        while i.borrow_mut().above.is_some() {
            let above = i.borrow().above.as_ref().map(|x| Weak::upgrade(x).unwrap()).unwrap();
            above.borrow_mut().below = Some(Rc::clone(&i));
            i = above;
        }

        MultiLevelQueue { top }
    }
}
