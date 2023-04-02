//! Operation-based object specs
//!
//! ```txt
//! payload Payload type; instantiated at all replicas
//!   initial Initial value
//! query Source-local operation (arguments) : returns
//!   pre Precondition
//!   let Execute at source, synchronously, no side effects
//! update Global update (arguments) : returns
//!   atSource (arguments) : returns
//!     pre Precondition at source
//!     let 1st phase: synchronous, at source, no side effects
//!   downstream (arguments passed downstream)
//!     pre Precondition against downstream state
//!     2nd phase, asynchronous, side-effects to downstream state
//! ```

pub trait OpsBased<T> {
    type Query: FnOnce(&T) -> Option<T>;
    type Args;
    type AtSource: FnOnce(&mut T, &Self::Args) -> Option<T>;
    type Downstream: FnOnce(&mut T, &Self::Args);
    type Error;

    fn query(&self, query: Self::Query) -> Result<Option<T>, Self::Error>;

    fn update(
        &mut self,
        args: &Self::Args,
        at_source: Self::AtSource,
        downstream: Self::Downstream,
    ) -> Result<Option<T>, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload<T> {
    initial: T,
}

impl<T> Payload<T> {
    pub fn new(initial: T) -> Self {
        Self { initial }
    }
}

impl<T> Payload<T>
where
    T: OpsBased<T>,
{
    pub fn query(&self, query: T::Query) -> Result<Option<T>, T::Error> {
        self.initial.query(query)
    }

    pub fn update(
        &mut self,
        args: &T::Args,
        at_source: T::AtSource,
        downstream: T::Downstream,
    ) -> Result<Option<T>, T::Error> {
        let res = at_source(&mut self.initial, args);
        downstream(&mut self.initial, args);
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use std::convert::Infallible;

    use super::*;

    impl OpsBased<i32> for i32 {
        type Query = fn(&i32) -> Option<i32>;
        type Args = i32;
        type AtSource = fn(&mut i32, &Self::Args) -> Option<i32>;
        type Downstream = fn(&mut i32, &Self::Args);
        type Error = Infallible;

        fn query(&self, query: Self::Query) -> Result<Option<i32>, Self::Error> {
            Ok(query(self))
        }

        fn update(
            &mut self,
            args: &Self::Args,
            at_source: Self::AtSource,
            downstream: Self::Downstream,
        ) -> Result<Option<i32>, Self::Error> {
            let res = at_source(self, args);
            downstream(self, args);
            Ok(res)
        }
    }

    #[test]
    fn test_query() {
        let payload = Payload::new(0);
        let res = payload.query(|x| Some(x + 1)).unwrap();
        assert_eq!(res, Some(1));
    }

    #[test]
    fn test_update() {
        let mut payload = Payload::new(0);
        let at_source: <i32 as OpsBased<i32>>::AtSource = |x, y| Some(*x + *y);
        let downstream: <i32 as OpsBased<i32>>::Downstream = |x, y| *x += *y;
        let res = payload.update(&1, at_source, downstream).unwrap().unwrap();
        assert_eq!(res, 1);
    }
}
