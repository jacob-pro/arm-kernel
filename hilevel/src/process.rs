use crate::Context;
use alloc::collections::BTreeMap;
use crate::device::PL011::UART0;
use core::fmt::Write;
use alloc::string::ToString;
use alloc::borrow::ToOwned;

pub type PID = u32;
const CPSR_USR: u32 = 0x50;

#[derive(Default)]
pub struct ProcessManager {
    // BTreeMap will be fast for ordered integer keys
    pub table: BTreeMap<PID, ProcessControlBlock>,
    pub executing: Option<PID>,
}

pub enum ProcessStatus {
    Ready,
    Executing,
}

pub enum ScheduleSource {
    Svc {id: u32},
    Timer,
    Reset,
}

pub struct ProcessControlBlock {
    pub pid: PID,
    pub status: ProcessStatus,
    pub top_of_stack: u32,
    pub context: Context,
}


impl ProcessManager {

    pub fn schedule(&mut self, ctx: &mut Context, src: ScheduleSource) {
        match self.executing {
            Some(x) => {

                if x == self.table[&0].pid {
                    self.dispatch( ctx, Some(0), 1 );  // context switch P_1 -> P_2

                } else if x == self.table[&1].pid {
                    self.dispatch( ctx, Some(1), 0 );  // context switch P_2 -> P_1
                }
            }
            None => {
                self.dispatch( ctx, None, 0 );  // context switch P_1 -> P_2
            }
        }
    }

    fn dispatch(&mut self, ctx: &mut Context, prev_pid: Option<PID>, next_pid: PID) {

        let prev_pid_str = match prev_pid {
            Some(x) => {
                let prev = self.table.get_mut(&x).unwrap();
                prev.context = *ctx;
                prev.status = ProcessStatus::Ready;
                x.to_string()
            },
            None => {
                "?".to_string()
            }
        };

        let next = self.table.get_mut(&next_pid).unwrap();
        *ctx = next.context;
        next.status = ProcessStatus::Executing;

        write!(UART0(), "[{}->{}]", prev_pid_str, next_pid).ok();

        self.executing = Some(next_pid); // update   executing process to P_{next}
    }

    pub fn create_process(&mut self, tos: *const cty::c_void, main: unsafe extern fn()) {

        let process = ProcessControlBlock {
            pid: self.new_pid(),
            status: ProcessStatus::Ready,
            top_of_stack: tos as u32,
            context: Context {
                cpsr: CPSR_USR,
                pc: main as u32,
                gpr: [0; 13],
                sp: tos as u32,
                lr: 0,
            }
        };

        self.table.insert(process.pid, process);
    }

    fn new_pid(&self) -> PID {
        match self.table.last_key_value().map(|x| x.0.to_owned()) {
            Some(x) => {
                if x < PID::MAX {
                    // Increment of current largest PID
                    return x + 1
                } else {
                    // Otherwise find first missing positive
                    for i in 0..PID::MAX {
                        if !self.table.contains_key(&i) { return i }
                    }
                    panic!("Process table full");
                }
            }
            // If no PIDs exist start at 0
            _ => {0}
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::process::ProcessManager;

    #[test]
    fn new_pid_test() {

        let pm = ProcessManager::default();
        assert_eq!(pm.new_pid(), 0);
        //assert_eq!(pm.new_pid(), 1);

    }
}
