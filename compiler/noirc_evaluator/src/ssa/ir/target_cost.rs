//! Brillig target cost estimation for SSA IR types.
//!
//! Provides cost methods on [BinaryOp], [Instruction], [TerminatorInstruction],
//! and [Function] that estimate Brillig opcode counts. These are useful for any
//! pass that needs to reason about Brillig code size (inlining, optimization ordering, etc.).
//!
//! The estimates are approximations — accurate costs would require full Brillig codegen.
//! If ACIR cost estimation is needed in the future, it can be added here alongside the
//! Brillig estimates.

use super::{
    dfg::DataFlowGraph,
    function::Function,
    instruction::{BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction},
    types::NumericType,
    value::Value,
};

impl Instruction {
    /// Whether this instruction can be safely duplicated into both branches
    /// when flattening a Brillig conditional (`basic_conditional` pass).
    ///
    /// Instructions with side effects (constraints, calls, memory ops) cannot be
    /// flattened because they would execute unconditionally in the merged block.
    /// A few instructions that report side effects are still safe in Brillig conditionals.
    ///  These instructions are expected to be handled by this method's caller:
    /// - `Allocate`, `IncrementRc`, `DecrementRc` are not predicate-dependent.
    ///
    /// Div/Mod and Shl/Shr are blocked unconditionally — even when `has_side_effects`
    /// would allow them (e.g. known non-zero divisor), they are rarely worth flattening.
    pub(crate) fn can_flatten_in_conditional(&self, dfg: &DataFlowGraph) -> bool {
        match self {
            Instruction::EnableSideEffectsIf { .. } => {
                if dfg.runtime().is_brillig() {
                    panic!("ICE: Instruction is expected to only be emitted in ACIR");
                } else {
                    true
                }
            }
            Instruction::Allocate
            | Instruction::IncrementRc { .. }
            | Instruction::DecrementRc { .. } => {
                panic!("ICE: Caller should handle memory ops");
            }

            // Calls are never worth flattening — even pure intrinsics can expand
            // into many Brillig opcodes, making unconditional execution expensive.
            Instruction::Call { .. } => false,

            Instruction::Binary(binary) => match binary.operator {
                BinaryOp::Div | BinaryOp::Mod | BinaryOp::Shl | BinaryOp::Shr => false,
                _ => !self.has_side_effects(dfg),
            },

            _ => !self.has_side_effects(dfg),
        }
    }
}

impl BinaryOp {
    /// Estimate the Brillig opcode cost of this binary operation given the operand type.
    ///
    /// Field operations are single opcodes. Checked unsigned operations are more expensive
    /// (e.g., checked add = add + lt_eq + constrain = 3 opcodes). Unchecked integer operations
    /// are single opcodes. Signed operations that reach Brillig are always unchecked.
    pub(crate) fn cost(&self, typ: NumericType) -> usize {
        match self {
            BinaryOp::Add { unchecked } | BinaryOp::Sub { unchecked } => {
                if *unchecked || typ.is_field() {
                    1
                } else {
                    // checked unsigned: op + lt_eq + constrain
                    3
                }
            }
            BinaryOp::Mul { unchecked } => {
                if *unchecked || typ.is_field() {
                    1
                } else {
                    // checked unsigned mul is expensive
                    8
                }
            }
            BinaryOp::Div | BinaryOp::Mod => {
                if typ.is_field() {
                    // Field div is a single opcode; field mod doesn't exist but cost 1 as fallback
                    1
                } else {
                    // Unsigned: div=1, mod=div+mul+sub=3
                    match self {
                        BinaryOp::Div => 1,
                        BinaryOp::Mod => 3,
                        _ => unreachable!(),
                    }
                }
            }
            BinaryOp::Eq | BinaryOp::Lt => 1,
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => 1,
            BinaryOp::Shl | BinaryOp::Shr => 1,
        }
    }
}

impl Instruction {
    /// Estimate the Brillig opcode cost of this instruction.
    ///
    /// These estimates are type-aware: Field operations are typically cheaper than
    /// checked integer operations because they don't need overflow checks.
    pub(crate) fn cost(&self, id: InstructionId, dfg: &DataFlowGraph) -> usize {
        match self {
            Instruction::Binary(binary) => {
                let typ = dfg.type_of_value(binary.lhs).unwrap_numeric();
                binary.operator.cost(typ)
            }
            // A Cast can be either simplified, or lead to a truncate
            Instruction::Cast(_, _) => 3,
            Instruction::Not(_) => 1,
            Instruction::Truncate { .. } => 7,

            Instruction::Constrain(..) => {
                // TODO: could put estimate cost for static or dynamic message. Just checking static at the moment
                4
            }

            // TODO: Only implemented in ACIR, probably just error here but right we compute costs of all functions
            Instruction::ConstrainNotEqual(..) => 1,

            // TODO: look into how common this is in Brillig, just return one for now
            Instruction::RangeCheck { .. } => 1,

            Instruction::Call { func, arguments } => {
                match dfg[*func] {
                    Value::Function(_) => {
                        let results = dfg.instruction_results(id);
                        5 + arguments.len() + results.len()
                    }
                    Value::ForeignFunction(_) => {
                        // ForeignCall opcode + arg marshalling + result unmarshalling
                        let results = dfg.instruction_results(id);
                        3 + arguments.len() + results.len()
                    }
                    Value::Intrinsic(intrinsic) => match intrinsic {
                        // Single opcode intrinsics
                        Intrinsic::ArrayLen
                        | Intrinsic::FieldLessThan
                        | Intrinsic::ArrayRefCount
                        | Intrinsic::VectorRefCount
                        | Intrinsic::IsUnconstrained => 1,

                        // Vector operations: procedure call + metadata + RC check + mem copy
                        Intrinsic::VectorPushBack | Intrinsic::VectorPushFront => 60,
                        Intrinsic::VectorPopBack | Intrinsic::VectorPopFront => 40,
                        Intrinsic::VectorInsert | Intrinsic::VectorRemove => 60,

                        // BlackBox: 1 BlackBoxOp + input/output array setup
                        Intrinsic::BlackBox(_) => {
                            let results = dfg.instruction_results(id);
                            3 + arguments.len() + results.len()
                        }

                        // ToBits/ToRadix: radix decomposition + optional reverse
                        Intrinsic::ToBits(_) => 20,
                        Intrinsic::ToRadix(_) => 12,

                        // DerivePedersenGenerators: similar to BlackBox
                        Intrinsic::DerivePedersenGenerators => 10,

                        // AsVector: array-to-vector conversion with metadata setup
                        Intrinsic::AsVector => 5,

                        // Removed before Brillig codegen / compile-time only
                        Intrinsic::ArrayAsStrUnchecked
                        | Intrinsic::StrAsBytes
                        | Intrinsic::AssertConstant
                        | Intrinsic::StaticAssert
                        | Intrinsic::AsWitness
                        | Intrinsic::ApplyRangeConstraint
                        | Intrinsic::Hint(_) => 0,
                    },

                    // Indirect calls (e.g., calling a function pointer from an instruction result or parameter).
                    // These can occur before defunctionalization.
                    Value::Instruction { .. }
                    | Value::Param { .. }
                    | Value::NumericConstant { .. }
                    | Value::Global(_) => {
                        let results = dfg.instruction_results(id);
                        5 + arguments.len() + results.len()
                    }
                }
            }

            Instruction::Allocate | Instruction::Load { .. } | Instruction::Store { .. } => 1,

            Instruction::ArraySet { .. } => {
                // NOTE: Assumes that the RC is one
                7
            }
            Instruction::ArrayGet { .. } => 3,
            // If less than 10 elements, it is translated into a store for each element (~2 ops each: const + store).
            // If 10 or more, it uses a loop, so cap at 20.
            Instruction::MakeArray { elements, .. } => std::cmp::min(elements.len() * 2, 20),

            Instruction::IncrementRc { .. } | Instruction::DecrementRc { .. } => 3,

            Instruction::EnableSideEffectsIf { .. } | Instruction::Noop => 0,
            // TODO: this is only true for non array values
            Instruction::IfElse { .. } => 1,
        }
    }
}

impl TerminatorInstruction {
    /// Estimate the Brillig opcode cost of this terminator instruction.
    pub(crate) fn cost(&self) -> usize {
        match self {
            TerminatorInstruction::JmpIf { .. } => 2, // jump_if + jump
            TerminatorInstruction::Jmp { arguments, .. } => 1 + arguments.len(), // moves + jump
            TerminatorInstruction::Return { return_values, .. } => 1 + return_values.len(), // moves + return
            TerminatorInstruction::Unreachable { .. } => 0,
        }
    }
}

impl Function {
    /// Sum of all instruction and terminator costs across reachable blocks.
    ///
    /// This is an approximation of the average increase in instruction ratio from SSA to Brillig.
    /// To get the actual weight we'd need to codegen this function to Brillig.
    pub(crate) fn cost(&self) -> usize {
        let mut weight = 0;
        for block_id in self.reachable_blocks() {
            for instruction in self.dfg[block_id].instructions() {
                weight += self.dfg[*instruction].cost(*instruction, &self.dfg);
            }
            weight += self.dfg[block_id].unwrap_terminator().cost();
        }
        weight
    }

    /// Compute the Brillig cost of the function's Return terminator.
    ///
    /// When a function is inlined, its Return terminator is eliminated entirely —
    /// the return values become direct SSA value references in the caller.
    /// This cost should be subtracted from `inline_cost` since it is not paid when inlined.
    pub(crate) fn return_cost(&self) -> i64 {
        for block_id in self.reachable_blocks() {
            if let TerminatorInstruction::Return { return_values, .. } =
                self.dfg[block_id].unwrap_terminator()
            {
                return (1 + return_values.len()) as i64;
            }
        }
        0
    }

    /// Per-call-site overhead of retaining this function, in Brillig opcode units.
    ///
    /// A Brillig function call costs `5 + N + M` opcodes at the call site (from `codegen_call`):
    ///   1 Const (stack size) + 1 Mov (save sp) + 1 BinaryIntOp (sp += size) + 1 Call + 1 Mov (restore sp)
    ///   + N Mov's for arguments + M Mov's for returns.
    ///
    /// Additionally, every retained function executes `CheckMaxStackDepth` at entry.
    /// The happy-path execution cost is 5 opcodes:
    ///   1 Call (to procedure) + 1 Const + 1 BinaryIntOp(Lt) + 1 JumpIf + 1 Return.
    ///
    /// This overhead vanishes when the function is inlined.
    pub(crate) fn call_overhead(&self) -> usize {
        let call_overhead = 5;
        let check_max_stack_depth_cost = 5;
        call_overhead
            + check_max_stack_depth_cost
            + self.parameters().len()
            + self.returns().unwrap_or_default().len()
    }
}
