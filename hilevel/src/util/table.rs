use alloc::collections::BTreeMap;
use alloc::borrow::ToOwned;
use core::ops;
use num::{Integer, Bounded};
use num_traits::FromPrimitive;
use core::iter::Step;

// A table of Integers keys to V, which can automatically generate new keys
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
    pub fn new_key(&self) -> Option<K> {
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
    use alloc::vec::Vec;
    use alloc::string::String;
    use alloc::borrow::ToOwned;
    use crate::util::IdTable;

    #[test]
    fn new_key_test() {
        let object = "Hello".to_owned();

        // An empty table first id should be 0
        let mut table: IdTable<i32, String> = IdTable::default();
        assert_eq!(table.new_key().unwrap(), 0);

        // A table with lowest id 0, next should be 1
        table.insert(0, object.clone());
        assert_eq!(table.new_key().unwrap(), 1);

        // A table with lowest id 5, next should be 6
        table.insert(5, object.clone());
        assert_eq!(table.new_key().unwrap(), 6);

        // A table that has been filled up, should loop back around and find first gap after 0
        table.insert(i32::MAX, object.clone());
        assert_eq!(table.new_key().unwrap(), 1);
    }
}
