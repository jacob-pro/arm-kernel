use crate::process::{ProcessControlBlock, WeakPcbRef, StrongPcbRef, ScheduleSource, ProcessStatus};
use alloc::rc::{Rc, Weak};
use alloc::collections::LinkedList;
use core::cell::{RefMut, RefCell};
use crate::util::WeakQueue;

const NUMBER_OF_QUEUES: usize = 4;
const QUEUE_QUANTUM: &'static [i32] = &[16, 8, 4, 2];

#[derive(Default)]
pub struct MLFQ {
    queues: [WeakQueue<RefCell<ProcessControlBlock>>; NUMBER_OF_QUEUES],
    pub executing: Option<StrongPcbRef>,
}

impl MLFQ {

    // Add new process to top queue, does not take ownership
    pub fn insert_process(&mut self, process: WeakPcbRef) {
        self.queues[NUMBER_OF_QUEUES - 1].push_back(process);
    }

    // Get the next process from the queues
    fn next_ready(&mut self) -> StrongPcbRef {
        // Iterate from High to Lower queues
        for queue in (0..NUMBER_OF_QUEUES).rev() {
            let queue = &mut self.queues[queue];
            // Search the queue for a Ready Process
            for _ in 0..queue.len() {
                let popped = queue.pop_front().unwrap();
                if (*popped).borrow_mut().status == ProcessStatus::Ready {
                    return popped
                } else {
                    queue.push_back(Rc::downgrade(&popped));
                }
            }
        }
        panic!("No processes to execute")
    }

    pub fn schedule<F>(&mut self, src: ScheduleSource, mut dispatch: F)
        where F: FnMut(Option<RefMut<ProcessControlBlock>>, RefMut<ProcessControlBlock>)
    {
        let next = self.next_ready();
        self.queues[0].push_back(Rc::downgrade(&next));

        // match src {
        //     ScheduleSource::Reset => {
        //
        //     }
        //     ScheduleSource::Timer => {
        //         let current = self.executing.unwrap();
        //
        //     },
        //     ScheduleSource::Svc { id: id } => {
        //         let current = self.executing.unwrap();
        //
        //     }
        // }

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

#[cfg(test)]
mod tests {

}

