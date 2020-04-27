use alloc::collections::BTreeMap;
use crate::process::{PID, ProcessControlBlock};
use alloc::borrow::ToOwned;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::ops;
use num::{Integer, Bounded};
use num_traits::FromPrimitive;
use core::iter::Step;

// BTreeMap will be fast for ordered integer keys
pub struct IdTable<K, V>(BTreeMap<K, V>);

impl <K, V> ops::Deref for IdTable<K, V>  {
    type Target = BTreeMap<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl <K, V> ops::DerefMut for IdTable<K, V>  {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl <K, V> Default for IdTable<K, V>
    where K: Integer + Bounded + FromPrimitive + Step
{
    fn default() -> Self {
        IdTable(BTreeMap::new())
    }
}

impl <K, V> IdTable<K, V>
    where K: Integer + Bounded + FromPrimitive + Step
{
    pub fn new_pid(&self) -> Option<K> {
        match self.0.last_key_value().map(|x| x.0.to_owned()) {
            Some(x) => {
                if x < K::max_value() {
                    // Increment of current largest PID - Fast
                    return Some(x.add_one())
                } else {
                    // Otherwise find first missing positive - Slower
                    for i in K::from_u32(0).unwrap()..K::max_value() {
                        if !self.0.contains_key(&i) { return Some(i) }
                    }
                    return None // Table full
                }
            }
            // If no PIDs exist start at 0
            _ => { Some(K::zero())}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::process::table::IdTable;
    use alloc::vec::Vec;
    use crate::process::{ProcessControlBlock, PID, Context, StrongPcbRef};
    use core::cell::RefCell;
    use alloc::rc::Rc;

    #[test]
    fn new_pid_test() {
        let stack = Vec::new();
        let pcb = ProcessControlBlock::new(0, stack, Context::new(0, 0));
        let pcb = Rc::new(RefCell::new(pcb));

        // An empty table first id should be 0
        let mut table: IdTable<PID, StrongPcbRef> = IdTable::default();
        assert_eq!(table.new_pid().unwrap(), 0);

        // A table with lowest id 0, next should be 1
        table.insert(0, pcb.clone());
        assert_eq!(table.new_pid().unwrap(), 1);

        // A table with lowest id 5, next should be 6
        table.insert(5, pcb.clone());
        assert_eq!(table.new_pid().unwrap(), 6);

        // A table that has been filled up, should loop back around and find first gap after 0
        table.insert(PID::MAX, pcb.clone());
        assert_eq!(table.new_pid().unwrap(), 1);
    }
}
