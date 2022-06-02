use super::block::{BasicBlock, BlockId};
use super::function::SSAFunction;
use super::inline::StackFrame;
use super::mem::Memory;
use super::node::{Instruction, NodeId, NodeObj, ObjectType, Operation};
use super::{block, flatten, inline, integer, node, optim};
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
use num_traits::{One, Zero};

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
    pub mem: Memory,
    pub functions: HashMap<FuncId, function::SSAFunction>,
    pub call_graph: Vec<Vec<u8>>,
    dummy_store: HashMap<u32, NodeId>,
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
            mem: Memory::default(),
            functions: HashMap::new(),
            call_graph: Vec::new(),
            dummy_store: HashMap::new(),
        };
        block::create_first_block(&mut pc);
        pc.one_type(node::ObjectType::Unsigned(1));
        pc.zero_type(node::ObjectType::Unsigned(1));
        pc
    }

    pub fn zero(&self) -> NodeId {
        self.find_const_with_type(&BigUint::zero(), node::ObjectType::Unsigned(1)).unwrap()
    }

    pub fn one(&self) -> NodeId {
        self.find_const_with_type(&BigUint::one(), node::ObjectType::Unsigned(1)).unwrap()
    }

    pub fn zero_type(&mut self, obj_type: ObjectType) -> NodeId {
        self.get_or_create_const(FieldElement::zero(), obj_type)
    }

    pub fn one_type(&mut self, obj_type: ObjectType) -> NodeId {
        self.get_or_create_const(FieldElement::one(), obj_type)
    }

    pub fn get_dummy_store(&self, a: u32) -> NodeId {
        *self.dummy_store.get(&a).unwrap()
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
            if let Some(Instruction { operator: op, .. }) = self.try_get_instruction(*i) {
                if *op != Operation::Nop && *op != Operation::Phi {
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
            return Some(FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()));
        }
        None
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

    //Returns true is a may be equal to b, and false else
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
            i.ins_arguments =
                function::CallStack { arguments: operands, return_values: Vec::new() };
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

    pub fn new_array(
        &mut self,
        name: &str,
        element_type: ObjectType,
        len: u32,
        def_id: Option<noirc_frontend::node_interner::DefinitionId>,
    ) -> NodeId {
        let array_index = self.mem.create_new_array(len, element_type, name);
        //we create a variable pointing to this MemArray
        let new_var = node::Variable {
            id: NodeId::dummy(),
            obj_type: node::ObjectType::Pointer(array_index),
            name: name.to_string(),
            root: None,
            def: def_id,
            witness: None,
            parent_block: self.current_block,
        };
        if let Some(def) = def_id {
            self.mem.arrays[array_index as usize].def = def;
        }
        self.add_variable(new_var, None)
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
        self.pause(interactive, "SSA:", "inline functions");
        function::inline_all(self);
        //Optimisation
        block::compute_dom(self);
        optim::full_cse(self, self.first_block);
        self.pause(interactive, "CSE:", "unrolling:");
        //Unrolling
        flatten::unroll_tree(self, self.first_block);
        //Inlining
        self.pause(interactive, "", "inlining:");
        inline::inline_tree(self, self.first_block);
        optim::full_cse(self, self.first_block);

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

    fn memcpy(&mut self, l_type: ObjectType, r_type: ObjectType) {
        if l_type == r_type {
            return;
        }

        if let (ObjectType::Pointer(a), ObjectType::Pointer(b)) = (l_type, r_type) {
            let len = self.mem.arrays[a as usize].len;
            let adr_a = self.mem.arrays[a as usize].adr;
            let adr_b = self.mem.arrays[b as usize].adr;
            let e_type = self.mem.arrays[b as usize].element_type;
            for i in 0..len {
                let idx_b = self.get_or_create_const(
                    FieldElement::from((adr_b + i) as i128),
                    ObjectType::Unsigned(32),
                );
                let idx_a = self.get_or_create_const(
                    FieldElement::from((adr_a + i) as i128),
                    ObjectType::Unsigned(32),
                );
                let load = self.new_instruction(idx_b, idx_b, Operation::Load(b), e_type);
                self.new_instruction(load, idx_a, Operation::Store(a), l_type);
            }
        } else {
            unreachable!("invalid type, expected arrays, got {:?} and {:?}", l_type, r_type);
        }
    }

    pub fn handle_assign(&mut self, lhs: NodeId, index: Option<NodeId>, rhs: NodeId) -> NodeId {
        let lhs_type = self.get_object_type(lhs);
        let rhs_type = self.get_object_type(rhs);
        let i = 0;
        let i_obj =
            self.get_or_create_const(FieldElement::from(i as i128), ObjectType::Unsigned(32));
        if let Some(Instruction {
            operator: Operation::Call(_), ins_arguments: call_stack, ..
        }) = self.try_get_mut_instruction(rhs)
        {
            if index.is_some() || !matches!(lhs_type, ObjectType::Pointer(_)) {
                let obj_type = if let ObjectType::Pointer(a) = lhs_type {
                    self.mem.arrays[a as usize].element_type
                } else {
                    lhs_type
                };
                let ret = self.new_instruction(rhs, i_obj, Operation::Res, obj_type);
                return self.handle_assign(lhs, index, ret);
            } else {
                call_stack.return_values.push(lhs);
                if let ObjectType::Pointer(a) = lhs_type {
                     //dummy store for a
                                                    let dummy_store = node::Instruction::new(
                                                        node::Operation::Store(a),
                                                        NodeId::dummy(),
                                                        NodeId::dummy(),
                                                        node::ObjectType::NotAnObject,
                                                        None,
                                                    );
                    let id = self.add_instruction(dummy_store);
                    self.dummy_store.insert(a, id);
                }
                return lhs;
            }
        }
        if let Some(idx) = index {
            if let ObjectType::Pointer(a) = lhs_type {
                //Store
                return self.new_instruction(
                    rhs,
                    idx,
                    Operation::Store(a),
                    self.mem.arrays[a as usize].element_type,
                );
            } else {
                unreachable!("Index expression must be for an array");
            }
        } else if matches!(lhs_type, ObjectType::Pointer(_)) {
            if let Some(Instruction {
                operator: Operation::Intrinsic(_), res_type: rtype, ..
            }) = self.try_get_mut_instruction(rhs)
            {
                *rtype = lhs_type;
            } else {
                self.memcpy(lhs_type, rhs_type);
                return lhs;
            }
        }
        let lhs_obj = self.get_variable(lhs).unwrap();
        let new_var = node::Variable {
            id: lhs,
            obj_type: lhs_type,
            name: String::new(),
            root: None,
            def: lhs_obj.def,
            witness: None,
            parent_block: self.current_block,
        };
        let ls_root = lhs_obj.get_root();
        //ssa: we create a new variable a1 linked to a
        let new_var_id = self.add_variable(new_var, Some(ls_root));
        let result = self.new_instruction(new_var_id, rhs, node::Operation::Ass, rhs_type);
        self.update_variable_id(ls_root, new_var_id, result); //update the name and the value map
        new_var_id
    }

    fn new_instruction_inline(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        opcode: node::Operation,
        optype: node::ObjectType,
        stack_frame: &mut StackFrame,
    ) -> NodeId {
        let i = node::Instruction::new(opcode, lhs, rhs, optype, Some(stack_frame.block));
        let ins_id = self.add_instruction(i);
        stack_frame.push(ins_id);
        ins_id
    }

    fn memcpy_inline(
        &mut self,
        l_type: ObjectType,
        r_type: ObjectType,
        stack_frame: &mut StackFrame,
    ) {
        if l_type == r_type {
            return;
        }

        if let (ObjectType::Pointer(a), ObjectType::Pointer(b)) = (l_type, r_type) {
            let len = self.mem.arrays[a as usize].len;
            let adr_a = self.mem.arrays[a as usize].adr;
            let adr_b = self.mem.arrays[b as usize].adr;
            let e_type = self.mem.arrays[b as usize].element_type;
            for i in 0..len {
                let idx_b = self.get_or_create_const(
                    FieldElement::from((adr_b + i) as i128),
                    ObjectType::Unsigned(32),
                );
                let idx_a = self.get_or_create_const(
                    FieldElement::from((adr_a + i) as i128),
                    ObjectType::Unsigned(32),
                );
                let load = self.new_instruction_inline(
                    idx_b,
                    idx_b,
                    Operation::Load(b),
                    e_type,
                    stack_frame,
                );
                self.new_instruction_inline(load, idx_a, Operation::Store(a), l_type, stack_frame);
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
                def: lhs_obj.def,
                witness: None,
                parent_block: self.current_block,
            };
            let ls_root = lhs_obj.get_root();
            //ssa: we create a new variable a1 linked to a
            let new_var_id = self.add_variable(new_var, Some(ls_root));
            //ass
            let result = self.new_instruction_inline(
                new_var_id,
                rhs,
                node::Operation::Ass,
                rhs_type,
                stack_frame,
            );
            self.update_variable_id_in_block(ls_root, new_var_id, result, block_id); //update the name and the value map
            result
        }
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
