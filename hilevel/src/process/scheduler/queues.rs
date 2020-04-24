use core::ops;
use core::cell::RefCell;
use crate::process::{ProcessControlBlock, StrongPcbRef};
use alloc::rc::{Weak, Rc};
use alloc::collections::VecDeque;
use alloc::vec::Vec;

pub type StrongQueueLevelRef = Rc<RefCell<QueueLevel>>;

const QUEUE_QUANTUM: &[u32] = &[2, 4, 8, 16];

pub struct MultiLevelQueue {
    top: StrongQueueLevelRef
}

// Both above and below can't be strong otherwise there would be a reference cycle
pub struct QueueLevel {
    above: Option<Weak<RefCell<QueueLevel>>>,
    internal: VecDeque<StrongPcbRef>,
    below: Option<StrongQueueLevelRef>,
    quantum: u32,
}

pub trait LinkedQueues {
    fn below(&self) -> Option<StrongQueueLevelRef>;
    fn above(&self) -> Option<StrongQueueLevelRef>;
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
            queues.push(Rc::new(RefCell::new(QueueLevel {
                above: Some(Rc::downgrade(queues.last().unwrap())),
                internal: Default::default(),
                below: None,
                quantum,
            })));
        }
        // Link the queues to the ones below, by iterating upwards
        let mut i = Rc::clone(queues.last().unwrap());
        while i.borrow().above.is_some() {
            let above = LinkedQueues::above(&i).unwrap();
            above.borrow_mut().below = Some(Rc::clone(&i));
            i = above;
        }
        MultiLevelQueue { top: queues.remove(0) }
    }

    pub fn top_queue(&self) -> StrongQueueLevelRef {
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
    pub fn pop_process<F>(&mut self, filter: F) -> Option<(StrongPcbRef, StrongQueueLevelRef)>
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

impl LinkedQueues for StrongQueueLevelRef {

    fn below(&self) -> Option<StrongQueueLevelRef> {
        self.borrow().below.as_ref().map(|x| Rc::clone(x))
    }

    fn above(&self) -> Option<StrongQueueLevelRef> {
        self.borrow().above.as_ref().map(|x| Weak::upgrade(x).unwrap())
    }
}

impl ops::Deref for QueueLevel {
    type Target = VecDeque<StrongPcbRef>;
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

struct MultiLevelQueueIterator {
    start: StrongQueueLevelRef,
    current: Option<StrongQueueLevelRef>,
}

// Iterates through queues downwards
impl Iterator for MultiLevelQueueIterator {
    type Item = StrongQueueLevelRef;

    fn next(&mut self) -> Option<Self::Item> {
        self.current = match &self.current {
            None => {
                Some(Rc::clone(&self.start))
            },
            Some(x) => {
                LinkedQueues::below(x)
            },
        };
        self.current.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::process::scheduler::queues::{MultiLevelQueue, LinkedQueues};
    use alloc::rc::Rc;
    use alloc::vec::Vec;
    use core::cell::RefCell;
    use crate::process::{ProcessControlBlock, Context, StrongPcbRef};

    #[test]
    fn new_test() {
        let mlq = MultiLevelQueue::new(vec![1, 2, 3]);

        let top = mlq.top_queue();
        let middle = LinkedQueues::below(&top).unwrap();
        let bottom = LinkedQueues::below(&middle).unwrap();
        assert!(LinkedQueues::below(&bottom).is_none());

        assert_eq!(top.borrow().quantum, 1);
        assert_eq!(middle.borrow().quantum, 2);
        assert_eq!(bottom.borrow().quantum, 3);

        assert!(LinkedQueues::above(&top).is_none());
        assert!(Rc::ptr_eq(&LinkedQueues::above(&bottom).unwrap(), &middle));
        assert!(Rc::ptr_eq(&LinkedQueues::above(&middle).unwrap(), &top));
    }

    #[test]
    fn iter_test() {
        let qs = vec![1, 2, 3, 5, 9];
        let mlq = MultiLevelQueue::new(qs.clone());
        let iterated: Vec<u32> = mlq.iter().map(|q| {
            q.borrow().quantum
        }).collect();
        assert_eq!(qs, iterated);
    }

    #[test]
    fn contains_remove_test() {
        let item = Rc::new(RefCell::new(ProcessControlBlock::new(0, Vec::new(), Context::new(0, 0))));
        let mut mlq = MultiLevelQueue::new(vec![1, 2, 3]);
        assert_eq!(mlq.contains(&item), false);
        mlq.top_queue().borrow_mut().push_front(Rc::clone(&item));
        assert_eq!(mlq.contains(&item), true);
        mlq.remove_process(&item);
        assert_eq!(mlq.contains(&item), false);
        let middle = LinkedQueues::below(&mlq.top_queue()).unwrap();
        middle.borrow_mut().push_back(Rc::clone(&item));
        assert_eq!(mlq.contains(&item), true);
    }

    #[test]
    fn boost_test() {
        let item = Rc::new(RefCell::new(ProcessControlBlock::new(0, Vec::new(), Context::new(0, 0))));
        let mut mlq = MultiLevelQueue::new(vec![1, 2, 3]);
        let mut last_queue = mlq.iter().last().unwrap();
        last_queue.borrow_mut().push_back(Rc::clone(&item));
        mlq.boost();
        let (popped, queue) = mlq.pop_process(|x| true).unwrap();
        assert!(Rc::ptr_eq(&item, &popped));
        assert!(Rc::ptr_eq(&queue, &mlq.top_queue()));
    }

    #[test]
    fn pop_test() {
        let item1 = Rc::new(RefCell::new(ProcessControlBlock::new(0, Vec::new(), Context::new(0, 0))));
        let item2 = Rc::new(RefCell::new(ProcessControlBlock::new(0, Vec::new(), Context::new(0, 0))));
        let item3 = Rc::new(RefCell::new(ProcessControlBlock::new(0, Vec::new(), Context::new(0, 0))));
        let mut mlq = MultiLevelQueue::new(vec![1, 2, 3]);
        let mut last_queue = mlq.iter().last().unwrap();
        mlq.top_queue().borrow_mut().push_back(Rc::clone(&item1));
        mlq.top_queue().borrow_mut().push_back(Rc::clone(&item2));
        last_queue.borrow_mut().push_back(Rc::clone(&item3));
        fn filter(pcb: &ProcessControlBlock) -> bool { true };
        assert!(Rc::ptr_eq(&item1, &mlq.pop_process(filter).unwrap().0));
        assert!(Rc::ptr_eq(&item2, &mlq.pop_process(filter).unwrap().0));
        assert!(Rc::ptr_eq(&item3, &mlq.pop_process(filter).unwrap().0));
        assert!(mlq.pop_process(filter).is_none());
    }

}
