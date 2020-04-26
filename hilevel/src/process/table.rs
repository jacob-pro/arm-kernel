use alloc::collections::BTreeMap;
use crate::process::{PID, ProcessControlBlock};
use alloc::borrow::ToOwned;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::ops;

// BTreeMap will be fast for ordered integer keys
type Internal = BTreeMap<PID, Rc<RefCell<ProcessControlBlock>>>;

#[derive(Default)]
pub struct ProcessTable(Internal);

impl ops::Deref for ProcessTable {
    type Target = Internal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for ProcessTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ProcessTable {

    pub fn new_pid(&self) -> PID {
        match self.0.last_key_value().map(|x| x.0.to_owned()) {
            Some(x) => {
                if x < PID::MAX {
                    // Increment of current largest PID - Fast
                    return x + 1
                } else {
                    // Otherwise find first missing positive - Slower
                    for i in 0..PID::MAX {
                        if !self.0.contains_key(&i) { return i }
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
    use alloc::vec::Vec;
    use crate::process::{ProcessControlBlock, PID, Context};
    use core::cell::RefCell;
    use alloc::rc::Rc;

    #[test]
    fn new_pid_test() {
        let stack = Vec::new();
        let pcb = ProcessControlBlock::new(0, stack, Context::new(0, 0));
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

        // A table that has been filled up, should loop back around and find first gap after 0
        table.insert(PID::MAX, pcb.clone());
        assert_eq!(table.new_pid(), 1);
    }
}
