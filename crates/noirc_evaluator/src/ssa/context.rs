use super::block::{BasicBlock, BlockId};
use super::conditional::{DecisionTree, TreeBuilder};
use super::function::{FuncIndex, SSAFunction};
use super::inline::StackFrame;
use super::mem::{ArrayId, Memory};
use super::node::{BinaryOp, FunctionKind, Instruction, NodeId, NodeObject, ObjectType, Operation};
use super::{block, builtin, flatten, inline, integer, node, optimizations};
use std::collections::{HashMap, HashSet};

use super::super::errors::RuntimeError;
use crate::errors::RuntimeErrorKind;
use crate::ssa::acir_gen::Acir;
use crate::ssa::function;
use crate::ssa::node::{Mark, Node};
use crate::Evaluator;
use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_frontend::monomorphization::ast::{Definition, FuncId};
use num_bigint::BigUint;
use num_traits::{One, Zero};

// This is a 'master' class for generating the SSA IR from the AST
// It contains all the data; the node objects representing the source code in the nodes arena
// and The CFG in the blocks arena
// everything else just reference objects from these two arena using their index.
pub struct SsaContext {
    pub first_block: BlockId,
    pub current_block: BlockId,
    blocks: arena::Arena<block::BasicBlock>,
    pub nodes: arena::Arena<node::NodeObject>,
    value_names: HashMap<NodeId, u32>,
    pub sealed_blocks: HashSet<BlockId>,
    pub mem: Memory,

    pub functions: HashMap<FuncId, function::SSAFunction>,
    pub opcode_ids: HashMap<builtin::Opcode, NodeId>,

    //Adjacency Matrix of the call graph; list of rows where each row indicates the functions called by the function whose FuncIndex is the row number
    pub call_graph: Vec<Vec<u8>>,
    dummy_store: HashMap<ArrayId, NodeId>,
    dummy_load: HashMap<ArrayId, NodeId>,
}

impl SsaContext {
    pub fn new() -> SsaContext {
        let mut pc = SsaContext {
            first_block: BlockId::dummy(),
            current_block: BlockId::dummy(),
            blocks: arena::Arena::new(),
            nodes: arena::Arena::new(),
            value_names: HashMap::new(),
            sealed_blocks: HashSet::new(),
            mem: Memory::default(),
            functions: HashMap::new(),
            opcode_ids: HashMap::new(),
            call_graph: Vec::new(),
            dummy_store: HashMap::new(),
            dummy_load: HashMap::new(),
        };
        block::create_first_block(&mut pc);
        pc.one_with_type(node::ObjectType::Boolean);
        pc.zero_with_type(node::ObjectType::Boolean);
        pc
    }

    pub fn zero(&self) -> NodeId {
        self.find_const_with_type(&BigUint::zero(), node::ObjectType::Boolean).unwrap()
    }

    pub fn one(&self) -> NodeId {
        self.find_const_with_type(&BigUint::one(), node::ObjectType::Boolean).unwrap()
    }

    pub fn zero_with_type(&mut self, obj_type: ObjectType) -> NodeId {
        self.get_or_create_const(FieldElement::zero(), obj_type)
    }

    pub fn one_with_type(&mut self, obj_type: ObjectType) -> NodeId {
        self.get_or_create_const(FieldElement::one(), obj_type)
    }

    pub fn is_one(&self, id: NodeId) -> bool {
        if id == NodeId::dummy() {
            return false;
        }
        let typ = self.get_object_type(id);
        if let Some(one) = self.find_const_with_type(&BigUint::one(), typ) {
            id == one
        } else {
            false
        }
    }

    pub fn is_zero(&self, id: NodeId) -> bool {
        if id == NodeId::dummy() {
            return false;
        }
        let typ = self.get_object_type(id);
        if let Some(zero) = self.find_const_with_type(&BigUint::zero(), typ) {
            id == zero
        } else {
            false
        }
    }

    pub fn get_dummy_store(&self, a: ArrayId) -> NodeId {
        self.dummy_store[&a]
    }

    pub fn get_dummy_load(&self, a: ArrayId) -> NodeId {
        self.dummy_load[&a]
    }

    #[allow(clippy::map_entry)]
    pub fn add_dummy_load(&mut self, a: ArrayId) {
        if !self.dummy_load.contains_key(&a) {
            let op_a = Operation::Load { array_id: a, index: NodeId::dummy() };
            let dummy_load = node::Instruction::new(op_a, self.mem[a].element_type, None);
            let id = self.add_instruction(dummy_load);
            self.dummy_load.insert(a, id);
        }
    }
    #[allow(clippy::map_entry)]
    pub fn add_dummy_store(&mut self, a: ArrayId) {
        if !self.dummy_store.contains_key(&a) {
            let op_a =
                Operation::Store { array_id: a, index: NodeId::dummy(), value: NodeId::dummy() };
            let dummy_store = node::Instruction::new(op_a, node::ObjectType::NotAnObject, None);
            let id = self.add_instruction(dummy_store);
            self.dummy_store.insert(a, id);
        }
    }

    pub fn get_function_index(&self) -> FuncIndex {
        FuncIndex::new(self.functions.values().len())
    }

    pub fn insert_block(&mut self, block: BasicBlock) -> &mut BasicBlock {
        let id = self.blocks.insert(block);
        let block = &mut self.blocks[id];
        block.id = BlockId(id);
        block
    }

    //Display an object for debugging purposes
    fn id_to_string(&self, id: NodeId) -> String {
        let mut result = String::new();
        if let Some(var) = self.try_get_node(id) {
            result = format!("{var}");
        }
        if result.is_empty() {
            result = format!("unknown {:?}", id.0.into_raw_parts().0)
        }
        result
    }

    fn binary_to_string(&self, binary: &node::Binary) -> String {
        let lhs = self.id_to_string(binary.lhs);
        let rhs = self.id_to_string(binary.rhs);
        let op = match &binary.operator {
            BinaryOp::Add => "add",
            BinaryOp::SafeAdd => "safe_add",
            BinaryOp::Sub { .. } => "sub",
            BinaryOp::SafeSub { .. } => "safe_sub",
            BinaryOp::Mul => "mul",
            BinaryOp::SafeMul => "safe_mul",
            BinaryOp::Udiv => "udiv",
            BinaryOp::Sdiv => "sdiv",
            BinaryOp::Urem => "urem",
            BinaryOp::Srem => "srem",
            BinaryOp::Div => "div",
            BinaryOp::Eq => "eq",
            BinaryOp::Ne => "ne",
            BinaryOp::Ult => "ult",
            BinaryOp::Ule => "ule",
            BinaryOp::Slt => "slt",
            BinaryOp::Sle => "sle",
            BinaryOp::Lt => "lt",
            BinaryOp::Lte => "lte",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
            BinaryOp::Xor => "xor",
            BinaryOp::Assign => "assign",
            BinaryOp::Shl => "shl",
            BinaryOp::Shr => "shr",
        };

        format!("{op} {lhs}, {rhs}")
    }

    pub fn operation_to_string(&self, op: &Operation) -> String {
        let join = |args: &[NodeId]| vecmap(args, |arg| self.id_to_string(*arg)).join(", ");

        match op {
            Operation::Binary(binary) => self.binary_to_string(binary),
            Operation::Cast(value) => format!("cast {}", self.id_to_string(*value)),
            Operation::Truncate { value, bit_size, max_bit_size } => {
                format!(
                    "truncate {}, bit size = {bit_size}, max bit size = {max_bit_size}",
                    self.id_to_string(*value),
                )
            }
            Operation::Not(v) => format!("not {}", self.id_to_string(*v)),
            Operation::Constrain(v, ..) => format!("constrain {}", self.id_to_string(*v)),
            Operation::Jne(v, b) => format!("jne {}, {b:?}", self.id_to_string(*v)),
            Operation::Jeq(v, b) => format!("jeq {}, {b:?}", self.id_to_string(*v)),
            Operation::Jmp(b) => format!("jmp {b:?}"),
            Operation::Phi { root, block_args } => {
                let mut s = format!("phi {}", self.id_to_string(*root));
                for (value, block) in block_args {
                    s = format!(
                        "{s}, {} from block {}",
                        self.id_to_string(*value),
                        block.0.into_raw_parts().0
                    );
                }
                s
            }
            Operation::Cond { condition, val_true: lhs, val_false: rhs } => {
                let lhs = self.id_to_string(*lhs);
                let rhs = self.id_to_string(*rhs);
                format!("cond({}) {lhs}, {rhs}", self.id_to_string(*condition))
            }
            Operation::Load { array_id, index } => {
                format!("load {array_id:?}, index {}", self.id_to_string(*index))
            }
            Operation::Store { array_id, index, value } => {
                format!(
                    "store {array_id:?}, index {}, value {}",
                    self.id_to_string(*index),
                    self.id_to_string(*value)
                )
            }
            Operation::Intrinsic(opcode, args) => format!("intrinsic {opcode}({})", join(args)),
            Operation::Nop => "nop".into(),
            Operation::Call { func, arguments, returned_arrays, .. } => {
                let name = self.try_get_func_id(*func).map(|id| self.functions[&id].name.clone());
                let name = name.unwrap_or_else(|| self.id_to_string(*func));
                format!("call {name}({}) _ {returned_arrays:?}", join(arguments))
            }
            Operation::Return(values) => format!("return ({})", join(values)),
            Operation::Result { call_instruction, index } => {
                let call = self.id_to_string(*call_instruction);
                format!("result {index} of {call}")
            }
        }
    }

    pub fn print_block(&self, b: &block::BasicBlock) {
        println!("************* Block n.{}", b.id.0.into_raw_parts().0);
        println!("Assumption:{:?}", b.assumption);
        self.print_instructions(&b.instructions);
        if b.left.is_some() {
            println!("Next block: {}", b.left.unwrap().0.into_raw_parts().0);
        }
    }

    pub fn print_instructions(&self, instructions: &[NodeId]) {
        for id in instructions {
            self.print_node(*id)
        }
    }

    pub fn print_node(&self, id: NodeId) {
        println!("{}", self.node_to_string(id));
    }

    pub fn node_to_string(&self, id: NodeId) -> String {
        match self.try_get_node(id) {
            Some(NodeObject::Instr(ins)) => {
                let mut str_res = if ins.res_name.is_empty() {
                    format!("{:?}", id.0.into_raw_parts().0)
                } else {
                    ins.res_name.clone()
                };
                if let Mark::ReplaceWith(replacement) = ins.mark {
                    let new = self.node_to_string(replacement);
                    str_res = format!(
                        "{str_res} -REPLACED with ({}) {new},    original was",
                        replacement.0.into_raw_parts().0,
                    );
                } else if ins.is_deleted() {
                    str_res = format!("{str_res}: DELETED");
                }
                let ins_str = self.operation_to_string(&ins.operation);
                format!("{str_res}: {ins_str}")
            }
            Some(other) => format!("{other}"),
            None => format!("unknown {:?}", id.0.into_raw_parts().0),
        }
    }

    pub fn print(&self, text: &str) {
        println!("{text}");
        for (_, b) in self.blocks.iter() {
            self.print_block(b);
        }
    }

    pub fn remove_block(&mut self, block: BlockId) {
        self.blocks.remove(block.0);
    }

    /// Add an instruction to self.nodes and sets its id.
    /// This function does NOT push the instruction to the current block.
    /// See push_instruction for that.
    pub fn add_instruction(&mut self, instruction: node::Instruction) -> NodeId {
        let obj = NodeObject::Instr(instruction);
        let id = NodeId(self.nodes.insert(obj));
        match &mut self[id] {
            NodeObject::Instr(i) => i.id = id,
            _ => unreachable!(),
        }

        id
    }

    /// Adds the instruction to self.nodes and pushes it to the current block
    pub fn push_instruction(&mut self, instruction: node::Instruction) -> NodeId {
        let id = self.add_instruction(instruction);
        if let NodeObject::Instr(_) = &self[id] {
            self.get_current_block_mut().instructions.push(id);
        }
        id
    }

    /// Adds the instruction to self.nodes and insert it after phi instructions of the provided block
    pub fn insert_instruction_after_phi(
        &mut self,
        instruction: node::Instruction,
        block: BlockId,
    ) -> NodeId {
        let id = self.add_instruction(instruction);
        let mut pos = 0;
        for i in &self[block].instructions {
            if let Some(ins) = self.try_get_instruction(*i) {
                let op = ins.operation.opcode();
                if op != node::Opcode::Nop && op != node::Opcode::Phi {
                    break;
                }
            }
            pos += 1;
        }
        self[block].instructions.insert(pos, id);
        id
    }

    //add the instruction to the block, after the provided instruction
    pub fn push_instruction_after(
        &mut self,
        instruction_id: NodeId,
        block: BlockId,
        after: NodeId,
    ) -> NodeId {
        let mut pos = 0;
        for i in &self[block].instructions {
            if after == *i {
                break;
            }
            pos += 1;
        }
        self[block].instructions.insert(pos + 1, instruction_id);
        instruction_id
    }

    pub fn add_const(&mut self, constant: node::Constant) -> NodeId {
        let obj = NodeObject::Const(constant);
        let id = NodeId(self.nodes.insert(obj));
        match &mut self[id] {
            NodeObject::Const(c) => c.id = id,
            _ => unreachable!(),
        }

        id
    }

    pub fn get_ssa_func(&self, func_id: FuncId) -> Option<&SSAFunction> {
        self.functions.get(&func_id)
    }

    pub fn try_get_func_id(&self, id: NodeId) -> Option<FuncId> {
        match &self[id] {
            NodeObject::Function(FunctionKind::Normal(id), ..) => Some(*id),
            _ => None,
        }
    }

    pub fn try_get_ssa_func(&self, id: NodeId) -> Option<&SSAFunction> {
        self.try_get_func_id(id).and_then(|id| self.get_ssa_func(id))
    }

    pub fn dummy_id() -> arena::Index {
        arena::Index::from_raw_parts(std::usize::MAX, 0)
    }

    pub fn try_get_node(&self, id: NodeId) -> Option<&node::NodeObject> {
        self.nodes.get(id.0)
    }

    pub fn try_get_node_mut(&mut self, id: NodeId) -> Option<&mut node::NodeObject> {
        self.nodes.get_mut(id.0)
    }

    pub fn get_object_type(&self, id: NodeId) -> node::ObjectType {
        self[id].get_type()
    }

    //Returns the object value if it is a constant, None if not.
    pub fn get_as_constant(&self, id: NodeId) -> Option<FieldElement> {
        if let Some(node::NodeObject::Const(c)) = self.try_get_node(id) {
            return Some(FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()));
        }
        None
    }

    pub fn get_instruction(&self, id: NodeId) -> &node::Instruction {
        self.try_get_instruction(id).expect("Index not found or not an instruction")
    }

    pub fn get_mut_instruction(&mut self, id: NodeId) -> &mut node::Instruction {
        self.try_get_mut_instruction(id).expect("Index not found or not an instruction")
    }

    pub fn try_get_instruction(&self, id: NodeId) -> Option<&node::Instruction> {
        if let Some(NodeObject::Instr(i)) = self.try_get_node(id) {
            return Some(i);
        }
        None
    }

    pub fn try_get_mut_instruction(&mut self, id: NodeId) -> Option<&mut node::Instruction> {
        if let Some(NodeObject::Instr(i)) = self.try_get_node_mut(id) {
            return Some(i);
        }
        None
    }

    pub fn get_variable(&self, id: NodeId) -> Result<&node::Variable, RuntimeErrorKind> {
        match self.nodes.get(id.0) {
            Some(t) => match t {
                node::NodeObject::Obj(o) => Ok(o),
                _ => Err(RuntimeErrorKind::UnstructuredError {
                    message: "Not an object".to_string(),
                }),
            },
            _ => Err(RuntimeErrorKind::UnstructuredError { message: "Invalid id".to_string() }),
        }
    }

    pub fn get_mut_variable(
        &mut self,
        id: NodeId,
    ) -> Result<&mut node::Variable, RuntimeErrorKind> {
        match self.nodes.get_mut(id.0) {
            Some(t) => match t {
                node::NodeObject::Obj(o) => Ok(o),
                _ => Err(RuntimeErrorKind::UnstructuredError {
                    message: "Not an object".to_string(),
                }),
            },
            _ => Err(RuntimeErrorKind::UnstructuredError { message: "Invalid id".to_string() }),
        }
    }

    pub fn get_result_instruction_mut(
        &mut self,
        target: BlockId,
        call_instruction: NodeId,
        index: u32,
    ) -> Option<&mut Instruction> {
        for id in &self.blocks[target.0].instructions {
            if let Some(NodeObject::Instr(i)) = self.nodes.get(id.0) {
                if i.operation == (Operation::Result { call_instruction, index }) {
                    let id = *id;
                    return self.try_get_mut_instruction(id);
                }
            }
        }
        None
    }

    pub fn get_root_value(&self, id: NodeId) -> NodeId {
        self.get_variable(id).map(|v| v.get_root()).unwrap_or(id)
    }

    pub fn add_variable(&mut self, obj: node::Variable, root: Option<NodeId>) -> NodeId {
        let id = NodeId(self.nodes.insert(NodeObject::Obj(obj)));
        match &mut self[id] {
            node::NodeObject::Obj(v) => {
                v.id = id;
                v.root = root;
            }
            _ => unreachable!(),
        }
        id
    }

    pub fn update_variable_id_in_block(
        &mut self,
        var_id: NodeId,
        new_var: NodeId,
        new_value: NodeId,
        block_id: BlockId,
    ) {
        let root_id = self.get_root_value(var_id);
        let root = self.get_variable(root_id).unwrap();
        let root_name = root.name.clone();
        let cb = &mut self[block_id];
        cb.update_variable(var_id, new_value);
        let v_name = self.value_names.entry(var_id).or_insert(0);
        *v_name += 1;
        let variable_id = *v_name;

        if let Ok(new_var) = self.get_mut_variable(new_var) {
            new_var.name = format!("{root_name}{variable_id}");
        }
    }

    //Returns true if a may be distinct from b, and false else
    pub fn maybe_distinct(&self, a: NodeId, b: NodeId) -> bool {
        if a == NodeId::dummy() || b == NodeId::dummy() {
            return true;
        }
        if a == b {
            return false;
        }
        if let (Some(a_value), Some(b_value)) = (self.get_as_constant(a), self.get_as_constant(b)) {
            if a_value == b_value {
                return false;
            }
        }
        true
    }

    //Returns true if a may be equal to b, and false otherwise
    pub fn maybe_equal(&self, a: NodeId, b: NodeId) -> bool {
        if a == NodeId::dummy() || b == NodeId::dummy() {
            return true;
        }

        if a == b {
            return true;
        }
        if let (Some(a_value), Some(b_value)) = (self.get_as_constant(a), self.get_as_constant(b)) {
            if a_value != b_value {
                return false;
            }
        }
        true
    }

    //same as update_variable but using the var index instead of var
    pub fn update_variable_id(&mut self, var_id: NodeId, new_var: NodeId, new_value: NodeId) {
        self.update_variable_id_in_block(var_id, new_var, new_value, self.current_block);
    }

    pub fn new_instruction(
        &mut self,
        opcode: Operation,
        op_type: ObjectType,
    ) -> Result<NodeId, RuntimeError> {
        //Add a new instruction to the nodes arena
        let mut i = Instruction::new(opcode, op_type, Some(self.current_block));

        //Basic simplification - we ignore RunTimeErrors when creating an instruction
        //because they must be managed after handling conditionals. For instance if false { b } should not fail whatever b is doing.
        optimizations::simplify(self, &mut i).ok();

        if let Mark::ReplaceWith(replacement) = i.mark {
            return Ok(replacement);
        }
        Ok(self.push_instruction(i))
    }

    pub fn find_const_with_type(
        &self,
        value: &BigUint,
        e_type: node::ObjectType,
    ) -> Option<NodeId> {
        //TODO We should map constant values to id
        for (idx, o) in &self.nodes {
            if let node::NodeObject::Const(c) = o {
                if c.value == *value && c.get_type() == e_type {
                    return Some(NodeId(idx));
                }
            }
        }
        None
    }

    // Retrieve the object corresponding to the const value given in argument
    // If such object does not exist, we create one
    pub fn get_or_create_const(&mut self, x: FieldElement, t: node::ObjectType) -> NodeId {
        let value = BigUint::from_bytes_be(&x.to_be_bytes());
        if let Some(prev_const) = self.find_const_with_type(&value, t) {
            return prev_const;
        }

        self.add_const(node::Constant {
            id: NodeId::dummy(),
            value,
            value_str: String::new(),
            value_type: t,
        })
    }

    //Return the type of the operation result, based on the left hand type
    pub fn get_result_type(&self, op: &Operation, lhs_type: node::ObjectType) -> node::ObjectType {
        use {BinaryOp::*, Operation::*};
        match op {
            Binary(node::Binary { operator: Eq, .. })
            | Binary(node::Binary { operator: Ne, .. })
            | Binary(node::Binary { operator: Ult, .. })
            | Binary(node::Binary { operator: Ule, .. })
            | Binary(node::Binary { operator: Slt, .. })
            | Binary(node::Binary { operator: Sle, .. })
            | Binary(node::Binary { operator: Lt, .. })
            | Binary(node::Binary { operator: Lte, .. }) => ObjectType::Boolean,
            Operation::Jne(_, _)
            | Operation::Jeq(_, _)
            | Operation::Jmp(_)
            | Operation::Nop
            | Operation::Constrain(..)
            | Operation::Store { .. } => ObjectType::NotAnObject,
            Operation::Load { array_id, .. } => self.mem[*array_id].element_type,
            Operation::Cast(_) | Operation::Truncate { .. } => {
                unreachable!("cannot determine result type")
            }
            _ => lhs_type,
        }
    }

    pub fn new_array(
        &mut self,
        name: &str,
        element_type: ObjectType,
        len: u32,
        def: Option<Definition>,
    ) -> (NodeId, ArrayId) {
        let array_index = self.mem.create_new_array(len, element_type, name);
        self.add_dummy_load(array_index);
        self.add_dummy_store(array_index);
        //we create a variable pointing to this MemArray
        let new_var = node::Variable {
            id: NodeId::dummy(),
            obj_type: node::ObjectType::Pointer(array_index),
            name: name.to_string(),
            root: None,
            def: def.clone(),
            witness: None,
            parent_block: self.current_block,
        };
        if let Some(def) = def {
            self.mem[array_index].def = def;
        }
        (self.add_variable(new_var, None), array_index)
    }

    //returns the value of the element array[index], if it exists in the memory_map
    pub fn get_indexed_value(&self, array_id: ArrayId, index: NodeId) -> Option<&NodeId> {
        if let Some(idx) = Memory::to_u32(self, index) {
            self.mem.get_value_from_map(array_id, idx)
        } else {
            None
        }
    }
    //blocks/////////////////////////
    pub fn try_get_block_mut(&mut self, id: BlockId) -> Option<&mut block::BasicBlock> {
        self.blocks.get_mut(id.0)
    }

    pub fn get_current_block(&self) -> &block::BasicBlock {
        &self[self.current_block]
    }

    pub fn get_current_block_mut(&mut self) -> &mut block::BasicBlock {
        let current = self.current_block;
        &mut self[current]
    }

    pub fn iter_blocks(&self) -> impl Iterator<Item = &BasicBlock> {
        self.blocks.iter().map(|(_id, block)| block)
    }

    pub fn log(&self, show_log: bool, before: &str, after: &str) {
        if show_log {
            self.print(before);
            println!("{after}");
        }
    }

    //Optimize, flatten and truncate IR and then generates ACIR representation from it
    pub fn ir_to_acir(
        &mut self,
        evaluator: &mut Evaluator,
        enable_logging: bool,
    ) -> Result<(), RuntimeError> {
        //SSA
        self.log(enable_logging, "SSA:", "\ninline functions");
        function::inline_all(self)?;

        //Optimization
        block::compute_dom(self);
        optimizations::full_cse(self, self.first_block, false)?;

        //flattening
        self.log(enable_logging, "\nCSE:", "\nunrolling:");
        //Unrolling
        flatten::unroll_tree(self, self.first_block)?;
        //reduce conditionals
        let mut decision = DecisionTree::new(self);
        let builder = TreeBuilder::new(self.first_block);
        decision.make_decision_tree(self, builder)?;
        decision.reduce(self, decision.root)?;
        //Inlining
        self.log(enable_logging, "reduce", "\ninlining:");
        inline::inline_tree(self, self.first_block, &decision)?;

        block::merge_path(self, self.first_block, BlockId::dummy(), None)?;

        //The CFG is now fully flattened, so we keep only the first block.
        let mut to_remove = Vec::new();
        for b in &self.blocks {
            if b.0 != self.first_block.0 {
                to_remove.push(b.0);
            }
        }
        for b in to_remove {
            self.blocks.remove(b);
        }
        let first_block = self.first_block;
        self[first_block].dominated.clear();

        optimizations::cse(self, first_block, true)?;

        //Truncation
        integer::overflow_strategy(self)?;
        self.log(enable_logging, "\noverflow:", "");
        //ACIR
        self.acir(evaluator)?;
        if enable_logging {
            Acir::print_circuit(&evaluator.opcodes);
            println!("DONE");
        }
        println!("ACIR opcodes generated : {}", evaluator.opcodes.len());
        Ok(())
    }

    pub fn acir(&self, evaluator: &mut Evaluator) -> Result<(), RuntimeError> {
        let mut acir = Acir::default();
        let mut fb = Some(&self[self.first_block]);
        while let Some(block) = fb {
            for iter in &block.instructions {
                let ins = self.get_instruction(*iter);
                acir.evaluate_instruction(ins, evaluator, self).map_err(RuntimeError::from)?;
            }
            //TODO we should rather follow the jumps
            fb = block.left.map(|block_id| &self[block_id]);
        }
        Ok(())
    }

    pub fn generate_empty_phi(&mut self, target_block: BlockId, phi_root: NodeId) -> NodeId {
        //Ensure there is not already a phi for the variable (n.b. probably not useful)
        for i in &self[target_block].instructions {
            match self.try_get_instruction(*i) {
                Some(Instruction { operation: Operation::Phi { root, .. }, .. })
                    if *root == phi_root =>
                {
                    return *i;
                }
                _ => (),
            }
        }

        let v_type = self.get_object_type(phi_root);
        let operation = Operation::Phi { root: phi_root, block_args: vec![] };
        let new_phi = Instruction::new(operation, v_type, Some(target_block));
        let phi_id = self.add_instruction(new_phi);
        self[target_block].instructions.insert(1, phi_id);
        phi_id
    }

    fn memcpy(&mut self, l_type: ObjectType, r_type: ObjectType) -> Result<(), RuntimeError> {
        if l_type == r_type {
            return Ok(());
        }

        if let (ObjectType::Pointer(a), ObjectType::Pointer(b)) = (l_type, r_type) {
            let len = self.mem[a].len;
            let e_type = self.mem[b].element_type;
            for i in 0..len {
                let idx_b = self
                    .get_or_create_const(FieldElement::from(i as i128), ObjectType::Unsigned(32));
                let idx_a = self
                    .get_or_create_const(FieldElement::from(i as i128), ObjectType::Unsigned(32));
                let op_b = Operation::Load { array_id: b, index: idx_b };
                let load = self.new_instruction(op_b, e_type)?;
                let op_a = Operation::Store { array_id: a, index: idx_a, value: load };
                self.new_instruction(op_a, l_type)?;
            }
        } else {
            unreachable!("invalid type, expected arrays, got {:?} and {:?}", l_type, r_type);
        }

        Ok(())
    }

    //This function handles assignment statements of the form lhs = rhs, depending on the nature of the arguments:
    // lhs can be: standard variable, array, array element (in which case we have an index)
    // rhs can be: standard variable, array, array element (depending on lhs type), call instruction, intrinsic, other instruction
    // For instance:
    // - if lhs and rhs are standard variables, we create a new ssa variable of lhs
    // - if lhs is an array element, we generate a store instruction
    // - if lhs and rhs are arrays, we perform a copy of rhs into lhs,
    // - if lhs is an array and rhs is a call instruction, we indicate in the call that lhs is the returned array (so that no copy is needed because the inlining will use it)
    // ...
    pub fn handle_assign(
        &mut self,
        lhs: NodeId,
        index: Option<NodeId>,
        rhs: NodeId,
    ) -> Result<NodeId, RuntimeError> {
        let lhs_type = self.get_object_type(lhs);
        let rhs_type = self.get_object_type(rhs);

        let mut ret_array = None;
        if let Some(Instruction {
            operation: Operation::Result { call_instruction: func, index: idx },
            ..
        }) = self.try_get_instruction(rhs)
        {
            if index.is_none() {
                if let ObjectType::Pointer(a) = lhs_type {
                    ret_array = Some((*func, a, *idx));
                }
            }
        }

        if let Some((func, a, idx)) = ret_array {
            if let Some(Instruction {
                operation: Operation::Call { returned_arrays, arguments, .. },
                ..
            }) = self.try_get_mut_instruction(func)
            {
                returned_arrays.push((a, idx));
                //Issue #579: we initialize the array, unless it is also in arguments in which case it is already initialized.
                let mut init = false;
                for i in arguments.clone() {
                    if let ObjectType::Pointer(b) = self.get_object_type(i) {
                        if a == b {
                            init = true;
                        }
                    }
                }
                if !init {
                    let mut stack = StackFrame::new(self.current_block);
                    self.init_array(a, &mut stack);
                    let pos = self[self.current_block]
                        .instructions
                        .iter()
                        .position(|x| *x == func)
                        .unwrap();
                    let current_block = self.current_block;
                    for i in stack.stack {
                        self[current_block].instructions.insert(pos, i);
                    }
                }
            }
            if let Some(i) = self.try_get_mut_instruction(rhs) {
                i.mark = Mark::ReplaceWith(lhs);
            }
            return Ok(lhs);
        }

        if let Some(idx) = index {
            if let ObjectType::Pointer(a) = lhs_type {
                //Store
                let op_a = Operation::Store { array_id: a, index: idx, value: rhs };
                return self.new_instruction(op_a, self.mem[a].element_type);
            } else {
                unreachable!("Index expression must be for an array");
            }
        } else if matches!(lhs_type, ObjectType::Pointer(_)) {
            if let Some(Instruction {
                operation: Operation::Intrinsic(_, _),
                res_type: result_type,
                ..
            }) = self.try_get_mut_instruction(rhs)
            {
                *result_type = lhs_type;
                return Ok(lhs);
            } else {
                self.memcpy(lhs_type, rhs_type)?;
                return Ok(lhs);
            }
        }
        let lhs_obj = self.get_variable(lhs).unwrap();
        let new_var = node::Variable {
            id: lhs,
            obj_type: lhs_type,
            name: String::new(),
            root: None,
            def: lhs_obj.def.clone(),
            witness: None,
            parent_block: self.current_block,
        };
        let ls_root = lhs_obj.get_root();
        //ssa: we create a new variable a1 linked to a
        let new_var_id = self.add_variable(new_var, Some(ls_root));
        let op = Operation::Binary(node::Binary {
            lhs: new_var_id,
            rhs,
            operator: node::BinaryOp::Assign,
            predicate: None,
        });
        let result = self.new_instruction(op, rhs_type)?;
        self.update_variable_id(ls_root, new_var_id, result); //update the name and the value map
        Ok(new_var_id)
    }

    fn new_instruction_inline(
        &mut self,
        operation: node::Operation,
        op_type: node::ObjectType,
        stack_frame: &mut StackFrame,
    ) -> NodeId {
        let i = node::Instruction::new(operation, op_type, Some(stack_frame.block));
        let ins_id = self.add_instruction(i);
        stack_frame.push(ins_id);
        ins_id
    }

    fn init_array(&mut self, array_id: ArrayId, stack_frame: &mut StackFrame) {
        let len = self.mem[array_id].len;
        let e_type = self.mem[array_id].element_type;
        for i in 0..len {
            let index =
                self.get_or_create_const(FieldElement::from(i as i128), ObjectType::Unsigned(32));
            let op_a = Operation::Store { array_id, index, value: self.zero_with_type(e_type) };
            self.new_instruction_inline(op_a, e_type, stack_frame);
        }
    }

    pub fn memcpy_inline(
        &mut self,
        l_type: ObjectType,
        r_type: ObjectType,
        stack_frame: &mut StackFrame,
    ) {
        if l_type == r_type {
            return;
        }

        if let (ObjectType::Pointer(a), ObjectType::Pointer(b)) = (l_type, r_type) {
            let len = self.mem[a].len;
            let e_type = self.mem[b].element_type;
            for i in 0..len {
                let idx_b = self
                    .get_or_create_const(FieldElement::from(i as i128), ObjectType::Unsigned(32));
                let idx_a = self
                    .get_or_create_const(FieldElement::from(i as i128), ObjectType::Unsigned(32));
                let op_b = Operation::Load { array_id: b, index: idx_b };
                let load = self.new_instruction_inline(op_b, e_type, stack_frame);
                let op_a = Operation::Store { array_id: a, index: idx_a, value: load };
                self.new_instruction_inline(op_a, l_type, stack_frame);
            }
        } else {
            unreachable!("invalid type, expected arrays");
        }
    }

    pub fn handle_assign_inline(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        stack_frame: &mut inline::StackFrame,
        block_id: BlockId,
    ) -> NodeId {
        let lhs_type = self.get_object_type(lhs);
        let rhs_type = self.get_object_type(rhs);
        if let ObjectType::Pointer(a) = lhs_type {
            //Array
            let b = stack_frame.get_or_default(a);
            self.memcpy_inline(ObjectType::Pointer(b), rhs_type, stack_frame);
            lhs
        } else {
            //new ssa
            let lhs_obj = self.get_variable(lhs).unwrap();
            let new_var = node::Variable {
                id: NodeId::dummy(),
                obj_type: lhs_type,
                name: String::new(),
                root: None,
                def: lhs_obj.def.clone(),
                witness: None,
                parent_block: self.current_block,
            };
            let ls_root = lhs_obj.get_root();
            //ssa: we create a new variable a1 linked to a
            let new_var_id = self.add_variable(new_var, Some(ls_root));
            //ass
            let op = Operation::Binary(node::Binary {
                lhs: new_var_id,
                rhs,
                operator: node::BinaryOp::Assign,
                predicate: None,
            });
            let result = self.new_instruction_inline(op, rhs_type, stack_frame);
            self.update_variable_id_in_block(ls_root, new_var_id, result, block_id); //update the name and the value map
            result
        }
    }

    pub fn under_assumption(&self, predicate: NodeId) -> bool {
        !(predicate == NodeId::dummy() || predicate == self.one())
    }

    //Returns the instruction used by a IF statement. None if the block is not a IF block.
    pub fn get_if_condition(&self, block: &BasicBlock) -> Option<&node::Instruction> {
        if let Some(ins) = self.try_get_instruction(*block.instructions.last().unwrap()) {
            if !block.is_join() && ins.operation.opcode() == super::node::Opcode::Jeq {
                return Some(ins);
            }
        }
        None
    }

    //Generate a new variable v and a phi instruction s.t. v = phi(a,b);
    // c is a counter used to name the variable v for debugging purposes
    // when a and b are pointers, we create a new array s.t v[i] = phi(a[i],b[i])
    pub fn new_phi(&mut self, a: NodeId, b: NodeId, c: &mut u32) -> NodeId {
        if a == NodeId::dummy() || b == NodeId::dummy() {
            return NodeId::dummy();
        }

        let exit_block = self.current_block;
        let block1 = self[exit_block].predecessor[0];
        let block2 = self[exit_block].predecessor[1];

        let a_type = self.get_object_type(a);

        let name = format!("if_{}_ret{c}", exit_block.0.into_raw_parts().0);
        *c += 1;
        if let node::ObjectType::Pointer(adr1) = a_type {
            let len = self.mem[adr1].len;
            let el_type = self.mem[adr1].element_type;
            let (id, array_id) = self.new_array(&name, el_type, len, None);
            for i in 0..len {
                let index = self
                    .get_or_create_const(FieldElement::from(i as u128), ObjectType::NativeField);
                self.current_block = block1;
                let op = Operation::Load { array_id: adr1, index };
                let v1 = self.new_instruction(op, el_type).unwrap();
                self.current_block = block2;
                let adr2 = super::mem::Memory::deref(self, b).unwrap();
                let op = Operation::Load { array_id: adr2, index };
                let v2 = self.new_instruction(op, el_type).unwrap();
                self.current_block = exit_block;
                let v = self.new_phi(v1, v2, c);
                let op = Operation::Store { array_id, index, value: v };
                self.new_instruction(op, el_type).unwrap();
            }
            id
        } else {
            let new_var = node::Variable::new(a_type, name, None, exit_block);
            let v = self.add_variable(new_var, None);
            let operation = Operation::Phi { root: v, block_args: vec![(a, block1), (b, block2)] };
            let new_phi = node::Instruction::new(operation, a_type, Some(exit_block));
            let phi_id = self.add_instruction(new_phi);
            self[exit_block].instructions.insert(1, phi_id);
            phi_id
        }
    }

    pub fn push_function_id(&mut self, func_id: FuncId, name: &str) -> NodeId {
        let index = self.nodes.insert_with(|index| {
            let node_id = NodeId(index);
            NodeObject::Function(FunctionKind::Normal(func_id), node_id, name.to_owned())
        });

        NodeId(index)
    }

    /// Return the standard NodeId for this FuncId.
    /// The 'standard' NodeId is just the NodeId assigned to the function when it
    /// is first compiled so that repeated NodeObjs are not made for the same function.
    /// If this function returns None, it means the given FuncId has yet to be compiled.
    pub fn get_function_node_id(&self, func_id: FuncId) -> Option<NodeId> {
        self.functions.get(&func_id).map(|f| f.node_id)
    }

    pub fn function_already_compiled(&self, func_id: FuncId) -> bool {
        self.get_ssa_func(func_id).is_some()
    }

    pub fn get_or_create_opcode_node_id(&mut self, opcode: builtin::Opcode) -> NodeId {
        if let Some(id) = self.opcode_ids.get(&opcode) {
            return *id;
        }

        let index = self.nodes.insert_with(|index| {
            NodeObject::Function(FunctionKind::Builtin(opcode), NodeId(index), opcode.to_string())
        });
        self.opcode_ids.insert(opcode, NodeId(index));
        NodeId(index)
    }

    pub fn get_builtin_opcode(&self, node_id: NodeId) -> Option<builtin::Opcode> {
        match &self[node_id] {
            NodeObject::Function(FunctionKind::Builtin(opcode), ..) => Some(*opcode),
            _ => None,
        }
    }

    pub fn convert_type(&mut self, t: &noirc_frontend::monomorphization::ast::Type) -> ObjectType {
        use noirc_frontend::monomorphization::ast::Type;
        use noirc_frontend::Signedness;
        match t {
            Type::Bool => ObjectType::Boolean,
            Type::Field => ObjectType::NativeField,
            Type::Integer(sign, bit_size) => {
                assert!(
                    *bit_size < super::integer::short_integer_max_bit_size(),
                    "long integers are not yet supported"
                );
                match sign {
                    Signedness::Signed => ObjectType::Signed(*bit_size),
                    Signedness::Unsigned => ObjectType::Unsigned(*bit_size),
                }
            }
            Type::Array(..) => panic!("Cannot convert an array type {t} into an ObjectType since it is unknown which array it refers to"),
            Type::Unit => ObjectType::NotAnObject,
            Type::Function(..) => ObjectType::Function,
            Type::Tuple(_) => todo!("Conversion to ObjectType is unimplemented for tuples"),
            Type::String(_) => todo!("Conversion to ObjectType is unimplemented for strings"),
        }
    }

    pub fn add_predicate(
        &mut self,
        pred: NodeId,
        instruction: &mut Instruction,
        stack: &mut StackFrame,
    ) {
        let op = &mut instruction.operation;

        match op {
            Operation::Binary(bin) => {
                assert!(bin.predicate.is_none());
                let cond = if let Some(pred_ins) = bin.predicate {
                    assert_ne!(pred_ins, NodeId::dummy());
                    if pred == NodeId::dummy() {
                        pred_ins
                    } else {
                        let op = Operation::Binary(node::Binary {
                            lhs: pred,
                            rhs: pred_ins,
                            operator: BinaryOp::Mul,
                            predicate: None,
                        });
                        let cond = self.add_instruction(Instruction::new(
                            op,
                            ObjectType::Boolean,
                            Some(stack.block),
                        ));
                        optimizations::simplify_id(self, cond).unwrap();
                        stack.push(cond);
                        cond
                    }
                } else {
                    pred
                };
                bin.predicate = Some(cond);
            }
            Operation::Constrain(cond, _) => {
                let operation =
                    Operation::Cond { condition: pred, val_true: *cond, val_false: self.one() };
                let c_ins = self.add_instruction(Instruction::new(
                    operation,
                    ObjectType::Boolean,
                    Some(stack.block),
                ));
                stack.push(c_ins);
                *cond = c_ins;
            }
            _ => unreachable!(),
        }
    }
}

impl std::ops::Index<BlockId> for SsaContext {
    type Output = BasicBlock;

    fn index(&self, index: BlockId) -> &Self::Output {
        &self.blocks[index.0]
    }
}

impl std::ops::IndexMut<BlockId> for SsaContext {
    fn index_mut(&mut self, index: BlockId) -> &mut Self::Output {
        &mut self.blocks[index.0]
    }
}

impl std::ops::Index<NodeId> for SsaContext {
    type Output = NodeObject;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl std::ops::IndexMut<NodeId> for SsaContext {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}
