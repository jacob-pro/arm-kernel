use alloc::collections::BTreeMap;
use crate::process::{PID, ProcessControlBlock};
use alloc::borrow::ToOwned;
use alloc::rc::Rc;
use core::cell::RefCell;

// BTreeMap will be fast for ordered integer keys
pub type ProcessTable = BTreeMap<PID, Rc<RefCell<ProcessControlBlock>>>;

pub trait ProcessTableMethods {
    fn new_pid(&self) -> PID;
}

impl ProcessTableMethods for ProcessTable {

    fn new_pid(&self) -> PID {
        match self.last_key_value().map(|x| x.0.to_owned()) {
            Some(x) => {
                if x < PID::MAX {
                    // Increment of current largest PID - Fast
                    return x + 1
                } else {
                    // Otherwise find first missing positive - Slower
                    for i in 0..PID::MAX {
                        if !self.contains_key(&i) { return i }
                    }
                    panic!("Process table full"); // 2^32 is a lot of processes
                }
            }
            // If no PIDs exist start at 0
            _ => {0}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::process::table::ProcessTable;
    use crate::process::table::ProcessTableMethods;
    use alloc::vec::Vec;
    use crate::process::{ProcessControlBlock, PID};
    use core::cell::RefCell;
    use alloc::rc::Rc;

    #[test]
    fn new_pid_test() {

        pub extern fn do_nothing() {}
        let stack = (0..1).collect::<Vec<u8>>();
        let pcb = ProcessControlBlock::new(0, stack, do_nothing);
        let pcb = Rc::new(RefCell::new(pcb));

        // An empty table first PID should be 0
        let mut table = ProcessTable::default();
        assert_eq!(table.new_pid(), 0);

        // A table with lowest PID 0, next should be 1
        table.insert(0, pcb.clone());
        assert_eq!(table.new_pid(), 1);

        // A table with lowest PID 5, next should be 6
        table.insert(5, pcb.clone());
        assert_eq!(table.new_pid(), 6);

        // A table that has been filled up, should loop back around
        table.insert(PID::MAX, pcb.clone());
        assert_eq!(table.new_pid(), 1);
    }
}
