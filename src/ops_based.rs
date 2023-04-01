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
