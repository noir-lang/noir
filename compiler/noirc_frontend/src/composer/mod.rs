//! The Composer's job is to recur on the Ast, calling name resolution,
//! type checking, and the comptime interpreter in lock-step one node at a time.
//! It accomplishes this with the `functor::Ast` which does not have sub-nodes
//! that these passes can recur upon. Instead, it has generic slots for result values.
//!
//! The Composer's job then is to just recur on the raw Ast, create a `functor::Ast`
//! for that node holding results from any recursive calls, and hand that off to each pass.
mod functor;
