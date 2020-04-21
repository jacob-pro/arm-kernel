mod queues;

use crate::process::{ProcessControlBlock, WeakPcbRef, StrongPcbRef, ScheduleSource, ProcessStatus};
use alloc::rc::Rc;
use queues::{MultiLevelQueue, LinkedQueues, StrongQueueRef};
use crate::process::scheduler::queues::Queue;
use core::cell::{RefMut, Ref};
use crate::SysCall;


#[derive(Default)]
pub struct MLFQScheduler {
    queues: MultiLevelQueue,
    current: Option<Current>,
}

// Info about the process which is currently being executed in user mode
struct Current {
    // Keep strong reference to current process, if deleted it will deallocate once we start executing something else
    process: StrongPcbRef,
    // The queue that the process was taken from
    queue: StrongQueueRef,
    // The number of time quantum the current process has already been running for
    run_count: u32
}

impl Current {

    fn new(process: StrongPcbRef, queue: StrongQueueRef) -> Current {
        Current {
            process,
            queue,
            run_count: 0
        }
    }

    fn incr_run_count(&mut self) {
        self.run_count = self.run_count.saturating_add(1);         // Don't overflow
    }
}

impl MLFQScheduler {

    // Add new process to top queue, scheduler does not keep ownership
    pub fn insert_process(&mut self, process: WeakPcbRef) {
        self.queues.top_queue().borrow_mut().push_back(process)
    }


    pub fn schedule<F>(&mut self, src: ScheduleSource, mut dispatch: F)
        where F: FnMut(Option<RefMut<ProcessControlBlock>>, RefMut<ProcessControlBlock>)
    {

        match src {

            // A reset means no process is currently running
            ScheduleSource::Reset => {
                let (next_p, from_q) = self.queues.first_process(ready).expect("No process found");
                dispatch(None, (*next_p).borrow_mut());
                self.current = Some(Current::new(next_p, from_q));
            }

            // Timer preemption
            ScheduleSource::Timer => {
                let current = self.current.as_mut().unwrap();
                current.incr_run_count();

                // If it has been running longer than its count, try to move to next top process
                if current.run_count >= Queue::quantum(&(*current.queue).borrow()) {

                    // If there is no other process ready, then just skip
                    let next = self.queues.first_process(ready).map(|(next_p, from_q)| {
                        // Move the current to a lower/same queue
                        let below = LinkedQueues::below(&current.queue).unwrap_or(Rc::clone(&current.queue));
                        below.borrow_mut().push_back(Rc::downgrade(&current.process));
                        dispatch(Some((*current.process).borrow_mut()), (*next_p).borrow_mut());
                        Some(Current::new(next_p, from_q))
                    });
                    next.map(|n| self.current = n);

                }
            },

            ScheduleSource::Svc { id } => {
                let current = self.current.as_mut().unwrap();
                current.incr_run_count();

                // Move current process back onto the MultiLevelQueue
                // If Sys Yield then move down queue
                // If below max quantum count then move up queue
                // Otherwise stay at same queue level
                if id == SysCall::Yield {
                    LinkedQueues::below(&current.queue).unwrap_or(Rc::clone(&current.queue))
                } else if current.run_count < Queue::quantum(&(*current.queue).borrow()) {
                    LinkedQueues::above(&current.queue).unwrap_or(Rc::clone(&current.queue))
                } else {
                    Rc::clone(&current.queue)
                }.borrow_mut().push_back(Rc::downgrade(&current.process));

                // Dispatch the next process
                let next = self.queues.first_process(ready).unwrap();
                dispatch(Some((*current.process).borrow_mut()), (*next.0).borrow_mut());
                self.current = Some(Current::new(next.0, next.1));
            }
        }
    }

    pub fn current_process(&self) -> StrongPcbRef {
        self.current.as_ref().map(|x| Rc::clone(&x.process)).unwrap()
    }

}

fn ready(process: Ref<ProcessControlBlock>) -> bool {
    process.status == ProcessStatus::Ready
}

// These tests demonstrate that the scheduler works as per Stage 1b
#[cfg(test)]
mod tests {


}

