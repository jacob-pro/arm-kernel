use crate::process::ProcessManager;
use crate::io::PL011::UART0;
use core::fmt::Write;
use crate::io::IoManager;

#[derive(Default)]
pub struct KernelState {
    pub process_manager: ProcessManager,
    pub io_manager: IoManager,
}

// Mutable statics are treated as unsafe because the compiler does not aware of any
// synchronisation to prevent concurrent access. However this should not be an issue because
// the kernel is only executed from interrupts, which are not configured to execute concurrently
// https://www.ole.bris.ac.uk/webapps/discussionboard/do/message?action=list_messages&course_id=_237259_1&nav=discussion_board_entry&conf_id=_228003_1&forum_id=_208813_1&message_id=_619372_1

static mut KERNEL_STATE: Option<KernelState> = None;

pub fn init() -> &'static mut KernelState {
    writeln!(UART0(), "Initialising kernel state").ok();
    unsafe {
        if KERNEL_STATE.is_some() { panic!("State has already initialised") }
        KERNEL_STATE = Some(KernelState::default())
    }
    get()
}

pub fn get() -> &'static mut KernelState {
    unsafe { KERNEL_STATE.as_mut().unwrap() }
}
