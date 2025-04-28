use crate::block_context::BlockContext;
use acvm::FieldElement;
use acvm::acir::native_types::Witness;
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, FuzzerBuilderError, InstructionWithOneArg, InstructionWithTwoArgs},
    config::NUMBER_OF_VARIABLES_INITIAL,
    typed_value::{TypedValue, ValueType},
};
use noirc_driver::CompiledProgram;
use noirc_evaluator::ssa::ir::basic_block::BasicBlockId;
use std::{clone, collections::HashMap};
#[derive(Arbitrary, Debug, Clone, Hash)]
pub(crate) struct Argument {
    /// Index of the argument in the context of stored variables of this type
    /// e.g. if we have variables with ids [0, 1] in u64 vector and variables with ids [5, 8] in fields vector
    /// Argument(Index(0), ValueType::U64) -> id 0
    /// Argument(Index(0), ValueType::Field) -> id 5
    /// Argument(Index(1), ValueType::Field) -> id 8
    pub(crate) index: usize,
    /// Type of the argument
    pub(crate) value_type: ValueType,
}

#[derive(Arbitrary, Debug, Clone, Hash)]
pub(crate) enum Instruction {
    /// Addition of two values
    AddChecked { lhs: Argument, rhs: Argument },
    /// Subtraction of two values
    SubChecked { lhs: Argument, rhs: Argument },
    /// Multiplication of two values
    MulChecked { lhs: Argument, rhs: Argument },
    /// Division of two values
    Div { lhs: Argument, rhs: Argument },
    /// Equality comparison
    Eq { lhs: Argument, rhs: Argument },
    /// Modulo operation
    Mod { lhs: Argument, rhs: Argument },
    /// Bitwise NOT
    Not { lhs: Argument },
    /// Left shift
    Shl { lhs: Argument, rhs: Argument },
    /// Right shift
    Shr { lhs: Argument, rhs: Argument },
    /// Cast into type
    Cast { lhs: Argument, type_: ValueType },
    /// Bitwise AND
    And { lhs: Argument, rhs: Argument },
    /// Bitwise OR
    Or { lhs: Argument, rhs: Argument },
    /// Bitwise XOR
    Xor { lhs: Argument, rhs: Argument },
}

#[derive(Arbitrary, Debug, Clone, Hash, Copy, PartialEq)]
pub(crate) enum Terminator {
    /// terminates ssa block with jmp
    /// block_index take blocks from already terminated with return
    Jmp { block_index: usize },
    /// terminates ssa block with jmp_if_else
    /// condition_index -- index of taken boolean
    /// if, else blocks are blocks terminated with Jmp or Return
    JmpIfElse { condition_index: usize, then_block_index: usize, else_block_index: usize },
}

#[derive(Arbitrary, Debug, Clone, Hash)]
pub(crate) struct Block {
    instructions: Vec<Instruction>,
    terminator: Terminator,
}

#[derive(Clone)]
pub(crate) struct StoredBlock {
    context: BlockContext,
    terminator: Terminator,
    block_id: BasicBlockId,
}

/// Main context for the fuzzer containing both ACIR and Brillig builders and their state
/// It works with indices of variables Ids, because it cannot handle Ids logic for ACIR and Brillig
pub(crate) struct FuzzerContext {
    /// ACIR builder
    acir_builder: FuzzerBuilder,
    /// Brillig builder
    brillig_builder: FuzzerBuilder,
    /// Ids of ACIR witnesses stored as TypedValue separated by type
    acir_ids: HashMap<ValueType, Vec<TypedValue>>,
    /// Ids of Brillig witnesses stored as TypedValue separated by type
    brillig_ids: HashMap<ValueType, Vec<TypedValue>>,
    /// ACIR and Brillig last changed value
    last_value_acir: Option<TypedValue>,
    last_value_brillig: Option<TypedValue>,
    /// Context of the current SSA block
    current_block_context: BlockContext,
    /// If `b0` SSA block processed
    first_block_processed: bool,
    /// List of prepared, but not terminated blocks
    stored_blocks: Vec<StoredBlock>,
    /// List of terminated blocks,
    terminated_blocks: Vec<BasicBlockId>,
    /// Whether the context is constant execution
    is_constant: bool,
}

impl FuzzerContext {
    /// Creates a new fuzzer context with the given types
    /// It creates a new variable for each type and stores it in the map
    ///
    /// For example, if we have types [u64, u64, field], it will create 3 variables
    /// and store them in the map as {u64: [0, 1], field: [2]} for both ACIR and Brillig
    pub(crate) fn new(types: Vec<ValueType>) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        let mut acir_ids = HashMap::new();
        let mut brillig_ids = HashMap::new();
        for type_ in types {
            let acir_id = acir_builder.insert_variable(type_.to_ssa_type());
            let brillig_id = brillig_builder.insert_variable(type_.to_ssa_type());
            acir_ids.entry(type_.clone()).or_insert(Vec::new()).push(acir_id);
            brillig_ids.entry(type_).or_insert(Vec::new()).push(brillig_id);
        }
        let current_block_context = BlockContext::new(acir_ids.clone(), brillig_ids.clone());

        Self {
            acir_builder,
            brillig_builder,
            acir_ids,
            brillig_ids,
            last_value_acir: None,
            last_value_brillig: None,
            current_block_context,
            first_block_processed: false,
            stored_blocks: vec![],
            terminated_blocks: vec![],
            is_constant: false,
        }
    }

    /// Creates a new fuzzer context with the given values for a constant folding checking
    ///
    /// For example, if we have values [1, 2, 3] and types [u64, u64, field], it will create 3 constants
    /// and store them in the map as {u64: [0, 1], field: [2]} for both ACIR and Brillig
    pub(crate) fn new_constant_context(
        values: Vec<impl Into<FieldElement>>,
        types: Vec<ValueType>,
    ) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        let mut acir_ids = HashMap::new();
        let mut brillig_ids = HashMap::new();

        for (value, type_) in values.into_iter().zip(types) {
            let field_element = value.into();
            acir_ids
                .entry(type_.clone())
                .or_insert(Vec::new())
                .push(acir_builder.insert_constant(field_element, type_.clone()));
            brillig_ids
                .entry(type_.clone())
                .or_insert(Vec::new())
                .push(brillig_builder.insert_constant(field_element, type_.clone()));
        }
        let current_block_context = BlockContext::new(acir_ids.clone(), brillig_ids.clone());

        Self {
            acir_builder,
            brillig_builder,
            acir_ids,
            brillig_ids,
            current_block_context,
            last_value_acir: None,
            last_value_brillig: None,
            first_block_processed: false,
            stored_blocks: vec![],
            terminated_blocks: vec![],
            is_constant: true,
        }
    }

    /// Inserts an instruction into both ACIR and Brillig programs (in the current block)
    fn insert_instruction(&mut self, instruction: Instruction) {
        self.current_block_context.insert_instruction(
            &mut self.acir_builder,
            &mut self.brillig_builder,
            instruction,
        );
    }

    /// if it is the first block, other blocks can use variables from it
    /// after inserting all instruction, adds new block and switch context to it
    /// previous block stored to `stored_blocks`
    pub(crate) fn process_block(&mut self, block: Block) {
        for instruction in block.instructions {
            self.insert_instruction(instruction);
        }
        if !self.first_block_processed {
            self.acir_ids = self.current_block_context.acir_ids.clone();
            self.brillig_ids = self.current_block_context.brillig_ids.clone();
        }
        self.stored_blocks.push(StoredBlock {
            context: self.current_block_context.clone(),
            terminator: block.terminator,
            block_id: self.acir_builder.get_current_block(),
        });

        self.first_block_processed = true;
        let acir_new_block = self.acir_builder.insert_block();
        let brillig_new_block = self.brillig_builder.insert_block();
        self.acir_builder.switch_to_block(acir_new_block);
        self.brillig_builder.switch_to_block(brillig_new_block);
        self.current_block_context =
            BlockContext::new(self.acir_ids.clone(), self.brillig_ids.clone());
    }

    fn get_block_ids_for_if_then(
        self,
        then_block_index: usize,
        else_block_index: usize,
    ) -> (Option<BasicBlockId>, Option<BasicBlockId>) {
        let then_destination =
            self.terminated_blocks[then_block_index % self.terminated_blocks.len()];
        let else_destination =
            self.terminated_blocks[else_block_index % self.terminated_blocks.len()];
        (Some(then_destination), Some(else_destination))
    }

    fn finalize_block(&mut self, stored_block: StoredBlock) {
        self.acir_builder.switch_to_block(stored_block.block_id);
        self.brillig_builder.switch_to_block(stored_block.block_id);
        match stored_block.terminator {
            Terminator::Jmp { block_index } => {
                let jmp_destination =
                    self.terminated_blocks[block_index % self.terminated_blocks.len()];
                stored_block.context.finalize_block_with_jmp(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    jmp_destination,
                );
            }
            Terminator::JmpIfElse { condition_index, then_block_index, else_block_index } => {
                let then_destination =
                    self.terminated_blocks[then_block_index % self.terminated_blocks.len()];
                let else_destination =
                    self.terminated_blocks[else_block_index % self.terminated_blocks.len()];
                stored_block.context.finalize_block_with_jmp_if(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    condition_index,
                    then_destination,
                    else_destination,
                );
            }
        }
        // If current block is empty we don't override last set value
        (self.last_value_acir, self.last_value_brillig) = match (
            self.current_block_context.last_value_acir.clone(),
            self.current_block_context.last_value_brillig.clone(),
        ) {
            (Some(acir_val), Some(brillig_val)) => (Some(acir_val), Some(brillig_val)),
            _ => (self.last_value_acir.clone(), self.last_value_brillig.clone()),
        };
        self.terminated_blocks.push(self.acir_builder.get_current_block());
    }

    fn terminating_cycle(&mut self, terminator_matches: fn(Terminator) -> bool) {
        for block in self.stored_blocks.clone() {
            if terminator_matches(block.terminator) {
                self.finalize_block(block);
            }
        }
    }

    /// Finalizes the function by terminating all blocks
    /// 1) Terminates blocks with Return;
    /// 2) Terminates blocks with Jmp;
    /// 3) Terminates blocks with JmpIf
    ///
    /// 1) Noir does not support early returns (only one block can be Return block), so we terminate with return only last block.
    /// 2) Flattening does not support case if `then_destination` ends in the end of the branch.
    /// The following program is invalid:
    /// acir(inline) fn main f0 {
    ///  b0(v0: Field, v1: Field, v2: Field, v3: Field, v4: Field, v5: Field, v6: u1):
    ///    jmpif v6 then: b3, else: b1
    ///  b1():
    ///    v7 = div v3, v0
    ///    jmp b3()
    ///  b3():
    ///    return v0
    ///
    /// because branch ends in b3 block
    pub(crate) fn finalize_function(&mut self) {
        // we force last block to terminate with return
        if self.terminated_blocks.is_empty() {
            let last_block = self.stored_blocks.pop().unwrap();
            self.acir_builder.switch_to_block(last_block.block_id);
            self.brillig_builder.switch_to_block(last_block.block_id);
            last_block
                .context
                .clone()
                .finalize_block_with_return(&mut self.acir_builder, &mut self.brillig_builder);
            (self.last_value_acir, self.last_value_brillig) =
                last_block.context.get_last_variables();
            self.terminated_blocks.push(last_block.block_id);
        }

        self.terminating_cycle(|terminator| match terminator {
            Terminator::Jmp { .. } => true,
            _ => false,
        });
        self.terminating_cycle(|terminator| match terminator {
            Terminator::JmpIfElse { .. } => true,
            _ => false,
        });
    }

    /// Returns witnesses for ACIR and Brillig
    /// If program does not have any instructions, it terminated with the last witness
    /// Resulting WitnessStack of programs contains only variables and return value
    /// If we inserted some instructions, WitnessStack contains return value, so we return the last one
    /// If we are checking constant folding, the witness stack will only contain the return value, so we return Witness(0)
    pub(crate) fn get_return_witnesses(&self) -> (Witness, Witness) {
        if self.is_constant {
            return (Witness(0), Witness(0));
        }
        match (self.last_value_acir.clone(), self.last_value_brillig.clone()) {
            (Some(_acir_result), Some(_brillig_result)) => {
                (Witness(NUMBER_OF_VARIABLES_INITIAL - 1), Witness(NUMBER_OF_VARIABLES_INITIAL - 1))
            }
            _ => (Witness(NUMBER_OF_VARIABLES_INITIAL), Witness(NUMBER_OF_VARIABLES_INITIAL)),
        }
    }

    /// Returns programs for ACIR and Brillig
    pub(crate) fn get_programs(
        self,
    ) -> (Result<CompiledProgram, FuzzerBuilderError>, Result<CompiledProgram, FuzzerBuilderError>)
    {
        (self.acir_builder.compile(), self.brillig_builder.compile())
    }
}
