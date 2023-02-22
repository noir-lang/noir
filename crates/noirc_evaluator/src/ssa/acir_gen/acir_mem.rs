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
        circuit::{directives::Directive, opcodes::Opcode as AcirOpcode},
        native_types::{Expression, Witness},
    },
    FieldElement,
};

use iter_extended::vecmap;
use std::collections::{BTreeMap, HashSet};

use super::{
    const_from_expression,
    constraints::{self, mul_with_witness, subtract},
    expression_from_witness,
    operations::{self},
};

#[derive(Clone, Debug)]
pub struct MemItem {
    op: Expression,
    value: Expression,
    index: Expression,
}

enum ArrayType {
    Init(HashSet<u32>, u32),
    WriteOnly,
    ReadOnly(Option<usize>),
    ReadWrite(Option<usize>),
}

impl Default for ArrayType {
    fn default() -> Self {
        ArrayType::Init(HashSet::default(), 0)
    }
}

#[derive(Default)]
pub struct ArrayHeap {
    // maps memory address to InternalVar
    memory_map: BTreeMap<u32, InternalVar>,
    trace: Vec<MemItem>,
    staged: BTreeMap<u32, (Expression, Expression)>,
    typ: ArrayType,
}

impl ArrayHeap {
    pub fn commit_staged(&mut self) {
        for (idx, (value, op)) in &self.staged.clone() {
            let item = MemItem {
                op: op.clone(),
                value: value.clone(),
                index: Expression::from_field(FieldElement::from(*idx as i128)),
            };
            self.trace.push(item);
        }
        self.staged.clear();
    }

    pub fn push(&mut self, item: MemItem) {
        let is_load = item.op == Expression::zero();
        let index_const = const_from_expression(&item.index);
        self.typ = match &self.typ {
            ArrayType::Init(init_idx, len) => match (is_load, index_const) {
                (false, Some(idx)) => {
                    let idx: u32 = idx.to_u128().try_into().unwrap();
                    let mut init_idx2 = init_idx.clone();
                    let mut len2 = *len;
                    init_idx2.insert(idx);
                    if idx + 1 > len2 {
                        len2 = idx + 1;
                    }
                    ArrayType::Init(init_idx2, len2)
                }
                (false, None) => ArrayType::WriteOnly,
                (true, _) => {
                    if *len as usize == init_idx.len() {
                        ArrayType::ReadOnly(None)
                    } else {
                        ArrayType::ReadWrite(None)
                    }
                }
            },
            ArrayType::WriteOnly => {
                if is_load {
                    ArrayType::ReadWrite(None)
                } else {
                    ArrayType::WriteOnly
                }
            }
            ArrayType::ReadOnly(last) => match (is_load, last) {
                (true, Some(_)) => ArrayType::ReadWrite(None),
                (true, None) => ArrayType::ReadOnly(None),
                (false, None) => ArrayType::ReadOnly(Some(self.trace.len())),
                (false, Some(_)) => ArrayType::ReadOnly(*last),
            },
            ArrayType::ReadWrite(last) => match (is_load, last) {
                (true, _) => ArrayType::ReadWrite(None),
                (false, None) => ArrayType::ReadWrite(Some(self.trace.len())),
                (false, Some(_)) => ArrayType::ReadWrite(*last),
            },
        };
        self.trace.push(item);
    }

    pub fn stage(&mut self, index: u32, value: Expression, op: Expression) {
        self.staged.insert(index, (value, op));
    }

    pub fn acir_gen(&self, evaluator: &mut Evaluator) {
        let mut len = self.trace.len();

        let mut read_write = true;
        match self.typ {
            ArrayType::Init(_, _) | ArrayType::WriteOnly => len = 0,
            ArrayType::ReadOnly(last) => {
                len = last.unwrap_or(len);
                read_write = false;
            }
            ArrayType::ReadWrite(last) => {
                len = last.unwrap_or(len);
            }
        }
        if len == 0 {
            return;
        }
        let len_bits = AcirMem::bits(len);
        // permutations
        let mut in_counter = Vec::new();
        let mut in_index = Vec::new();
        let mut in_value = Vec::new();
        let mut in_op = Vec::new();
        let mut out_counter = Vec::new();
        let mut out_index = Vec::new();
        let mut out_value = Vec::new();
        let mut out_op = Vec::new();
        let mut tuple_expressions = Vec::new();
        for (counter, item) in self.trace.iter().take(len).enumerate() {
            let counter_expr = Expression::from_field(FieldElement::from(counter as i128));
            in_counter.push(counter_expr.clone());
            in_index.push(item.index.clone());
            in_value.push(item.value.clone());
            in_op.push(item.op.clone());
            out_counter.push(expression_from_witness(evaluator.add_witness_to_cs()));
            out_index.push(expression_from_witness(evaluator.add_witness_to_cs()));
            out_value.push(expression_from_witness(evaluator.add_witness_to_cs()));
            if read_write {
                out_op.push(expression_from_witness(evaluator.add_witness_to_cs()));
            }
            tuple_expressions.push(vec![item.index.clone(), counter_expr.clone()]);
        }
        let bit_counter =
            operations::sort::evaluate_permutation(&in_counter, &out_counter, evaluator);
        operations::sort::evaluate_permutation_with_witness(
            &in_index,
            &out_index,
            &bit_counter,
            evaluator,
        );
        operations::sort::evaluate_permutation_with_witness(
            &in_value,
            &out_value,
            &bit_counter,
            evaluator,
        );
        if read_write {
            operations::sort::evaluate_permutation_with_witness(
                &in_op,
                &out_op,
                &bit_counter,
                evaluator,
            );
        }

        // sort directive
        evaluator.opcodes.push(AcirOpcode::Directive(Directive::PermutationSort {
            inputs: tuple_expressions,
            tuple: 2,
            bits: bit_counter,
            sort_by: vec![0, 1],
        }));
        if read_write {
            let init = subtract(&out_op[0], FieldElement::one(), &Expression::one());
            evaluator.opcodes.push(AcirOpcode::Arithmetic(init));
        }
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
            let sub2 = subtract(&out_value[i + 1], FieldElement::one(), &out_value[i]);
            let load_on_same_adr = if read_write {
                let sub1 = subtract(&Expression::one(), FieldElement::one(), &out_op[i + 1]);
                let store_on_new_adr = mul_with_witness(evaluator, &index_sub, &sub1);
                evaluator.opcodes.push(AcirOpcode::Arithmetic(store_on_new_adr));
                mul_with_witness(evaluator, &sub1, &sub2)
            } else {
                subtract(
                    &mul_with_witness(evaluator, &index_sub, &sub2),
                    FieldElement::one(),
                    &sub2,
                )
            };
            evaluator.opcodes.push(AcirOpcode::Arithmetic(load_on_same_adr));
        }
    }
}

/// Handle virtual memory access
#[derive(Default)]
pub struct AcirMem {
    virtual_memory: BTreeMap<ArrayId, ArrayHeap>,
    pub dummy: Witness,
}

impl AcirMem {
    // Returns the memory_map for the array
    fn array_map_mut(&mut self, array_id: ArrayId) -> &mut BTreeMap<u32, InternalVar> {
        &mut self.virtual_memory.entry(array_id).or_default().memory_map
    }

    // returns the memory trace for the array
    pub fn array_heap_mut(&mut self, array_id: ArrayId) -> &mut ArrayHeap {
        let e = self.virtual_memory.entry(array_id);
        e.or_default()
    }

    // Write the value to the array's VM at the specified index
    pub fn insert(&mut self, array_id: ArrayId, index: u32, value: InternalVar) {
        let e = self.virtual_memory.entry(array_id).or_default();
        let value_expr = value.to_expression();
        e.memory_map.insert(index, value);

        e.stage(index, value_expr, Expression::one());
    }

    //Map the outputs into the array
    pub(crate) fn map_array(&mut self, a: ArrayId, outputs: &[Witness], ctx: &SsaContext) {
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
    pub(crate) fn load_array(&mut self, array: &MemArray) -> Vec<InternalVar> {
        vecmap(0..array.len, |offset| {
            self.load_array_element_constant_index(array, offset)
                .expect("infallible: array out of bounds error")
        })
    }

    // number of bits required to store the input
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
    pub(crate) fn load_array_element_constant_index(
        &mut self,
        array: &MemArray,
        offset: u32,
    ) -> Option<InternalVar> {
        // Check the memory_map to see if the element is there
        self.array_map_mut(array.id).get(&offset).cloned()
    }

    // Apply staged stores to the memory trace
    pub fn commit(&mut self, array_id: &ArrayId, clear: bool) {
        let e = self.virtual_memory.entry(*array_id).or_default();
        e.commit_staged();
        if clear {
            e.memory_map.clear();
        }
    }
    pub fn add_to_trace(
        &mut self,
        array_id: &ArrayId,
        index: Expression,
        value: Expression,
        op: Expression,
    ) {
        self.commit(array_id, op != Expression::zero());
        let item = MemItem { op, value, index };
        self.array_heap_mut(*array_id).push(item);
    }
    pub fn acir_gen(&self, evaluator: &mut Evaluator) {
        for mem in &self.virtual_memory {
            mem.1.acir_gen(evaluator);
        }
    }
}
