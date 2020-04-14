use crate::process::{ProcessControlBlock, WeakPcbRef, StrongPcbRef, ScheduleSource, ProcessStatus};
use alloc::rc::{Rc, Weak};
use alloc::collections::LinkedList;
use core::cell::RefMut;

const NUMBER_OF_QUEUES: usize = 8;

#[derive(Default)]
pub struct MLFQ {
    queues: [LinkedList<WeakPcbRef>; NUMBER_OF_QUEUES],
    pub executing: Option<StrongPcbRef>,
}


impl MLFQ {

    pub fn insert_process(&mut self, process: WeakPcbRef) {
        // Add new process to top queue
        self.queues[0].push_back(process);
    }

    pub fn schedule<F>(&mut self, _src: ScheduleSource, mut dispatch: F)
        where F: FnMut(Option<RefMut<ProcessControlBlock>>, RefMut<ProcessControlBlock>)
    {
        let next = Weak::upgrade(&self.queues[0].pop_front().unwrap()).unwrap();
        self.queues[0].push_back(Rc::downgrade(&next));

        match &self.executing {
            Some(x) => {
                let prev = Rc::clone(x);
                dispatch(Some((*prev).borrow_mut()), (*next).borrow_mut());
            },
            None => {
                dispatch(None, (*next).borrow_mut());
            }
        }

        self.executing = Some(next); // update   executing process to P_{next}
    }

    fn pop_front(&mut self, queue: &mut LinkedList<WeakPcbRef>) -> Option<StrongPcbRef> {
        while !queue.is_empty() {
            let popped = queue.pop_front().unwrap();
            match Weak::upgrade(&popped) {
                Some(strong) => {return Some(strong)}
                _ => {}
            }
        }
        None
    }

}

