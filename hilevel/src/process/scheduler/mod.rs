mod queues;

use crate::process::{ProcessControlBlock, WeakPcbRef, StrongPcbRef, ScheduleSource, ProcessStatus};
use alloc::rc::Rc;
use queues::{MultiLevelQueue, LinkedQueues, StrongQueueRef};
use crate::process::scheduler::queues::Queue;
use core::cell::RefMut;


#[derive(Default)]
pub struct MLFQScheduler {
    queues: MultiLevelQueue,
    current: Option<Current>,
}

struct Current {
    process: StrongPcbRef,
    queue: StrongQueueRef,
    run_count: u32
}

impl Current {

    fn new(process: StrongPcbRef, queue: StrongQueueRef) -> Current {
        Current {
            process,
            queue,
            run_count: 1
        }
    }

    fn incr_run_count(&mut self) {
        self.run_count = self.run_count + 1;
    }
}

impl MLFQScheduler {

    // Add new process to top queue, does not take ownership
    pub fn insert_process(&mut self, process: WeakPcbRef) {
        self.queues.top_queue().borrow_mut().push_back(process)
    }

    //Get the next process from the queues
    fn next_ready(&mut self) -> (StrongPcbRef, StrongQueueRef) {

        // Iterate from High to Lower queues
        let mut queue_ref = self.queues.top_queue();
        loop {
            let mut borrow = queue_ref.borrow_mut();
            for _ in 0..borrow.len() {
                let popped = borrow.pop_front().unwrap();
                if popped.clone().borrow_mut().status == ProcessStatus::Ready {
                    return (popped, Rc::clone(&queue_ref))
                } else {
                    borrow.push_back(Rc::downgrade(&popped));
                }
            }
            drop(borrow);
            queue_ref = LinkedQueues::below(&queue_ref.clone()).expect("No process to execute");
        }
    }

    pub fn schedule<F>(&mut self, src: ScheduleSource, mut dispatch: F)
        where F: FnMut(Option<RefMut<ProcessControlBlock>>, RefMut<ProcessControlBlock>)
    {

        match src {

            // A reset means no process is currently running
            ScheduleSource::Reset => {
                let next = self.next_ready();
                dispatch(None, (*next.0).borrow_mut());
                self.current = Some(Current::new(next.0, next.1));
            }

            // Timer preemption
            ScheduleSource::Timer => {
                let current = self.current.as_mut().unwrap();
                let current_process = Rc::clone(&current.process);

                // If it is allowed to run for more time don't stop it, just increment count
                if current.run_count < Queue::quantum(&(*current.queue).borrow()) {
                    current.incr_run_count();
                } else {
                    // Otherwise move to lower queue
                    let below = LinkedQueues::below(&current.queue).unwrap_or(Rc::clone(&current.queue));
                    below.borrow_mut().push_back(Rc::downgrade(&current.process));
                    // And dispatch the next top process
                    let next = self.next_ready();
                    dispatch(Some((*current_process).borrow_mut()), (*next.0).borrow_mut());
                    self.current = Some(Current::new(next.0, next.1));
                }
            },

            ScheduleSource::Svc { id: id } => {
                let current = self.current.as_mut().unwrap();
                let current_process = Rc::clone(&current.process);

                // If below max quantum count, and not SYS_YIELD then promote, otherwise stay in same queue
                if current.run_count < Queue::quantum(&(*current.queue).borrow()) {
                    LinkedQueues::above(&current.queue).unwrap_or(Rc::clone(&current.queue))
                } else {
                    Rc::clone(&current.queue)
                }.borrow_mut().push_back(Rc::downgrade(&current.process));

                // Dispatch the next top process
                let next = self.next_ready();
                dispatch(Some((*current_process).borrow_mut()), (*next.0).borrow_mut());
                self.current = Some(Current::new(next.0, next.1));
            }
        }
    }

    pub fn current_process(&self) -> StrongPcbRef {
        self.current.as_ref().map(|x| x.process.clone()).unwrap()
    }

}

#[cfg(test)]
mod tests {

}

