use super::block::{BasicBlock, BlockId};
use super::mem::Memory;
use super::node::{BinaryOp, Instruction, NodeId, NodeObj, ObjectType, Operation};
use super::{block, flatten, integer, node, optim};
use std::collections::{HashMap, HashSet};

use super::super::errors::RuntimeError;
use crate::ssa::acir_gen::Acir;
use crate::ssa::function;
use crate::ssa::node::Node;
use crate::Evaluator;
use acvm::FieldElement;
use noirc_frontend::hir::Context;
use noirc_frontend::node_interner::FuncId;
use noirc_frontend::util::vecmap;
use num_bigint::BigUint;

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
    pub sealed_blocks: HashSet<BlockId>,
    pub mem: Memory,
    pub functions_cfg: HashMap<FuncId, function::SSAFunction<'a>>,
}

impl<'a> SsaContext<'a> {
    pub fn new(context: &Context) -> SsaContext {
        let mut pc = SsaContext {
            context,
            first_block: BlockId::dummy(),
            current_block: BlockId::dummy(),
            blocks: arena::Arena::new(),
            nodes: arena::Arena::new(),
            sealed_blocks: HashSet::new(),
            mem: Memory::default(),
            functions_cfg: HashMap::new(),
        };
        block::create_first_block(&mut pc);
        pc.get_or_create_const(FieldElement::one(), node::ObjectType::Unsigned(1));
        pc.get_or_create_const(FieldElement::zero(), node::ObjectType::Unsigned(1));
        pc
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
            return format!("{}", var);
        } else {
            return format!("unknown {:?}", id.0.into_raw_parts().0);
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
            BinaryOp::Constrain(node::ConstrainOp::Eq) => "constrain_eq",
            BinaryOp::Constrain(node::ConstrainOp::Neq) => "constrain_neq",
        };

        format!("{} {}, {}", op, lhs, rhs)
    }

    fn operation_to_string(&self, op: &Operation) -> String {
        let join = |args: &[NodeId]| vecmap(args, |arg| self.node_to_string(*arg)).join(", ");

        match op {
            Operation::Binary(binary) => self.binary_to_string(binary),
            Operation::Cast(value) => format!("cast {}", self.node_to_string(*value)),
            Operation::Truncate {
                value,
                bit_size,
                max_bit_size,
            } => {
                format!(
                    "truncate {}, bitsize = {}, max bitsize = {}",
                    self.node_to_string(*value),
                    bit_size,
                    max_bit_size
                )
            }
            Operation::Not(v) => format!("not {}", self.node_to_string(*v)),
            Operation::Jne(v, b) => format!("jne {}, {:?}", self.node_to_string(*v), b),
            Operation::Jeq(v, b) => format!("jeq {}, {:?}", self.node_to_string(*v), b),
            Operation::Jmp(b) => format!("jmp {:?}", b),
            Operation::Phi { root, block_args } => {
                let mut s = format!("phi {}", self.node_to_string(*root));
                for (value, block) in block_args {
                    s += &format!(", {} from {:?}", self.node_to_string(*value), block);
                }
                s
            }
            Operation::Load { array_id, index } => {
                format!("load {:?}, index {}", array_id, self.node_to_string(*index))
            }
            Operation::Store {
                array_id,
                index,
                value,
            } => {
                format!(
                    "store {:?}, index {}, value {}",
                    array_id,
                    self.node_to_string(*index),
                    self.node_to_string(*value)
                )
            }
            Operation::Intrinsic(opcode, args) => format!("intrinsic {}({})", opcode, join(args)),
            Operation::Nop => format!("nop"),
            Operation::Call(f, args) => format!("call {:?}({})", f, join(args)),
            Operation::Return(values) => format!("return ({})", join(values)),
            Operation::Results {
                call_instruction,
                results,
            } => {
                let call = self.node_to_string(*call_instruction);
                format!("results {} = ({})", call, join(results))
            }
        }
    }

    pub fn print_block(&self, b: &block::BasicBlock) {
        for id in &b.instructions {
            let ins = self.get_instruction(*id);
            let mut str_res = if ins.res_name.is_empty() {
                format!("{:?}", id.0.into_raw_parts().0)
            } else {
                ins.res_name.clone()
            };
            if let Some(replacement) = ins.replacement {
                str_res = format!("{} -REPLACED with id {:?}", str_res, replacement.0);
            }

            let ins_str = self.operation_to_string(&ins.operator);
            println!("{}: {}", str_res, ins_str);
        }
    }

    pub fn print(&self, text: &str) {
        println!("{}", text);
        for (i, (_, b)) in self.blocks.iter().enumerate() {
            println!("************* Block n.{}", i);
            self.print_block(b);
        }
        for (_, f) in self.functions_cfg.iter().enumerate() {
            println!("************* FUNCTION n.{:?}", f.1.id);
            f.1.igen.context.print("");
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

    pub fn add_const(&mut self, constant: node::Constant) -> NodeId {
        let obj = NodeObj::Const(constant);
        let id = NodeId(self.nodes.insert(obj));
        match &mut self[id] {
            node::NodeObj::Const(c) => c.id = id,
            _ => unreachable!(),
        }

        id
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
            return Some(FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()));
        }
        None
    }

    pub fn get_function_context(&self, func_id: FuncId) -> &SsaContext {
        &self.functions_cfg[&func_id].igen.context
    }

    //todo handle errors
    fn get_instruction(&self, id: NodeId) -> &node::Instruction {
        self.try_get_instruction(id)
            .expect("Index not found or not an instruction")
    }

    pub fn get_mut_instruction(&mut self, id: NodeId) -> &mut node::Instruction {
        self.try_get_mut_instruction(id)
            .expect("Index not found or not an instruction")
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

    pub fn new_instruction(&mut self, opcode: Operation, optype: ObjectType) -> NodeId {
        //Add a new instruction to the nodes arena
        let mut i = Instruction::new(opcode, optype, Some(self.current_block));
        //Basic simplification
        optim::simplify(self, &mut i);

        if let Some(replacement) = i.replacement {
            return replacement;
        }
        self.push_instruction(i)
    }

    pub fn new_binary_instruction(
        &mut self,
        operator: BinaryOp,
        lhs: NodeId,
        rhs: NodeId,
        optype: ObjectType,
    ) -> NodeId {
        let operation = Operation::binary(operator, lhs, rhs);
        self.new_instruction(operation, optype)
    }

    pub fn find_const_with_type(
        &self,
        value: &BigUint,
        e_type: node::ObjectType,
    ) -> Option<NodeId> {
        //TODO We should map constant values to id
        for (idx, o) in &self.nodes {
            if let node::NodeObj::Const(c) = o {
                if c.value == *value && c.get_type() == e_type {
                    return Some(NodeId(idx));
                }
            }
        }
        None
    }

    // Retrieve the object conresponding to the const value given in argument
    // If such object does not exist, we create one
    pub fn get_or_create_const(&mut self, x: FieldElement, t: node::ObjectType) -> NodeId {
        let value = BigUint::from_bytes_be(&x.to_bytes()); //TODO a const should be a field element
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
            | Binary(node::Binary {
                operator: Constrain(_),
                ..
            })
            | Operation::Store { .. } => ObjectType::NotAnObject,
            Operation::Load { array_id, .. } => self.mem[*array_id].element_type,
            Operation::Cast(_) | Operation::Truncate { .. } => {
                unreachable!("cannot determine result type")
            }
            _ => lhs_type,
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

    pub fn pause(&self, interactive: bool, before: &str, after: &str) {
        if_debug::if_debug!(if interactive {
            self.print(before);
            let mut number = String::new();
            println!("Press enter to continue");
            std::io::stdin().read_line(&mut number).unwrap();
            println!("{}", after);
        });
    }

    //Optimise, flatten and truncate IR and then generates ACIR representation from it
    pub fn ir_to_acir(
        &mut self,
        evaluator: &mut Evaluator,
        interactive: bool,
    ) -> Result<(), RuntimeError> {
        //SSA
        self.pause(interactive, "SSA:", "CSE:");

        //Optimisation
        block::compute_dom(self);
        optim::cse(self);
        self.pause(interactive, "", "unrolling:");
        //Unrolling
        flatten::unroll_tree(self);
        self.pause(interactive, "", "inlining:");
        flatten::inline_tree(self);
        optim::cse(self);
        //Truncation
        integer::overflow_strategy(self);
        self.pause(interactive, "overflow:", "");
        //ACIR
        self.acir(evaluator);
        if_debug::if_debug!(if interactive {
            dbg!("DONE");
            dbg!(&evaluator.current_witness_index);
        });
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
        Acir::print_circuit(&evaluator.gates);
    }

    pub fn generate_empty_phi(&mut self, target_block: BlockId, phi_root: NodeId) -> NodeId {
        //Ensure there is not already a phi for the variable (n.b. probably not usefull)
        for i in &self[target_block].instructions {
            match self.try_get_instruction(*i) {
                Some(Instruction {
                    operator: Operation::Phi { root, .. },
                    ..
                }) if *root == phi_root => {
                    return *i;
                }
                _ => (),
            }
        }

        let v_type = self.get_object_type(phi_root);
        let operation = Operation::Phi {
            root: phi_root,
            block_args: vec![],
        };
        let new_phi = Instruction::new(operation, v_type, Some(target_block));
        let phi_id = self.add_instruction(new_phi);
        self[target_block].instructions.insert(1, phi_id);
        phi_id
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
