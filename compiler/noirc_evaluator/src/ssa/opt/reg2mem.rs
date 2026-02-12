use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use iter_extended::vecmap;
use rustc_hash::FxHashMap as HashMap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        instruction::{Instruction, TerminatorInstruction},
        types::Type,
        value::ValueId,
    },
    ssa_gen::Ssa,
};
use noirc_errors::call_stack::CallStackId;

/// For each non-entry block that has parameters, the list of (param_value_id, param_type).
/// BTreeMap gives deterministic iteration order so allocations appear in block-ID order.
type BlockParams = BTreeMap<BasicBlockId, Vec<(ValueId, Type)>>;

/// Maps each destination block to a Vec of Allocate result ValueIds (one per parameter).
type AllocMap = HashMap<BasicBlockId, Vec<ValueId>>;

impl Ssa {
    /// Converts block parameters (on non-entry blocks) back to memory operations
    /// (Allocate/Store/Load) for Brillig functions. This is the inverse of `mem2reg_simple`.
    ///
    /// This simplifies the Brillig codegen backend by replacing block parameter passing
    /// (which requires complex "simultaneous move" logic with temp registers) with simple
    /// memory operations.
    pub(crate) fn reg2mem(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            if function.runtime().is_brillig() {
                reg2mem(function);
            }
        }
        self
    }
}

fn reg2mem(function: &mut Function) {
    let entry_block = function.entry_block();
    let reachable = function.reachable_blocks();

    let block_params = collect_block_params(function, entry_block, &reachable);
    if block_params.is_empty() {
        return;
    }

    let allocs = insert_allocations(function, entry_block, &block_params);
    replace_args_with_stores(function, &reachable, &allocs);
    let value_map = replace_params_with_loads(function, &block_params, &allocs);
    replace_value_uses(function, &reachable, &value_map);
}

/// Collect every non-entry block that has block parameters.
fn collect_block_params(
    function: &Function,
    entry_block: BasicBlockId,
    reachable: &BTreeSet<BasicBlockId>,
) -> BlockParams {
    let mut result = BlockParams::default();
    for &block_id in reachable {
        if block_id == entry_block {
            continue;
        }
        let params = function.dfg.block_parameters(block_id);
        if params.is_empty() {
            continue;
        }
        let infos = vecmap(params, |&id| (id, function.dfg.type_of_value(id)));
        result.insert(block_id, infos);
    }
    result
}

/// For every block parameter, insert an `Allocate` in the entry block.
fn insert_allocations(
    function: &mut Function,
    entry_block: BasicBlockId,
    block_params: &BlockParams,
) -> AllocMap {
    let mut allocs = AllocMap::default();
    for (&block_id, params) in block_params {
        let alloc_ids: Vec<ValueId> = params
            .iter()
            .map(|(_, typ)| {
                let ref_type = Type::Reference(Arc::new(typ.clone()));
                function
                    .dfg
                    .insert_instruction_and_results_without_simplification(
                        Instruction::Allocate,
                        entry_block,
                        Some(vec![ref_type]),
                        CallStackId::root(),
                    )
                    .first()
            })
            .collect();
        allocs.insert(block_id, alloc_ids);
    }
    allocs
}

/// For each terminator that passes arguments, insert `Store` instructions for every
/// argument, then clear the terminator's argument lists.
fn replace_args_with_stores(
    function: &mut Function,
    reachable: &BTreeSet<BasicBlockId>,
    allocs: &AllocMap,
) {
    for &block_id in reachable {
        let terminator = function.dfg[block_id].take_terminator();
        let new_terminator = match terminator {
            TerminatorInstruction::Jmp { destination, arguments, call_stack } => {
                insert_stores(function, block_id, destination, &arguments, allocs);
                TerminatorInstruction::Jmp { destination, arguments: Vec::new(), call_stack }
            }
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                then_arguments,
                else_destination,
                else_arguments,
                call_stack,
            } => {
                insert_stores(function, block_id, then_destination, &then_arguments, allocs);
                insert_stores(function, block_id, else_destination, &else_arguments, allocs);
                TerminatorInstruction::JmpIf {
                    condition,
                    then_destination,
                    then_arguments: Vec::new(),
                    else_destination,
                    else_arguments: Vec::new(),
                    call_stack,
                }
            }
            other => other,
        };
        function.dfg[block_id].set_terminator(new_terminator);
    }
}

/// Insert a `Store` for each argument into the corresponding allocation for `destination`.
fn insert_stores(
    function: &mut Function,
    block_id: BasicBlockId,
    destination: BasicBlockId,
    arguments: &[ValueId],
    allocs: &AllocMap,
) {
    let Some(alloc_ids) = allocs.get(&destination) else {
        return;
    };
    for (idx, &arg) in arguments.iter().enumerate() {
        function.dfg.insert_instruction_and_results_without_simplification(
            Instruction::Store { address: alloc_ids[idx], value: arg },
            block_id,
            None,
            CallStackId::root(),
        );
    }
}

/// For each block that had parameters, prepend `Load` instructions and remove the
/// parameters. Returns a mapping from old parameter ValueIds to the new Load results.
fn replace_params_with_loads(
    function: &mut Function,
    block_params: &BlockParams,
    allocs: &AllocMap,
) -> HashMap<ValueId, ValueId> {
    let mut value_map = HashMap::default();
    for (&block_id, params) in block_params {
        let alloc_ids = &allocs[&block_id];
        let original_instructions = function.dfg[block_id].take_instructions();

        // Insert Load instructions (appended to the now-empty block).
        for (idx, (param_id, typ)) in params.iter().enumerate() {
            let load_value = function
                .dfg
                .insert_instruction_and_results_without_simplification(
                    Instruction::Load { address: alloc_ids[idx] },
                    block_id,
                    Some(vec![typ.clone()]),
                    CallStackId::root(),
                )
                .first();
            value_map.insert(*param_id, load_value);
        }

        // Loads are now at the front; re-append the original instructions after them.
        function.dfg[block_id].instructions_mut().extend(original_instructions);

        function.dfg[block_id].take_parameters();
    }
    value_map
}

/// Rewrite every value reference in the function so old block-parameter ValueIds
/// point to their replacement Load results.
fn replace_value_uses(
    function: &mut Function,
    reachable: &BTreeSet<BasicBlockId>,
    value_map: &HashMap<ValueId, ValueId>,
) {
    let map = |value: ValueId| -> ValueId { *value_map.get(&value).unwrap_or(&value) };
    for &block_id in reachable {
        let instructions = function.dfg[block_id].take_instructions();
        for &instruction_id in &instructions {
            function.dfg[instruction_id].map_values_mut(map);
        }
        *function.dfg[block_id].instructions_mut() = instructions;

        let mut terminator = function.dfg[block_id].take_terminator();
        terminator.map_values_mut(map);
        function.dfg[block_id].set_terminator(terminator);
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn simple_jmp() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            jmp b1(v0)
          b1(v1: Field):
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.reg2mem();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = allocate -> &mut Field
            store v0 at v1
            jmp b1()
          b1():
            v2 = load v1 -> Field
            return v2
        }
        ");
    }

    #[test]
    fn jmpif_with_arguments() {
        // JmpIf with arguments on both branches, merging into a common exit block
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(Field 1), else: b1(Field 2)
          b1(v1: Field):
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.reg2mem();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 1 at v1
            store Field 2 at v1
            jmpif v0 then: b1(), else: b1()
          b1():
            v4 = load v1 -> Field
            return v4
        }
        ");
    }

    #[test]
    fn multiple_parameters() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            jmp b1(Field 1, u32 2)
          b1(v0: Field, v1: u32):
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.reg2mem();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut u32
            store Field 1 at v0
            store u32 2 at v1
            jmp b1()
          b1():
            v4 = load v0 -> Field
            v5 = load v1 -> u32
            return v4
        }
        ");
    }

    #[test]
    fn chain_of_blocks() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            jmp b1(Field 1)
          b1(v0: Field):
            v1 = add v0, Field 1
            jmp b2(v1)
          b2(v2: Field):
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.reg2mem();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            store Field 1 at v0
            jmp b1()
          b1():
            v3 = load v0 -> Field
            v4 = add v3, Field 1
            store v4 at v1
            jmp b2()
          b2():
            v5 = load v1 -> Field
            return v5
        }
        ");
    }

    #[test]
    fn loop_back_edge() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            jmp b1(Field 0)
          b1(v0: Field):
            v1 = eq v0, Field 10
            jmpif v1 then: b2(), else: b3()
          b2():
            return v0
          b3():
            v2 = add v0, Field 1
            jmp b1(v2)
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.reg2mem();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            jmp b1()
          b1():
            v2 = load v0 -> Field
            v4 = eq v2, Field 10
            jmpif v4 then: b2(), else: b3()
          b2():
            return v2
          b3():
            v6 = add v2, Field 1
            store v6 at v0
            jmp b1()
        }
        ");
    }

    #[test]
    fn no_op_no_block_params() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            jmp b1()
          b1():
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.reg2mem();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            jmp b1()
          b1():
            return v0
        }
        ");
    }

    #[test]
    fn entry_block_params_preserved() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: u32):
            jmp b1(v0)
          b1(v2: Field):
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.reg2mem();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: u32):
            v2 = allocate -> &mut Field
            store v0 at v2
            jmp b1()
          b1():
            v3 = load v2 -> Field
            return v3
        }
        ");
    }

    #[test]
    fn acir_function_unchanged() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            jmp b1(Field 1)
          b1(v0: Field):
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.reg2mem();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            jmp b1(Field 1)
          b1(v0: Field):
            return v0
        }
        ");
    }

    #[test]
    fn diamond_with_block_params() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1(), else: b2()
          b1():
            jmp b3(Field 1)
          b2():
            jmp b3(Field 2)
          b3(v1: Field):
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.reg2mem();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            jmpif v0 then: b1(), else: b2()
          b1():
            store Field 1 at v1
            jmp b3()
          b2():
            store Field 2 at v1
            jmp b3()
          b3():
            v4 = load v1 -> Field
            return v4
        }
        ");
    }
}
