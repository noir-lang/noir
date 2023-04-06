//! SSA stands for Single Static Assignment
//! This module will convert the monomorphized AST
//! into an SSA IR. This IR will consist of basic blocks and
//! a control flow graph.

mod basic_block;
mod ir;
