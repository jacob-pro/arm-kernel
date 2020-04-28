mod queues;
mod idle;

use crate::process::{ProcessControlBlock, StrongPcbRef, ScheduleSource, ProcessStatus};
use alloc::rc::Rc;
use queues::{MultiLevelQueue, LinkedQueues, StrongQueueLevelRef};
use crate::process::scheduler::queues::QueueLevel;
use crate::SysCall;
use core::cell::RefCell;
use crate::process::scheduler::idle::idle_process;

const BOOST_QUANTUM: u32 = 50;

pub struct MLFQScheduler {
    queues: MultiLevelQueue,
    current: Option<Current>,
    boost_tracker: u32,
    idle_process: StrongPcbRef
}

impl Default for MLFQScheduler {
    fn default() -> Self {
        MLFQScheduler{
            queues: Default::default(),
            current: None,
            boost_tracker: 0,
            idle_process: Rc::new(RefCell::new(idle_process()))
        }
    }
}

// Info about the process which is currently being executed in user mode
struct Current {
    process: StrongPcbRef,
    // The queue that the process was taken from
    queue: StrongQueueLevelRef,
    // The number of time quantum the current process has already been running for
    run_count: u32
}

impl Current {

    fn new(process: StrongPcbRef, queue: StrongQueueLevelRef) -> Current {
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
        if self.queues.contains(&process) { panic!("Process already in scheduler") }
        self.queues.top_queue().borrow_mut().push_front(process)
    }

    // Remove a process from the scheduler, will return None if process == current_process()
    pub fn remove_process(&mut self, process: &StrongPcbRef) -> Option<StrongPcbRef> {
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
                if self.current.is_some() {
                    let current = self.current.as_mut().unwrap();
                    current.incr_run_count();

                    // If it has used up its run count, try to move to next top process
                    if current.run_count >= QueueLevel::quantum(&(*current.queue).borrow()) {

                        // Switch to next process only if one is ready
                        let next = self.queues.pop_process(ready).map(|(next_p, from_q)| {
                            // Move the current to a lower/same queue
                            let below = LinkedQueues::below(&current.queue).unwrap_or(Rc::clone(&current.queue));
                            below.borrow_mut().push_back(Rc::clone(&current.process));
                            dispatch(Some(&mut current.process.borrow_mut()), &mut next_p.borrow_mut());
                            Current::new(next_p, from_q)
                        });
                        next.map(|n| self.current = Some(n));
                    }
                }
            },

            ScheduleSource::Svc { id } => {
                // There must have been a current process to have made a service call
                let current = self.current.as_mut().unwrap();
                let current_status = (*current.process).borrow().status.clone();

                // Move current process back onto the MultiLevelQueue iff it is not terminated
                let move_current_back_to_queue = || {
                    if current_status != ProcessStatus::Terminated {
                        // If Sys Yield then move down queue
                        // If below max quantum count then move up queue
                        // Otherwise stay at same queue level
                        if id == SysCall::Yield {
                            LinkedQueues::below(&current.queue).unwrap_or(Rc::clone(&current.queue))
                        } else if current.run_count < QueueLevel::quantum(&(*current.queue).borrow()) {
                            LinkedQueues::above(&current.queue).unwrap_or(Rc::clone(&current.queue))
                        } else {
                            Rc::clone(&current.queue)
                        }.borrow_mut().push_back(Rc::clone(&current.process));
                    }
                };

                // Try to move to the next top process, if one exists
                let next = self.queues.pop_process(ready).map(|(next_p, from_q)| {
                    move_current_back_to_queue();
                    dispatch(Some(&mut current.process.borrow_mut()), &mut next_p.borrow_mut());
                    Current::new(next_p, from_q)
                });

                // If there are no new processes and this one is no longer executing then we must idle
                if next.is_none() && current_status != ProcessStatus::Executing {
                    move_current_back_to_queue();
                    dispatch(Some(&mut current.process.borrow_mut()), &mut (self.idle_process.borrow_mut()));
                    self.current = None;
                }

                // If we were able to switch to a new process, then update current
                next.map(|n| self.current = Some(n));
            }

            ScheduleSource::Io => {
                // Once IO has completed we may no longer need to idle
                if self.current.is_none() {
                    let next = self.queues.pop_process(ready).map(|(next_p, from_q)| {
                        dispatch(Some(&mut (self.idle_process.borrow_mut())), &mut (*next_p).borrow_mut());
                        Current::new(next_p, from_q)
                    });
                    next.map(|x| { self.current = Some(x); });
                }

            }
        }
    }
}

fn ready(process: &ProcessControlBlock) -> bool {
    process.status == ProcessStatus::Ready
}
