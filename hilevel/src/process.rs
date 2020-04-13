use crate::Context;
use alloc::collections::BTreeMap;
use crate::device::PL011::UART0;
use core::fmt::Write;
use crate::alloc::string::ToString;

pub type PID = u32;
const CPSR_USR: u32 = 0x50;

#[derive(Default)]
pub struct ProcessManager {
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

                if x == self.table[&1].pid {
                    self.dispatch( ctx, Some(1), 2 );  // context switch P_1 -> P_2

                } else if x == self.table[&2].pid {
                    self.dispatch( ctx, Some(2), 1 );  // context switch P_2 -> P_1
                }
            }
            None => {
                self.dispatch( ctx, None, 1 );  // context switch P_1 -> P_2
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

    pub fn create_process(&mut self, pid: PID, tos: *const cty::c_void, main: unsafe extern fn()) {

        self.table.insert(pid, ProcessControlBlock {
            pid: pid,
            status: ProcessStatus::Ready,
            top_of_stack: tos as u32,
            context: Context {
                cpsr: CPSR_USR,
                pc: main as u32,
                gpr: [0; 13],
                sp: tos as u32,
                lr: 0,
            }
        });


    }

}

