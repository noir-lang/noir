use crate::{
    ssa::{
        acir_gen::InternalVar,
        context::SsaContext,
        mem::{ArrayId, MemArray},
    },
    Evaluator,
};
use acvm::{
    acir::{
        circuit::{directives::Directive, opcodes::{Opcode as AcirOpcode, MemoryBlock, BlockId as AcirBlockId, MemOp}},
        native_types::{Expression, Witness},
    },
    FieldElement,
};

use iter_extended::vecmap;
use std::collections::BTreeMap;

use super::{
    constraints::{self, mul_with_witness, subtract},
    operations::{self},
};

/// Represent a memory operation on the ArrayHeap, at the specified index
/// Operation is one for a store and 0 for a load
// #[derive(Clone, Debug)]
// struct MemOp {
//     operation: Expression,
//     value: Expression,
//     index: Expression,
// }

type MemAddress = u32;

#[derive(Default)]
struct ArrayHeap {
    // maps memory address to InternalVar
    memory_map: BTreeMap<MemAddress, InternalVar>,
    trace: Vec<MemOp>,
    // maps memory address to (values,operation) that must be committed to the trace
    staged: BTreeMap<MemAddress, (Expression, Expression)>,
}

impl ArrayHeap {
    fn commit_staged(&mut self) {
        for (idx, (value, op)) in &self.staged {
            let item = MemOp {
                operation: op.clone(),
                value: value.clone(),
                index: Expression::from_field(FieldElement::from(*idx as i128)),
            };
            self.trace.push(item);
        }
        self.staged.clear();
    }

    fn push(&mut self, index: Expression, value: Expression, op: Expression) {
        let item = MemOp { operation: op, value, index };
        self.trace.push(item);
    }

    fn stage(&mut self, index: u32, value: Expression, op: Expression) {
        self.staged.insert(index, (value, op));
    }


    /// This helper function transforms an expression into a linear expression, by generating a witness if the input expression is not linear nor constant
    fn normalize_expression(expr: &mut Expression, evaluator: &mut Evaluator) -> Expression {
        if !expr.is_linear() || expr.linear_combinations.len() > 1 {
            let w = evaluator.create_intermediate_variable(expr);
            Expression::from(w)
        } else {
            expr.clone()
        }
    }

    /// Decide which opcode to use, depending on the backend support
    fn acir_gen(&self, evaluator: &mut Evaluator, array_id: ArrayId, array_len: u32, is_opcde_supported: IsOpcodeSupported) {
        let (block,ram,rom) = (IsOpcodeSupported(AcirOpcode::Block),IsOpcodeSupported(AcirOpcode::RAM),IsOpcodeSupported(AcirOpcode::ROM));
       if rom
       {
            todo!("Check if the array is read-only and add the rom opcode if it is");   //TODO we need the R-O arrays PR
            self.add_rom_opcode(evaluator,array_id, array_len);
            return;
       }

        match (block,ram) {
           (false, false) =>  self.generate_permutation_constraints(evaluator, array_id, array_len),
           (false, true) => self.add_ram_opcode(evaluator,array_id, array_len),
           (true, _) => 
           {
            evaluator.opcodes.push(AcirOpcode::Block(MemoryBlock {
                id: AcirBlockId(id),
                len: array_len,
                trace: self.trace.clone(),
            }));
           },

        }
    }

    fn add_rom_opcode(&self, evaluator: &mut Evaluator, array_id: ArrayId, array_len: u32) {
        let mut trace = Vec::new();
        for op in self.trace {
            let index = Self::normalize_expression(&mut op.index, evaluator);
            let value = Self::normalize_expression(&mut op.value, evaluator);
            trace.push(MemOp {
                operation: op.operation,
                index,
                value,
            });
        }
        evaluator.opcodes.push(AcirOpcode::ROM(MemoryBlock {
            id: AcirBlockId(id),
            len: array_len,
            trace,
        }));
    }

    fn add_ram_opcode(&self, evaluator: &mut Evaluator, array_id: ArrayId, array_len: u32) {
        todo!("Check there is an initialization phase"); //TODO we need the R-O array PR for this
        let mut trace = Vec::new();
        for op in self.trace {
            //TODO - after the init, we need a witness per-index but this will be managed by BB - we need to wait for the BB ram PR
            let index = Self::normalize_expression(&mut op.index, evaluator);
            let value = Self::normalize_expression(&mut op.value, evaluator);
            trace.push(MemOp {
                operation: op.operation,
                index,
                value,
            });
        }
        evaluator.opcodes.push(AcirOpcode::ROM(MemoryBlock {
            id: AcirBlockId(id),
            len: array_len,
            trace,
        }));
    }

   

    fn generate_outputs(
        inputs: Vec<Expression>,
        bits: &mut Vec<Witness>,
        evaluator: &mut Evaluator,
    ) -> Vec<Expression> {
        let outputs = vecmap(0..inputs.len(), |_| evaluator.add_witness_to_cs().into());
        if bits.is_empty() {
            *bits = operations::sort::evaluate_permutation(&inputs, &outputs, evaluator);
        } else {
            operations::sort::evaluate_permutation_with_witness(&inputs, &outputs, bits, evaluator);
        }
        outputs
    }
    pub(crate) fn generate_permutation_constraints(&self, evaluator: &mut Evaluator, array_id: ArrayId, array_len: u32) {
        let len = self.trace.len();
        if len == 0 {
            return;
        }
        let len_bits = AcirMem::bits(len);
        // permutations
        let mut in_counter = Vec::new();
        let mut in_index = Vec::new();
        let mut in_value = Vec::new();
        let mut in_op = Vec::new();

        let mut tuple_expressions = Vec::new();
        for (counter, item) in self.trace.iter().enumerate() {
            let counter_expr = Expression::from_field(FieldElement::from(counter as i128));
            in_counter.push(counter_expr.clone());
            in_index.push(item.index.clone());
            in_value.push(item.value.clone());
            in_op.push(item.operation.clone());
            tuple_expressions.push(vec![item.index.clone(), counter_expr.clone()]);
        }
        let mut bit_counter = Vec::new();
        let out_counter = Self::generate_outputs(in_counter, &mut bit_counter, evaluator);
        let out_index = Self::generate_outputs(in_index, &mut bit_counter, evaluator);
        let out_value = Self::generate_outputs(in_value, &mut bit_counter, evaluator);
        let out_op = Self::generate_outputs(in_op, &mut bit_counter, evaluator);

        // sort directive
        evaluator.opcodes.push(AcirOpcode::Directive(Directive::PermutationSort {
            inputs: tuple_expressions,
            tuple: 2,
            bits: bit_counter,
            sort_by: vec![0, 1],
        }));
        let init = subtract(&out_op[0], FieldElement::one(), &Expression::one());
        evaluator.opcodes.push(AcirOpcode::Arithmetic(init));
        for i in 0..len - 1 {
            // index sort
            let index_sub = subtract(&out_index[i + 1], FieldElement::one(), &out_index[i]);
            let primary_order = constraints::boolean_expr(&index_sub, evaluator);
            evaluator.opcodes.push(AcirOpcode::Arithmetic(primary_order));
            // counter sort
            let cmp = constraints::evaluate_cmp(
                &out_counter[i],
                &out_counter[i + 1],
                len_bits,
                false,
                evaluator,
            );
            let sub_cmp = subtract(&cmp, FieldElement::one(), &Expression::one());
            let secondary_order = subtract(
                &mul_with_witness(evaluator, &index_sub, &sub_cmp),
                FieldElement::one(),
                &sub_cmp,
            );
            evaluator.opcodes.push(AcirOpcode::Arithmetic(secondary_order));
            // consistency checks
            let sub1 = subtract(&Expression::one(), FieldElement::one(), &out_op[i + 1]);
            let sub2 = subtract(&out_value[i + 1], FieldElement::one(), &out_value[i]);
            let load_on_same_adr = mul_with_witness(evaluator, &sub1, &sub2);
            let store_on_new_adr = mul_with_witness(evaluator, &index_sub, &sub1);
            evaluator.opcodes.push(AcirOpcode::Arithmetic(store_on_new_adr));
            evaluator.opcodes.push(AcirOpcode::Arithmetic(load_on_same_adr));
        }
        
        let id = array_id.as_u32();
        evaluator.opcodes.push(AcirOpcode::Block(MemoryBlock {
            id: AcirBlockId(id),
            len: array_len,
            trace: self.trace.clone(),
            init: 0,
        }));
        
    }
}

/// Handle virtual memory access
#[derive(Default)]
pub(crate) struct AcirMem {
    virtual_memory: BTreeMap<ArrayId, ArrayHeap>,
    //pub(crate) dummy: Witness,
}

impl AcirMem {
    // Returns the memory_map for the array
    fn array_map_mut(&mut self, array_id: ArrayId) -> &mut BTreeMap<u32, InternalVar> {
        &mut self.virtual_memory.entry(array_id).or_default().memory_map
    }

    // returns the memory trace for the array
    fn array_heap_mut(&mut self, array_id: ArrayId) -> &mut ArrayHeap {
        let e = self.virtual_memory.entry(array_id);
        e.or_default()
    }

    // Write the value to the array's VM at the specified index
    pub(super) fn insert(&mut self, array_id: ArrayId, index: MemAddress, value: InternalVar) {
        let heap = self.virtual_memory.entry(array_id).or_default();
        let value_expr = value.to_expression();
        heap.memory_map.insert(index, value);
        heap.stage(index, value_expr, Expression::one());
    }

    //Map the outputs into the array
    pub(super) fn map_array(&mut self, a: ArrayId, outputs: &[Witness], ctx: &SsaContext) {
        let array = &ctx.mem[a];
        for i in 0..array.len {
            let var = if i < outputs.len() as u32 {
                InternalVar::from(outputs[i as usize])
            } else {
                InternalVar::zero_expr()
            };
            self.array_map_mut(array.id).insert(i, var);
        }
    }

    // Load array values into InternalVars
    // If create_witness is true, we create witnesses for values that do not have witness
    pub(super) fn load_array(&mut self, array: &MemArray) -> Vec<InternalVar> {
        vecmap(0..array.len, |offset| {
            self.load_array_element_constant_index(array, offset)
                .expect("infallible: array out of bounds error")
        })
    }

    // Number of bits required to store the input
    fn bits(mut t: usize) -> u32 {
        let mut r = 0;
        while t != 0 {
            t >>= 1;
            r += 1;
        }
        r
    }

    // Loads the associated `InternalVar` for the element
    // in the `array` at the given `offset`.
    //
    // We check if the address of the array element
    // is in the memory_map.
    //
    //
    // Returns `None` if not found
    pub(super) fn load_array_element_constant_index(
        &mut self,
        array: &MemArray,
        offset: MemAddress,
    ) -> Option<InternalVar> {
        // Check the memory_map to see if the element is there
        self.array_map_mut(array.id).get(&offset).cloned()
    }

    // Apply staged stores to the memory trace
    fn commit(&mut self, array_id: &ArrayId, clear: bool) {
        let e = self.virtual_memory.entry(*array_id).or_default();
        e.commit_staged();
        if clear {
            e.memory_map.clear();
        }
    }
    pub(crate) fn add_to_trace(
        &mut self,
        array_id: &ArrayId,
        index: Expression,
        value: Expression,
        op: Expression,
    ) {
        self.commit(array_id, op != Expression::zero());
        self.array_heap_mut(*array_id).push(index, value, op);
    }
    pub(crate) fn acir_gen(&self, evaluator: &mut Evaluator, ctx: &SsaContext) {
        for mem in &self.virtual_memory {
            let array = ctx.mem[*mem.0];
            mem.1.acir_gen(evaluator, array.id, array.len);
        }
    }
}
