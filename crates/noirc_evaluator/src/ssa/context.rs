use super::block::{BasicBlock, BlockId};
use super::mem::Memory;
use super::node::{Instruction, NodeId, NodeObj, ObjectType, Operation};
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
use num_bigint::BigUint;
use num_traits::Zero;

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

    pub fn zero(&self) -> NodeId {
        self.find_const_with_type(&BigUint::zero(), node::ObjectType::Unsigned(1)).unwrap()
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

    pub fn print_block(&self, b: &block::BasicBlock) {
        for id in &b.instructions {
            let ins = self.get_instruction(*id);
            let mut str_res = if ins.res_name.is_empty() {
                format!("{:?}", id.0.into_raw_parts().0)
            } else {
                ins.res_name.clone()
            };
            if ins.is_deleted {
                str_res += " -DELETED";
            }
            let lhs_str = self.node_to_string(ins.lhs);
            let rhs_str = self.node_to_string(ins.rhs);
            let mut ins_str = format!("{} op:{:?} {}", lhs_str, ins.operator, rhs_str);

            if ins.operator == node::Operation::Phi {
                ins_str += "(";
                for (v, b) in &ins.phi_arguments {
                    ins_str +=
                        &format!("{:?}:{:?}, ", v.0.into_raw_parts().0, b.0.into_raw_parts().0);
                }
                ins_str += ")";
            }
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

    pub fn new_instruction(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        opcode: node::Operation,
        optype: node::ObjectType,
    ) -> NodeId {
        let operands = vec![lhs, rhs];
        self.new_instruction_with_multiple_operands(operands, opcode, optype)
    }

    pub fn new_instruction_with_multiple_operands(
        &mut self,
        mut operands: Vec<NodeId>,
        opcode: node::Operation,
        optype: node::ObjectType,
    ) -> NodeId {
        while operands.len() < 2 {
            operands.push(NodeId::dummy());
        }
        //Add a new instruction to the nodes arena
        let mut i = node::Instruction::new(
            opcode,
            operands[0],
            operands[1],
            optype,
            Some(self.current_block),
        );
        //Basic simplification
        optim::simplify(self, &mut i);
        if operands.len() > 2 {
            i.ins_arguments = operands;
        }
        if i.is_deleted {
            return i.rhs;
        }
        self.push_instruction(i)
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
    pub fn get_result_type(&self, op: Operation, lhs_type: node::ObjectType) -> node::ObjectType {
        match op {
            Operation::Eq
            | Operation::Ne
            | Operation::Ugt
            | Operation::Uge
            | Operation::Ult
            | Operation::Ule
            | Operation::Sgt
            | Operation::Sge
            | Operation::Slt
            | Operation::Sle
            | Operation::Lt
            | Operation::Gt
            | Operation::Lte
            | Operation::Gte => ObjectType::Boolean,
            Operation::Jne
            | Operation::Jeq
            | Operation::Jmp
            | Operation::Nop
            | Operation::Constrain(_)
            | Operation::Store(_) => ObjectType::NotAnObject,
            Operation::Load(adr) => self.mem.arrays[adr as usize].element_type,
            Operation::Cast | Operation::Trunc => unreachable!("cannot determine result type"),
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

    pub fn generate_empty_phi(&mut self, target_block: BlockId, root: NodeId) -> NodeId {
        //Ensure there is not already a phi for the variable (n.b. probably not usefull)
        for i in &self[target_block].instructions {
            if let Some(ins) = self.try_get_instruction(*i) {
                if ins.operator == node::Operation::Phi && ins.rhs == root {
                    return *i;
                }
            }
        }

        let v_type = self.get_object_type(root);
        let new_phi = Instruction::new(Operation::Phi, root, root, v_type, Some(target_block));
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
