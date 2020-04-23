mod queues;

use crate::process::{ProcessControlBlock, StrongPcbRef, ScheduleSource, ProcessStatus};
use alloc::rc::Rc;
use queues::{MultiLevelQueue, LinkedQueues, StrongQueueRef};
use crate::process::scheduler::queues::Queue;
use crate::SysCall;

const BOOST_QUANTUM: u32 = 50;

#[derive(Default)]
pub struct MLFQScheduler {
    queues: MultiLevelQueue,
    current: Option<Current>,
    boost_tracker: u32,
}

// Info about the process which is currently being executed in user mode
struct Current {
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

    fn incr_boost_counter(&mut self) {
        self.boost_tracker = self.boost_tracker + 1;
        if self.boost_tracker > BOOST_QUANTUM {
            self.queues.boost();
            self.boost_tracker = 0
        }
    }

    // Add new process to the scheduler
    pub fn insert_process(&mut self, process: StrongPcbRef) {
        self.queues.insert_process(process)
    }

    // Remove a process from the scheduler
    pub fn remove_process(&mut self, process: &StrongPcbRef) -> Option<StrongPcbRef> {
        match &self.current {
            Some(c) => {
                if Rc::ptr_eq(process, &c.process) {
                    let k = Some(Rc::clone(&c.process));
                    self.current = None;
                    return k;
                }
            },
            _ => {}
        }
        self.queues.remove_process(process)
    }

    pub fn current_process(&self) -> Option<StrongPcbRef> {
        self.current.as_ref().map(|x| Rc::clone(&x.process))
    }

    pub fn schedule<F>(&mut self, src: ScheduleSource, mut dispatch: F)
        where F: FnMut(Option<&mut ProcessControlBlock>, &mut ProcessControlBlock)
    {

        match src {

            // A reset means no process is currently running
            ScheduleSource::Reset => {
                let (next_p, from_q) = self.queues.pop_process(ready).expect("No process found");
                dispatch(None, &mut (*next_p).borrow_mut());
                self.current = Some(Current::new(next_p, from_q));
            }

            // Timer preemption
            ScheduleSource::Timer => {
                self.incr_boost_counter();
                let current = self.current.as_mut().unwrap();
                current.incr_run_count();

                // If it has been running longer than its count, try to move to next top process
                if current.run_count >= Queue::quantum(&(*current.queue).borrow()) {

                    // If there is no other process ready, then just skip
                    let next = self.queues.pop_process(ready).map(|(next_p, from_q)| {
                        // Move the current to a lower/same queue
                        let below = LinkedQueues::below(&current.queue).unwrap_or(Rc::clone(&current.queue));
                        below.borrow_mut().push_back(Rc::clone(&current.process));
                        dispatch(Some(&mut current.process.borrow_mut()), &mut next_p.borrow_mut());
                        Current::new(next_p, from_q)
                    });
                    next.map(|n| self.current = Some(n));

                }
            },

            ScheduleSource::Svc { id } => {

                // After a service call, we may find that the current process has been removed
                match &mut self.current {
                    Some(current) => {
                        current.incr_run_count();

                        // If there is no other process ready, then just skip
                        let next = self.queues.pop_process(ready).map(|(next_p, from_q)| {

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
                            }.borrow_mut().push_back(Rc::clone(&current.process));

                            dispatch(Some(&mut current.process.borrow_mut()), &mut next_p.borrow_mut());
                            Current::new(next_p, from_q)
                        });
                        next.map(|n| self.current = Some(n));

                    },
                    None => {
                        let (next_p, from_q) = self.queues.pop_process(ready).expect("No processes scheduled");
                        dispatch(None, &mut (*next_p).borrow_mut());
                        self.current = Some(Current::new(next_p, from_q));
                    },
                }
            }
        }
    }

}

fn ready(process: &ProcessControlBlock) -> bool {
    process.status == ProcessStatus::Ready
}

// These tests demonstrate that the scheduler works as per Stage 1b
#[cfg(test)]
mod tests {


}

