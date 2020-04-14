use crate::process::{ProcessControlBlock, WeakPcbRef, StrongPcbRef, ScheduleSource, ProcessStatus};
use alloc::rc::{Rc, Weak};
use alloc::collections::LinkedList;
use core::cell::{RefMut, RefCell};
use crate::util::WeakQueue;

const NUMBER_OF_QUEUES: usize = 8;

#[derive(Default)]
pub struct MLFQ {
    queues: [WeakQueue<RefCell<ProcessControlBlock>>; NUMBER_OF_QUEUES],
    pub executing: Option<StrongPcbRef>,
}


impl MLFQ {

    // Add new process to top queue, does not take ownership
    pub fn insert_process(&mut self, process: WeakPcbRef) {
        self.queues[0].push_back(process);
    }

    pub fn schedule<F>(&mut self, _src: ScheduleSource, mut dispatch: F)
        where F: FnMut(Option<RefMut<ProcessControlBlock>>, RefMut<ProcessControlBlock>)
    {
        let next = self.queues[0].pop_front().unwrap();
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

}

