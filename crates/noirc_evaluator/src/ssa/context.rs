use super::block::{BasicBlock, BlockId};
use super::conditional::DecisionTree;
use super::function::{FuncIndex, SSAFunction};
use super::inline::StackFrame;
use super::node::{BinaryOp, Instruction, NodeId, NodeObj, ObjectType, Operation};
use super::{block, flatten, inline, integer, node, optim};
use std::collections::{HashMap, HashSet};

use super::super::errors::RuntimeError;
use crate::ssa::acir_gen::Acir;
use crate::ssa::function;
use crate::ssa::node::{Mark, Node};
use crate::Evaluator;
use acvm::FieldElement;
use noirc_frontend::hir::Context;
use noirc_frontend::node_interner::FuncId;
use noirc_frontend::util::vecmap;

// This is a 'master' class for generating the SSA IR from the AST
// It contains all the data; the node objects representing the source code in the nodes arena
// and The CFG in the blocks arena
// everything else just reference objects from these two arena using their index.
pub struct SsaContext<'a> {
    pub context: &'a Context,
    pub first_block: BlockId,
    pub current_block: BlockId,
    blocks: arena::Arena<block::BasicBlock>,
    pub nodes: arena::Arena<node::NodeObj>,
    value_names: HashMap<NodeId, u32>,
    pub sealed_blocks: HashSet<BlockId>,
    pub functions: HashMap<FuncId, function::SSAFunction>,
    //Adjacency Matrix of the call graph; list of rows where each row indicates the functions called by the function whose FuncIndex is the row number
    pub call_graph: Vec<Vec<u8>>,
}

impl<'a> SsaContext<'a> {
    pub fn new(context: &Context) -> SsaContext {
        let mut pc = SsaContext {
            context,
            first_block: BlockId::dummy(),
            current_block: BlockId::dummy(),
            blocks: arena::Arena::new(),
            nodes: arena::Arena::new(),
            value_names: HashMap::new(),
            sealed_blocks: HashSet::new(),
            functions: HashMap::new(),
            call_graph: Vec::new(),
        };
        block::create_first_block(&mut pc);
        pc.one_with_type(ObjectType::BOOL);
        pc.zero_with_type(ObjectType::BOOL);
        pc
    }

    pub fn zero(&self) -> NodeId {
        self.find_const_with_type(&FieldElement::zero(), ObjectType::BOOL).unwrap()
    }

    pub fn one(&self) -> NodeId {
        self.find_const_with_type(&FieldElement::one(), ObjectType::BOOL).unwrap()
    }

    pub fn zero_with_type(&mut self, obj_type: ObjectType) -> NodeId {
        self.get_or_create_const(FieldElement::zero(), obj_type)
    }

    pub fn one_with_type(&mut self, obj_type: ObjectType) -> NodeId {
        self.get_or_create_const(FieldElement::one(), obj_type)
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

    //Display an object for debugging puposes
    fn node_to_string(&self, id: NodeId) -> String {
        if let Some(var) = self.try_get_node(id) {
            match &var {
                NodeObj::Const(constant) => {
                    match &constant.value {
                        node::ConstantValue::Array(array) => {
                            let values = vecmap(&array.values, |id| self.node_to_string(*id));
                            format!("[{}]", values.join(", "))
                        },
                        other => format!("{}", other),
                    }
                },
                other => format!("{}", other),
            }
        } else {
            format!("unknown {:?}", id.0.into_raw_parts().0)
        }
    }

    fn binary_to_string(&self, binary: &node::Binary) -> String {
        let lhs = self.node_to_string(binary.lhs);
        let rhs = self.node_to_string(binary.rhs);
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

        format!("{} {}, {}", op, lhs, rhs)
    }

    pub fn operation_to_string(&self, op: &Operation) -> String {
        let join = |args: &[NodeId]| vecmap(args, |arg| self.node_to_string(*arg)).join(", ");

        match op {
            Operation::Binary(binary) => self.binary_to_string(binary),
            Operation::Cast(value) => format!("cast {}", self.node_to_string(*value)),
            Operation::Truncate { value, bit_size, max_bit_size } => {
                format!(
                    "truncate {}, bitsize = {}, max bitsize = {}",
                    self.node_to_string(*value),
                    bit_size,
                    max_bit_size
                )
            }
            Operation::Not(v) => format!("not {}", self.node_to_string(*v)),
            Operation::Constrain(v, ..) => format!("constrain {}", self.node_to_string(*v)),
            Operation::Jne(v, b) => format!("jne {}, {:?}", self.node_to_string(*v), b),
            Operation::Jeq(v, b) => format!("jeq {}, {:?}", self.node_to_string(*v), b),
            Operation::Jmp(b) => format!("jmp {:?}", b),
            Operation::Phi { root, block_args } => {
                let mut s = format!("phi {}", self.node_to_string(*root));
                for (value, block) in block_args {
                    s = format!(
                        "{}, {} from block {}",
                        s,
                        self.node_to_string(*value),
                        block.0.into_raw_parts().0
                    );
                }
                s
            }
            Operation::Cond { condition, val_true: lhs, val_false: rhs } => {
                let lhs = self.node_to_string(*lhs);
                let rhs = self.node_to_string(*rhs);
                format!("cond({}) {}, {}", self.node_to_string(*condition), lhs, rhs)
            }
            Operation::Load { array, index } => {
                format!(
                    "load {:?}, index {}",
                    self.node_to_string(*array),
                    self.node_to_string(*index)
                )
            }
            Operation::Store { array, index, value } => {
                format!(
                    "store {:?}, index {}, value {}",
                    self.node_to_string(*array),
                    self.node_to_string(*index),
                    self.node_to_string(*value)
                )
            }
            Operation::Intrinsic(opcode, args) => format!("intrinsic {}({})", opcode, join(args)),
            Operation::Nop => "nop".into(),
            Operation::Call(f, args) => format!("call {:?}({})", f, join(args)),
            Operation::Return(values) => format!("return ({})", join(values)),
            Operation::Result { call_instruction, index } => {
                let call = self.node_to_string(*call_instruction);
                format!("result {} of {}", index, call)
            }
        }
    }

    pub fn print_block(&self, b: &block::BasicBlock) {
        println!("************* Block n.{}", b.id.0.into_raw_parts().0);
        println!("Assumption:{:?}", b.assumption);
        for id in &b.instructions {
            let ins = self.get_instruction(*id);
            let mut str_res = if ins.res_name.is_empty() {
                format!("{:?}", id.0.into_raw_parts().0)
            } else {
                ins.res_name.clone()
            };
            if let Mark::ReplaceWith(replacement) = ins.mark {
                str_res = format!("{} -REPLACED with id {:?}", str_res, replacement.0);
            } else if ins.is_deleted() {
                str_res = format!("{}: DELETED", str_res);
            }
            let ins_str = self.operation_to_string(&ins.operation);
            println!("{}: {}", str_res, ins_str);
        }
    }

    pub fn print(&self, text: &str) {
        println!("{}", text);
        for (_, b) in self.blocks.iter() {
            self.print_block(b);
        }
    }

    pub fn context(&self) -> &'a Context {
        self.context
    }

    pub fn remove_block(&mut self, block: BlockId) {
        self.blocks.remove(block.0);
    }

    /// Add an instruction to self.nodes and sets its id.
    /// This function does NOT push the instruction to the current block.
    /// See push_instruction for that.
    pub fn add_instruction(&mut self, instruction: node::Instruction) -> NodeId {
        let obj = NodeObj::Instr(instruction);
        let id = NodeId(self.nodes.insert(obj));
        match &mut self[id] {
            NodeObj::Instr(i) => i.id = id,
            _ => unreachable!(),
        }

        id
    }

    /// Adds the instruction to self.nodes and pushes it to the current block
    pub fn push_instruction(&mut self, instruction: node::Instruction) -> NodeId {
        let id = self.add_instruction(instruction);
        if let NodeObj::Instr(_) = &self[id] {
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

    pub fn add_const(&mut self, constant: node::Constant) -> NodeId {
        let obj = NodeObj::Const(constant);
        let id = NodeId(self.nodes.insert(obj));
        match &mut self[id] {
            node::NodeObj::Const(c) => c.id = id,
            _ => unreachable!(),
        }

        id
    }

    pub fn get_ssafunc(&'a self, func_id: FuncId) -> Option<&SSAFunction> {
        self.functions.get(&func_id)
    }

    pub fn dummy_id() -> arena::Index {
        arena::Index::from_raw_parts(std::usize::MAX, 0)
    }

    pub fn try_get_node(&self, id: NodeId) -> Option<&node::NodeObj> {
        self.nodes.get(id.0)
    }

    pub fn try_get_node_mut(&mut self, id: NodeId) -> Option<&mut node::NodeObj> {
        self.nodes.get_mut(id.0)
    }

    pub fn get_object_type(&self, id: NodeId) -> node::ObjectType {
        self[id].get_type()
    }

    //Returns the object value if it is a constant, None if not. TODO: handle types
    pub fn get_as_constant(&self, id: NodeId) -> Option<FieldElement> {
        if let Some(node::NodeObj::Const(c)) = self.try_get_node(id) {
            match &c.value {
                node::ConstantValue::Field(field) => return Some(field.clone()),
                _ => (),
            }
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
        if let Some(NodeObj::Instr(i)) = self.try_get_node(id) {
            return Some(i);
        }
        None
    }

    pub fn try_get_mut_instruction(&mut self, id: NodeId) -> Option<&mut node::Instruction> {
        if let Some(NodeObj::Instr(i)) = self.try_get_node_mut(id) {
            return Some(i);
        }
        None
    }

    pub fn get_variable(&self, id: NodeId) -> Result<&node::Variable, &str> {
        //TODO proper error handling
        match self.nodes.get(id.0) {
            Some(t) => match t {
                node::NodeObj::Obj(o) => Ok(o),
                _ => Err("Not an object"),
            },
            _ => Err("Invalid id"),
        }
    }

    pub fn get_mut_variable(&mut self, id: NodeId) -> Result<&mut node::Variable, &str> {
        //TODO proper error handling
        match self.nodes.get_mut(id.0) {
            Some(t) => match t {
                node::NodeObj::Obj(o) => Ok(o),
                _ => Err("Not an object"),
            },
            _ => Err("Invalid id"),
        }
    }

    pub fn get_result_instruction_mut(
        &mut self,
        target: BlockId,
        call_instruction: NodeId,
        index: u32,
    ) -> Option<&mut Instruction> {
        for id in &self.blocks[target.0].instructions {
            if let Some(NodeObj::Instr(i)) = self.nodes.get(id.0) {
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
        let id = NodeId(self.nodes.insert(NodeObj::Obj(obj)));
        match &mut self[id] {
            node::NodeObj::Obj(v) => {
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

        if let Ok(nvar) = self.get_mut_variable(new_var) {
            nvar.name = format!("{}{}", root_name, variable_id);
        }
    }

    //same as update_variable but using the var index instead of var
    pub fn update_variable_id(&mut self, var_id: NodeId, new_var: NodeId, new_value: NodeId) {
        self.update_variable_id_in_block(var_id, new_var, new_value, self.current_block);
    }

    pub fn new_instruction(
        &mut self,
        opcode: Operation,
        optype: ObjectType,
    ) -> Result<NodeId, RuntimeError> {
        //Add a new instruction to the nodes arena
        let mut i = Instruction::new(opcode, optype, Some(self.current_block));
        //Basic simplification
        optim::simplify(self, &mut i)?;

        if let Mark::ReplaceWith(replacement) = i.mark {
            return Ok(replacement);
        }
        Ok(self.push_instruction(i))
    }

    pub fn new_binary_instruction(
        &mut self,
        operator: BinaryOp,
        lhs: NodeId,
        rhs: NodeId,
        optype: ObjectType,
    ) -> Result<NodeId, RuntimeError> {
        let operation = Operation::binary(operator, lhs, rhs);
        self.new_instruction(operation, optype)
    }

    pub fn find_const_with_type(
        &self,
        value: &FieldElement,
        e_type: node::ObjectType,
    ) -> Option<NodeId> {
        //TODO We should map constant values to id
        for (idx, o) in &self.nodes {
            if let node::NodeObj::Const(c) = o {
                if c.get_type() == e_type {
                    match &c.value {
                        node::ConstantValue::Field(field) if field == value => {
                            return Some(NodeId(idx))
                        }
                        _ => (),
                    }
                }
            }
        }
        None
    }

    // Retrieve the object conresponding to the const value given in argument
    // If such object does not exist, we create one
    pub fn get_or_create_const(&mut self, value: FieldElement, typ: node::ObjectType) -> NodeId {
        if let Some(prev_const) = self.find_const_with_type(&value, typ) {
            return prev_const;
        }

        let value = node::ConstantValue::Field(value);
        self.add_const(node::Constant { id: NodeId::dummy(), value, value_type: typ })
    }

    pub fn new_array(&mut self, element_type: ObjectType, values: Vec<NodeId>) -> NodeId {
        let values = vecmap(values, |value| {
            let var = self.add_variable(node::Variable::new(element_type, "".into(), None, self.current_block), None);
            self.update_variable_id(var, NodeId::dummy(), value);
            var
        });
        let array = node::Array { element_type, values };
        let value = node::ConstantValue::Array(array);
        self.add_const(node::Constant { id: NodeId::dummy(), value, value_type: ObjectType::Array })
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
            println!("{}", after);
        }
    }

    //Optimise, flatten and truncate IR and then generates ACIR representation from it
    pub fn ir_to_acir(
        &mut self,
        evaluator: &mut Evaluator,
        enable_logging: bool,
    ) -> Result<(), RuntimeError> {
        //SSA
        self.log(enable_logging, "SSA:", "\ninline functions");
        function::inline_all(self)?;

        //Optimisation
        block::compute_dom(self);
        optim::full_cse(self, self.first_block)?;
        self.log(enable_logging, "\nCSE:", "\nunrolling:");
        //Unrolling
        flatten::unroll_tree(self, self.first_block)?;

        //reduce conditionals
        let mut decision = DecisionTree::new(self);
        decision.make_decision_tree(self);
        decision.reduce(self, decision.root)?;

        //Inlining
        self.log(enable_logging, "reduce", "\ninlining:");
        inline::inline_tree(self, self.first_block)?;
        optim::full_cse(self, self.first_block)?;

        //Truncation
        integer::overflow_strategy(self)?;
        self.log(enable_logging, "\noverflow:", "");
        //ACIR
        self.acir(evaluator);
        if enable_logging {
            Acir::print_circuit(&evaluator.gates);
            println!("DONE");
            dbg!(&evaluator.current_witness_index);
        }
        Ok(())
    }

    pub fn acir(&self, evaluator: &mut Evaluator) {
        let mut acir = Acir::default();
        let mut fb = Some(&self[self.first_block]);
        while let Some(block) = fb {
            for iter in &block.instructions {
                let ins = self.get_instruction(*iter);
                acir.evaluate_instruction(ins, evaluator, self);
            }
            //TODO we should rather follow the jumps
            fb = block.left.map(|block_id| &self[block_id]);
        }
    }

    pub fn generate_empty_phi(&mut self, target_block: BlockId, phi_root: NodeId) -> NodeId {
        //Ensure there is not already a phi for the variable (n.b. probably not usefull)
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

    fn new_instruction_inline(
        &mut self,
        operation: node::Operation,
        optype: node::ObjectType,
        stack_frame: &mut StackFrame,
    ) -> NodeId {
        let i = node::Instruction::new(operation, optype, Some(stack_frame.block));
        let ins_id = self.add_instruction(i);
        stack_frame.push(ins_id);
        ins_id
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

        let lhs_obj = self.get_variable(lhs).unwrap();
        let new_var = node::Variable::new(lhs_type, "".into(), lhs_obj.def, self.current_block);

        let ls_root = lhs_obj.get_root();
        //ssa: we create a new variable a1 linked to a
        let new_var_id = self.add_variable(new_var, Some(ls_root));
        //ass
        let op = Operation::Binary(node::Binary {
            lhs: new_var_id,
            rhs,
            operator: node::BinaryOp::Assign,
        });
        let result = self.new_instruction_inline(op, rhs_type, stack_frame);
        self.update_variable_id_in_block(ls_root, new_var_id, result, block_id); //update the name and the value map
        result
    }
}

impl std::ops::Index<BlockId> for SsaContext<'_> {
    type Output = BasicBlock;

    fn index(&self, index: BlockId) -> &Self::Output {
        &self.blocks[index.0]
    }
}

impl std::ops::IndexMut<BlockId> for SsaContext<'_> {
    fn index_mut(&mut self, index: BlockId) -> &mut Self::Output {
        &mut self.blocks[index.0]
    }
}

impl std::ops::Index<NodeId> for SsaContext<'_> {
    type Output = NodeObj;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl std::ops::IndexMut<NodeId> for SsaContext<'_> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}
