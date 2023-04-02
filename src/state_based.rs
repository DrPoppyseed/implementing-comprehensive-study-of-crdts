//! State-based object specs
//!
//! ```txt
//! payload Payload type; instantiated at all replicas
//!   initial Initial value
//! query Query (arguments) : returns
//!   pre Precondition
//!   let Evaluate synchronously, no side effects
//! update Source-local operation (arguments) : returns
//!   pre Precondition
//!   let Evaluate at source, synchronously
//!   Side-effects at source to execute synchronously
//! compare (value1, value2) : boolean b
//!   Is value1 ≤ value2 in semilattice?
//! merge (value1, value2) : payload mergedValue
//!   LUB merge of value1 and value2, at any replica
//! ```

pub trait Semilattice {
    fn compare(&self, other: &Self) -> bool;

    fn merge(&self, other: &Self) -> Self;
}

pub trait StateBased<T> {
    type Query: FnOnce(&T) -> Option<T>;
    type Update: FnOnce(&mut T) -> Option<T>;
    type Error;

    fn query(&self, query: Self::Query) -> Result<Option<T>, Self::Error>;

    fn update(&mut self, update: Self::Update) -> Result<Option<T>, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload<T> {
    initial: T,
}

impl<T> Payload<T>
where
    T: Semilattice + StateBased<T>,
{
    pub fn query(&self, query: T::Query) -> Result<Option<T>, T::Error> {
        Ok(query(&self.initial))
    }

    pub fn update(&mut self, update: T::Update) -> Result<Option<T>, T::Error> {
        Ok(update(&mut self.initial))
    }
}

#[cfg(test)]
mod tests {
    use std::{cmp::max, convert::Infallible};

    use super::*;

    impl Semilattice for i32 {
        fn compare(&self, other: &Self) -> bool {
            self <= other
        }

        fn merge(&self, other: &Self) -> Self {
            max(*self, *other)
        }
    }

    impl StateBased<i32> for i32 {
        type Query = fn(&i32) -> Option<i32>;
        type Update = fn(&mut i32) -> Option<i32>;
        type Error = Infallible;

        fn query(&self, query: Self::Query) -> Result<Option<i32>, Self::Error> {
            Ok(query(self))
        }

        fn update(&mut self, update: Self::Update) -> Result<Option<i32>, Self::Error> {
            Ok(update(self))
        }
    }

    #[test]
    fn test_compare() {
        let value1: i32 = 1;
        let value2: i32 = 2;
        assert!(value1.compare(&value2));
        assert!(!value2.compare(&value1));
    }

    #[test]
    fn test_merge() {
        let value1: i32 = 2;
        let value2: i32 = 4;
        assert_eq!(value1.merge(&value2), 4);
        assert_eq!(value2.merge(&value1), 4);
    }

    #[test]
    fn test_query() {
        let payload = Payload { initial: 1 };
        let query = |value: &i32| Some(*value + 1);
        assert_eq!(payload.query(query).unwrap().unwrap(), 2);
    }

    #[test]
    fn test_update() {
        let mut payload = Payload { initial: 1 };
        let update = |value: &mut i32| {
            *value += 1;
            Some(*value)
        };
        assert_eq!(payload.update(update).unwrap().unwrap(), 2);
        assert_eq!(payload.initial, 2);
    }
}
