//! SSA stands for Single Static Assignment
//! The IR presented in this module will already
//! be in SSA form and will be used to apply
//! conventional optimizations like Common Subexpression
//! elimination and constant folding.

#[allow(dead_code)]
mod basic_block;
#[allow(dead_code)]
mod ir;
