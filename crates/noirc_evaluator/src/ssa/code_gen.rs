use super::node;
use super::optim;
use std::collections::{HashMap, VecDeque};
use std::ops::Add;

use super::super::environment::{Environment, FuncContext};
use super::super::errors::{RuntimeError, RuntimeErrorKind};
use crate::object::{Array, Integer, Object, RangedObject};
use crate::ssa::acir_gen::Acir;
use crate::ssa::node::Node;
use crate::Evaluator;
use acvm::FieldElement;
use arena;
use noirc_frontend::hir::Context;
use noirc_frontend::hir_def::{
    expr::{
        HirBinaryOp, HirBinaryOpKind, HirBlockExpression, HirCallExpression, HirExpression,
        HirForExpression, HirLiteral,
    },
    stmt::{HirConstrainStatement, HirLetStatement, HirPrivateStatement, HirStatement},
};
use noirc_frontend::node_interner::{ExprId, FuncId, IdentId, StmtId};
//use noirc_frontend::{FunctionKind, Type};
use num_bigint::BigUint;

// This is a 'master' class for generating the SSA IR from the AST
// It contains all the data; the node objects representing the source code in the nodes arena
// and The CFG in the blocks arena
// everything else just reference objects from these two arena using their index.
//TODO to rename, or may be refactor?
pub struct ParsingContext<'a> {
    pub context: Option<&'a Context>,

    pub first_block: arena::Index,
    pub current_block: arena::Index,
    pub blocks: arena::Arena<node::BasicBlock>,
    pub nodes: arena::Arena<node::NodeObj>,
    pub id0: arena::Index, //dummy index.. should we put a dummy object somewhere?
    pub dummy_instruction: arena::Index,
    // pub objects: arena::Arena<node::Variable>,
}

impl<'a> ParsingContext<'a> {
    pub fn new(context: &Context) -> ParsingContext {
        let mut pc = ParsingContext {
            context: Some(context),
            id0: ParsingContext::dummy_id(),
            first_block: ParsingContext::dummy_id(),
            current_block: ParsingContext::dummy_id(),
            blocks: arena::Arena::new(),
            nodes: arena::Arena::new(),
            dummy_instruction: ParsingContext::dummy_id(),
        }; //, objects: arena::Arena::new()
        pc.create_first_block();
        pc
    }

    fn print_operand(&self, idx: arena::Index) -> String {
        let var = self.get_object(idx);
        if var.is_none() {
            return format!("unknown {:?}", idx);
        }
        match var.unwrap() {
            node::NodeObj::Obj(v) => v.print(),
            node::NodeObj::Instr(i) => i.print_i(),
            node::NodeObj::Const(c) => c.print(),
        }
    }

    pub fn print(&self) {
        let mut i = 0;
        for (_, b) in &self.blocks {
            println!("************* Block n.{}", i);
            i += 1;
            for idx in &b.instructions {
                let ins = self.get_instruction(*idx);
                let mut str_res;
                if ins.res_name.is_empty() {
                    str_res = format!("{:?}", idx);
                } else {
                    str_res = ins.res_name.clone();
                }
                if ins.is_deleted {
                    str_res += " -DELETED";
                }
                let lhs_str = self.print_operand(ins.lhs);
                let rhs_str = self.print_operand(ins.rhs);
                let ins_str = format!(
                    "{} {} {}",
                    lhs_str,
                    format!(" op:{:?} ", ins.operator),
                    rhs_str
                );
                println!("{}: {}", str_res, ins_str);
                //DEBUG...
                //     let li = self.get_as_instruction(ins.lhs);
                //     if (li.is_some()) {
                //         dbg!(li.unwrap());
                //     }
                //     let ri = self.get_as_instruction(ins.rhs);
                //     if (ri.is_some()) {
                //         dbg!(ri.unwrap());
                //     }
            }
        }
    }

    pub fn context(&self) -> &Context {
        self.context.unwrap()
    }

    pub fn add_object2(&mut self, obj: node::NodeObj) -> arena::Index {
        let idx = self.nodes.insert(obj);
        let obj2 = self.nodes.get_mut(idx).unwrap(); //TODO-RIA can we avoid this? and simply modify obj?
        match obj2 {
            node::NodeObj::Obj(o) => o.id = idx,
            node::NodeObj::Instr(i) => {
                i.idx = idx;
                let cb = self.get_current_block_mut();
                cb.instructions.push(idx);
            }
            node::NodeObj::Const(c) => c.id = idx,
        }

        idx
    }

    pub fn find_variable(&self, name: &String) -> Option<&node::Variable> {
        //TODO check if this is really used
        for (_, o) in &self.nodes {
            match o {
                node::NodeObj::Obj(v) => {
                    if v.name == *name {
                        return Some(v);
                    }
                }
                _ => (),
            }
        }
        None
    }

    pub fn find_const(&self, value: &BigUint) -> Option<arena::Index> {
        //TODO We should map constant values to id
        for (idx, o) in &self.nodes {
            match o {
                node::NodeObj::Const(c) => {
                    if c.value == *value {
                        return Some(idx);
                    }
                }
                _ => (),
            }
        }
        None
    }

    pub fn dummy_id() -> arena::Index {
        arena::Index::from_raw_parts(std::usize::MAX, 0)
    }

    pub fn dummy(&self) -> arena::Index {
        ParsingContext::dummy_id()
    }

    pub fn get_object(&self, idx: arena::Index) -> Option<&node::NodeObj> {
        self.nodes.get(idx)
    }

    pub fn get_mut_object(&mut self, idx: arena::Index) -> Option<&mut node::NodeObj> {
        self.nodes.get_mut(idx)
    }

    fn get_object_type(&self, idx: arena::Index) -> node::ObjectType {
        self.get_object(idx).unwrap().get_type()
    }

    pub fn get_as_constant(&self, idx: arena::Index) -> Option<FieldElement> {
        let obj = self.get_object(idx).unwrap();
        match obj {
            node::NodeObj::Const(c) => {
                Some(FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()))
            }
            _ => None,
        }
    }

    fn get_mut_instruction(&mut self, idx: arena::Index) -> &mut node::Instruction {
        let obj = self.get_mut_object(idx).unwrap();
        match obj {
            node::NodeObj::Instr(i) => i,
            _ => todo!(), //Panic
        }
    }

    pub fn get_as_instruction(&self, idx: arena::Index) -> Option<&node::Instruction> {
        let obj = self.get_object(idx);
        if obj.is_some() {
            match obj.unwrap() {
                node::NodeObj::Instr(i) => return Some(i),
                _ => return None,
            }
        }
        None
    }

    pub fn get_as_mut_instruction(&mut self, idx: arena::Index) -> Option<&mut node::Instruction> {
        let obj = self.get_mut_object(idx);
        if obj.is_some() {
            let result = match obj.unwrap() {
                node::NodeObj::Instr(i) => Some(i),
                _ => None,
            };
            return result;
        }
        None
    }

    fn get_instruction(&self, idx: arena::Index) -> &node::Instruction {
        let obj = self.get_object(idx);
        if obj.is_some() {
            let result = match obj.unwrap() {
                node::NodeObj::Instr(i) => i,
                _ => unreachable!("Not and instruction"),
            };
            return result;
        }
        unreachable!("Index not found")
    }

    pub fn get_variable(&self, idx: arena::Index) -> Result<&node::Variable, &str> {
        //TODO RuntimeError!!
        match self.nodes.get(idx) {
            Some(t) => match t {
                node::NodeObj::Obj(o) => Ok(o),
                _ => Err("Not an object"),
            },
            _ => Err("Invalid id"),
        }
    }

    pub fn get_mut_variable(&mut self, idx: arena::Index) -> Result<&mut node::Variable, &str> {
        //TODO RuntimeError!!
        match self.nodes.get_mut(idx) {
            Some(t) => match t {
                node::NodeObj::Obj(o) => Ok(o),
                _ => Err("Not an object"),
            },
            _ => Err("Invalid id"),
        }
    }

    pub fn try_into_instruction(&self, idx: arena::Index) -> Option<&node::Instruction> {
        let obj = self.get_object(idx);
        if obj.is_some() {
            return match obj.unwrap() {
                node::NodeObj::Instr(i) => Some(i),
                _ => None,
            };
        }
        None
    }

    pub fn try_into_mut_instruction(
        &mut self,
        idx: arena::Index,
    ) -> Option<&mut node::Instruction> {
        let obj = self.get_mut_object(idx);
        if obj.is_some() {
            return match obj.unwrap() {
                node::NodeObj::Instr(i) => Some(i),
                _ => None,
            };
        }
        None
    }

    pub fn get_root(&self, var: &node::Variable) -> arena::Index {
        match var.root {
            Some(r) => r,
            _ => var.id,
        }
    }

    pub fn get_root_id(&self, var_id: arena::Index) -> arena::Index {
        let var = self.get_variable(var_id).unwrap();
        match var.root {
            Some(r) => r,
            _ => var_id,
        }
    }

    pub fn add_variable(
        &mut self,
        obj: node::Variable,
        root: Option<arena::Index>,
    ) -> arena::Index {
        let idx = self.nodes.insert(node::NodeObj::Obj(obj));
        let obj2 = self.nodes.get_mut(idx).unwrap();
        match obj2 {
            //TODO RIA
            node::NodeObj::Obj(v) => {
                v.id = idx;
                v.root = root;
                v.cur_value = idx; //TODO we should update root's current value - but we use the value_array instead
            }
            _ => todo!(), //panic
        }

        idx
    }

    //We do not use a double linked list anymore as it is too tricky because of Rust ownership
    fn add_instruction2_to_current_block(&mut self, idx: arena::Index) -> arena::Index {
        let cb = self.get_current_block_mut();
        //Add the instruction to the block
        cb.instructions.push(idx);
        idx
    }

    pub fn new_instruction(
        &mut self,
        lhs: arena::Index,
        rhs: arena::Index,
        opcode: node::Operation,
        optype: node::ObjectType,
    ) -> arena::Index {
        //Add a new instruction to the nodes arena
        let cb = self.get_current_block();

        let mut i = node::Instruction::new(opcode, lhs, rhs, optype, Some(cb.idx));
        //Basic simplification
        optim::simplify(self, &mut i);
        if i.is_deleted {
            return i.rhs;   //TODO how should we handle fully deleted instruction (i.e i.rhs is not an object)?
        }
        self.add_object2(node::NodeObj::Instr(i))
    }

    pub fn get_const(&mut self, x: FieldElement, t: node::ObjectType) -> arena::Index {
        let value = BigUint::from_bytes_be(&x.to_bytes()); //TODO a const should be a field element
        let obj = self.find_const(&value); //todo type
        if obj.is_some() {
            return obj.unwrap();
        }
        let obj_cst = node::Constant {
            id: self.dummy(),
            value: value,
            value_str: String::new(),
            value_type: t,
        };
        let obj = node::NodeObj::Const(obj_cst);
        self.add_object2(obj)
    }

    //TODO the type should be provided by previous step
    pub fn new_constant(&mut self, x: FieldElement) -> arena::Index {
        //we try to convert it to a supported integer type
        //if it does not work, we use the field type
        //n.b we cannot support custom fields bigger than the native field, we would need to support bigint and
        //use bigint inside HiLiterrals.
        //default to i32 (like rust)

        //We first check if a constant with the same value already exists, and use it if it exists. it will allow for better CSE.
        let value = BigUint::from_bytes_be(&x.to_bytes()); //TODO a const should be a field element
        if let Some(prev_const) = self.find_const(&value) {
            return prev_const;
        }

        //TODO default should be FieldElement, not i32
        let num_bits = x.num_bits();
        let idx: arena::Index;
        if num_bits < 32 {
            let obj_cst = node::Constant {
                id: self.id0,
                value: value,
                value_type: node::ObjectType::signed(32),
                value_str: String::new(),
            };
            let obj = node::NodeObj::Const(obj_cst);
            idx = self.add_object2(obj);
            return idx;
        } else {
            idx = self.id0.clone();
            todo!();
            //we should support integer of size < FieldElement::max_num_bits()/2, because else we cannot support multiplication!
            //for bigger size, we will need to represent an integer using several field elements, it may be easier to implement them in Noir! (i.e as a Noir library)
        }
        idx
    }

    //Returns the current SSA value of a variable.
    pub fn get_current_value(&self, var: &node::Variable) -> arena::Index {
        if let Some(root) = var.root {
            match self.get_object(root).unwrap() {
                node::NodeObj::Obj(o) => {
                    return self.get_current_value(&o);
                }
                _ => todo!(), //Err
            }
        }
        //Current value is taken from the value_array, we do not use anymore the cur_value of the variable because it was painfull to update due to rust ownership
        let mut block = self.get_current_block();
        if let Some(idx) = block.get_current_value(var.id) {
            return idx;
        }
        // if the block value array does not have the variable, we ask recursively to its predecessor
        while let Some(prev) = block.predecessor.first() {
            let prev_opt = self.get_block(*prev);
            if prev_opt.is_some()
            {
                block = prev_opt.unwrap();
                if let Some(idx) = block.get_current_value(var.id) {
                    return idx;
                }
            }
            else {
                return var.id; 
            }
   
        }
        return var.id;
    }

    //update 'new_var' which is a new ssa value of previous 'var' by:
    //- setting a new name  (e.g if var is x, then new_var becomes x5)
    //- update value array and value name structures of the block
    // pub fn update_variable(&mut self, var: &node::Variable, new_var: arena::Index, new_value: arena::Index)
    // {
    //     let root = self.get_variable(self.get_root(var)).unwrap();
    //     let root_name = root.name.clone();
    //     let cb = self.get_current_block_mut();
    //     cb.update_variable(var.id, new_value);
    //     let vname = cb.get_value_name(var.id).to_string();
    //     let nvar = self.get_mut_variable(new_var).unwrap();

    //     nvar.name = root_name + &vname;
    // }

    //same as update_variable but using the var index instead of var
    pub fn update_variable_id(
        &mut self,
        var_id: arena::Index,
        new_var: arena::Index,
        new_value: arena::Index,
    ) {
        let root_id = self.get_root_id(var_id);
        let root = self.get_variable(root_id).unwrap();
        let root_name = root.name.clone();
        let cb = self.get_current_block_mut();
        cb.update_variable(var_id, new_value);
        let vname = cb.get_value_name(var_id).to_string();
        let nvar = self.get_mut_variable(new_var);
        if nvar.is_ok() {
            nvar.unwrap().name = root_name + &vname;
        }
    }

    ///////////
    pub fn get_node(&self, idx: arena::Index) -> Option<&node::NodeObj> {
        self.nodes.get(idx)
    }

    pub fn get_block(&self, idx: arena::Index) -> Option<&node::BasicBlock> {
        self.blocks.get(idx)
    }

    pub fn get_block_mut(&mut self, idx: arena::Index) -> Option<&mut node::BasicBlock> {
        self.blocks.get_mut(idx)
    }

    pub fn get_current_block_mut(&mut self) -> &mut node::BasicBlock {
        self.blocks.get_mut(self.current_block).unwrap()
    }

    pub fn get_current_block(&self) -> &node::BasicBlock {
        self.blocks.get(self.current_block).unwrap()
    }

    fn create_first_block(&mut self) {
        let mut first_block = node::BasicBlock::new(self.dummy(), node::BlockType::Normal);
        first_block.left = None;
        let new_idx = self.blocks.insert(first_block);
        let block2 = self.blocks.get_mut(new_idx).unwrap(); //RIA..
        block2.idx = new_idx;
        self.first_block = new_idx;
        self.current_block = new_idx;
        self.dummy_instruction = self.new_instruction(
            self.dummy(),
            self.dummy(),
            node::Operation::nop,
            node::ObjectType::none,
        );
    }

    //Not suitable for the first block
    pub fn new_block(&mut self, kind: node::BlockType) -> arena::Index {
        let new_block = node::BasicBlock::new(self.current_block, kind);
        let new_idx = self.blocks.insert(new_block);
        let block2 = self.blocks.get_mut(new_idx).unwrap(); //RIA..
        block2.idx = new_idx;
        block2.dominator = Some(self.current_block);
        //update current block
        let cb = self.get_block_mut(self.current_block).unwrap();
        cb.left = Some(new_idx);
        self.current_block = new_idx;
        self.dummy_instruction = self.new_instruction(
            self.dummy(),
            self.dummy(),
            node::Operation::nop,
            node::ObjectType::none,
        );
        new_idx
    }

    //create a block and sets its id, but do not update current block, and do not add dummy instruction!
    pub fn create_block(&mut self, kind: node::BlockType) -> arena::Index {
        let new_block = node::BasicBlock::new(self.current_block, kind);
        let new_idx = self.blocks.insert(new_block);
        let block2 = self.blocks.get_mut(new_idx).unwrap(); //RIA..
        block2.idx = new_idx;
        new_idx
    }

    //link the current block to the target block so that current block becomes its target
    pub fn fixup(
        &mut self,
        target: arena::Index,
        left: Option<arena::Index>,
        right: Option<arena::Index>,
    ) {
        if let Some(target_block) = self.get_block_mut(target) {
            target_block.right = right;
            target_block.left = left;
            //TODO should also update the last instruction rhs to the first instruction of the current block  -- TODOshoud we do it here??
            if right.is_some() {
                let rb = self.get_block_mut(right.unwrap());
                rb.unwrap().dominator = Some(target);
            }
            if left.is_some() {
                let lb = self.get_block_mut(left.unwrap());
                lb.unwrap().dominator = Some(target);
            }
        }
    }

    pub fn compute_dominated(&mut self) {
        let mut b = self.get_block(self.first_block);
        let mut rira: HashMap<arena::Index, Vec<arena::Index>> = HashMap::new();
        while b.unwrap().left.is_some() {
            let r = b.unwrap().left.unwrap();
            b = self.blocks.get(r);
            if let Some(b) = self.blocks.get(r) {
                if let Some(dom_b_idx) = b.dominator {
                    if self.get_block(dom_b_idx).is_some() {
                        if rira.contains_key(&dom_b_idx) {
                            let mut v = rira[&dom_b_idx].clone(); //TODO can we avoid it?
                            v.push(r);
                            rira.insert(dom_b_idx, v);
                        } else {
                            rira.insert(dom_b_idx, [r].to_vec());
                        }
                    }
                }
            }
        }
        //RIA
        for (master, svec) in rira {
            let dom_b = self.get_block_mut(master).unwrap();
            for slave in svec {
                dom_b.dominated.push(slave);
            }
        }
    }

    ////////////////////OPTIMISATIONS/////////////////////////////////////////////////////////////////////////

    pub fn find_similar_instruction(
        &self,
        lhs: arena::Index,
        rhs: arena::Index,
        prev_ins: &VecDeque<arena::Index>,
    ) -> Option<arena::Index> {
        for iter in prev_ins {
            let i = self.get_as_instruction(*iter);
            if i.is_some() {
                let ins = i.unwrap();
                if ins.lhs == lhs && ins.rhs == rhs {
                    return Some(*iter);
                }
            }
        }
        return None;
    }

    pub fn propagate(&self, idx: arena::Index) -> arena::Index {
        let obj = self.get_as_instruction(idx);
        let mut result = idx;
        if obj.is_some() {
            if (obj.unwrap().operator == node::Operation::ass || obj.unwrap().is_deleted) {
                result = obj.unwrap().rhs;
            }
        }

        result
    }

    //common subexpression elimination
    pub fn cse(&mut self) {
        let mut anchor: HashMap<node::Operation, VecDeque<arena::Index>> = HashMap::new();
        // let mut i_list : Vec<arena::Index> = Vec::new();
        self.cse_tree(self.first_block, &mut anchor);

        //TODO do it for all blocks by following the dominator tree and passing around the anchor list.
        //self.block_cse(self.first_block, &mut anchor, &mut i_list);
    }

    pub fn cse_tree(
        &mut self,
        b_idx: arena::Index,
        anchor: &mut HashMap<node::Operation, VecDeque<arena::Index>>,
    ) {
        let mut i_list: Vec<arena::Index> = Vec::new();
        self.block_cse(b_idx, anchor, &mut i_list);
        let block = self.get_block(b_idx).unwrap();
        let bd = block.dominated.clone();
        for b in bd {
            self.cse_tree(b, &mut anchor.clone());
        }
    }
    //Performs common subexpression elimination and copy propagation on a block
    pub fn block_cse(
        &mut self,
        b_idx: arena::Index,
        anchor: &mut HashMap<node::Operation, VecDeque<arena::Index>>,
        block_list: &mut Vec<arena::Index>,
    ) {
        let mut new_list: Vec<arena::Index> = Vec::new();
        let bb = self.blocks.get(b_idx).unwrap();

        if block_list.is_empty() {
            //RIA...
            for iter in &bb.instructions {
                block_list.push(iter.clone());
            }
        }

        for iter in block_list {
            if let Some(ins) = self.get_as_instruction(*iter)
            {
                let mut to_delete = false;
                let mut i_lhs = ins.lhs.clone();
                let mut i_rhs = ins.rhs.clone();
                let mut i_lhs_name = String::new();
                let mut i_rhs_name = String::new();
                let mut phi_args: Vec<(arena::Index, arena::Index)> = Vec::new();
                let mut to_update_phi = false;
                if !anchor.contains_key(&ins.operator) {
                    anchor.insert(ins.operator, VecDeque::new());
                }
                if node::is_binary(ins.operator) {
                    //binary operation:
                    i_lhs = self.propagate(ins.lhs);
                    i_rhs = self.propagate(ins.rhs);
                    let j = self.find_similar_instruction(i_lhs, i_rhs, &anchor[&ins.operator]);
                    if j.is_some() {
                        to_delete = true; //we want to delete ins but ins is immutable so we use the new_list instead
                        i_rhs = j.unwrap();
                    } else {
                        new_list.push(*iter);
                        anchor.get_mut(&ins.operator).unwrap().push_front(*iter);
                        //TODO - Take into account store and load for arrays
                    }
                } else if ins.operator == node::Operation::ass {
                    //assignement
                    i_rhs = self.propagate(ins.rhs);
                    to_delete = true;
                } else if ins.operator == node::Operation::phi {
                    // propagate phi arguments
                    for a in &ins.phi_arguments {
                        phi_args.push((self.propagate(a.0), a.1));
                        if phi_args.last().unwrap().0 != a.0 {
                            to_update_phi = true;
                        }
                    }
                    if let Some(first) = node::Instruction::simplify_one_phi(ins.idx, ins.lhs, &phi_args) {
                        if first == ins.idx {
                            new_list.push(*iter);
                        } else {
                            to_delete = true;
                            i_rhs = first;
                            to_update_phi = false;
                        }
                    } else {
                        todo!(); //is this hapenning?
                    }
                    //   dbg!(&ins.unwrap().phi_arguments);
                    //   dbg!(&phi_args);
                } else {
                    new_list.push(*iter);
                }
                if to_update_phi {
                    let update = self.get_mut_instruction(*iter);
                    update.phi_arguments = phi_args;
                } else if to_delete || ins.lhs != i_lhs || ins.rhs != i_rhs {
                    //update i:
                    let ii_l = ins.lhs.clone();
                    let ii_r = ins.rhs.clone();
                    let update = self.get_mut_instruction(*iter);
                    update.lhs = i_lhs;
                    update.rhs = i_rhs;
                    update.is_deleted = to_delete;
                    //update instruction name - for debug/pretty print purposes only /////////////////////
                    if let Some(node::Instruction {
                        operator: node::Operation::ass,
                        lhs,
                        ..
                    }) = self.try_into_instruction(ii_l)
                    {
                        if let Ok(lv) = self.get_variable(*lhs) {
                            let i_name = lv.name.clone();
                            if let Some(p_ins) = self.try_into_mut_instruction(i_lhs) {
                                if p_ins.res_name.is_empty() {
                                    p_ins.res_name = i_name;
                                }
                            }
                        }
                    }
                    if let Some(node::Instruction {
                        operator: node::Operation::ass,
                        lhs,
                        ..
                    }) = self.try_into_instruction(ii_r)
                    {
                        if let Ok(lv) = self.get_variable(*lhs) {
                            let i_name = lv.name.clone();
                            if let Some(p_ins) = self.try_into_mut_instruction(i_rhs) {
                                if p_ins.res_name.is_empty() {
                                    p_ins.res_name = i_name;
                                }
                            }
                        }
                    }
                    ////////////////////////////////////////update instruction name for debug purposes////////////////////////////////
                }
            }
        }
        let bb = self.blocks.get_mut(b_idx).unwrap();
        bb.instructions = new_list;
    }

    ////////////////PARSING THE AST//////////////////////////////////////////////
    /// Compiles the AST into the intermediate format by evaluating the main function
    pub fn evaluate_main(
        &mut self,
        env: &mut Environment,
        context: &'a Context,
    ) -> Result<(), RuntimeError> {
        self.context = Some(context);
        //todo... for now it is done in lib.rs evaluate_main_alt

        // let block = main_func_body.block(&self.context.def_interner);
        // for stmt_id in block.statements() {
        //     self.evaluate_statement(env, stmt_id)?;
        // }
        Ok(())
    }

    fn evaluate_expression(
        &mut self,
        env: &mut Environment,
        expr_id: &ExprId,
    ) -> Result<arena::Index, RuntimeError> {
        let expr = self.context.unwrap().def_interner.expression(expr_id);
        let span = self.context.unwrap().def_interner.expr_span(expr_id);
        match expr {
        HirExpression::Infix(infx) => {
            dbg!(infx.clone());
            let lhs = self.evaluate_expression(env, &infx.lhs)?;
            let rhs = self.evaluate_expression(env, &infx.rhs)?;
            self.evaluate_infix_expression(lhs, rhs, infx.operator)
        },
        HirExpression::Literal(HirLiteral::Integer(x)) => Ok(self.new_constant(x)),
        // TODO HirExpression::Literal(HirLiteral::Array(arr_lit)) =>...
        HirExpression::Ident(x) => Ok(self.evaluate_identifier(env,&x, None)),  //probably needs the create-phi here..TODO
        HirExpression::Cast(cast_expr) => {
            let lhs = self.evaluate_expression(env, &cast_expr.lhs)?;
            let r_type = node::ObjectType::from_type(cast_expr.r#type);
            Ok(self.new_cast_expression(lhs, r_type))
        },
        //HirExpression::Index(indexed_expr) => todo!(),
        //a[i] could be a load or a store instruction...TODO
       // HirExpression::Call(call_expr) =>  TODO .. control flow..
        HirExpression::For(for_expr) => self.handle_for_expr(env,for_expr).map_err(|kind|kind.add_span(span)),
       HirExpression::If(_) => todo!(),
       HirExpression::Prefix(_) => todo!(),
       HirExpression::Predicate(_) => todo!(),
       HirExpression::Literal(_) => todo!(),
       HirExpression::Block(_) => todo!("currently block expressions not in for/if branches are not being evaluated. In the future, we should be able to unify the eval_block and all places which require block_expr here"),

        _ => todo!(),
    }
    }

    fn evaluate_identifier(
        &mut self,
        env: &mut Environment,
        ident_id: &IdentId,
        create_phi: Option<arena::Index>,
    ) -> arena::Index {
        let ident_name = self.context.unwrap().def_interner.ident_name(ident_id);
        let var = self.find_variable(&ident_name); //TODO by name or by id?
        if var.is_some() {
            let mut v_id = self.get_current_value(&var.unwrap()); //ssa
            if var.unwrap().root.is_none() {
                dbg!(var);
            }
            if create_phi.is_some() {
                let phi = self.generate_one_phi(
                    create_phi.unwrap(),
                    v_id,
                    var.unwrap().id,//self.get_root(var.unwrap()),
                    v_id,
                    self.current_block,
                );
                if phi.is_some() {
                    v_id = phi.unwrap();
                }
            }
            return v_id; //self.get_current_value(&var.unwrap());
        }
        let obj = env.get(&ident_name);
        let obj_type = node::ObjectType::get_type_from_object(&obj);

        //new variable - should be in a let statement? The let statement should set the type
        let obj = node::Variable {
            id: self.id0,
            name: ident_name.clone(),
            /*symbol_id: ident_iid,*/ obj_type: obj_type,
            cur_value: self.id0,
            root: None,
            witness: node::get_witness_from_object(&obj),
        };
        self.add_variable(obj, None)
    }

    //cast devrait etre: (a)b => a cast b....TODO!!!
    //Cast lhs into type rtype
    fn new_cast_expression(&mut self, lhs: arena::Index, rtype: node::ObjectType) -> arena::Index {
        //generate instruction 'a cast a', with result type rtype
        let i = node::Instruction::new(
            node::Operation::cast,
            lhs,
            lhs,
            rtype,
            Some(self.current_block),
        );
        let idx = self.add_object2(node::NodeObj::Instr(i));
        idx
    }

    fn evaluate_infix_expression(
        &mut self,
        lhs: arena::Index,
        rhs: arena::Index,
        op: HirBinaryOp,
    ) -> Result<arena::Index, RuntimeError> {
        let ltype = self.get_object_type(lhs);
        let rtype = self.get_object_type(rhs);
        let optype = ltype; //TODO if type differs try to cast them: check how rust handle this
                            //and else returns an error
        // Get the opcode from the infix operator
        let opcode = node::to_operation(op.kind, optype);
        //TODO we should validate the types with the opcode
        Ok(self.new_instruction(lhs, rhs, opcode, optype)) //todo new_instruction should return a result to handle error
    }

    pub fn evaluate_statement(
        &mut self,
        env: &mut Environment,
        stmt_id: &StmtId,
        create_phi: Option<arena::Index>,
    ) -> Result<arena::Index, RuntimeError> {
        let statement = self.context().def_interner.statement(stmt_id);
        match statement {
            HirStatement::Private(x) => self.handle_private_statement(env, x),
            HirStatement::Constrain(constrain_stmt) => {
                self.handle_constrain_statement(env, constrain_stmt)
            }
            HirStatement::Const(x) => {
                let variable_name: String = self.context().def_interner.ident_name(&x.identifier);
                // const can only be integers/Field elements, cannot involve the witness, so we can possibly move this to
                // analysis. Right now it would not make a difference, since we are not compiling to an intermediate Noir format
                let span = self.context().def_interner.expr_span(&x.expression);
                self.expression_to_object(env, &x.expression, None) //devrait renvoyer un nodeobj de type const
                                                                    //TODO the result of expression_to_object should be an assignement, we should modify the lhs to specify it is a const
                                                                    // and then forbid any other assignement with the same variable during the SSA phase (and instead of applying the SSA form of it).
            }
            HirStatement::Expression(expr) | HirStatement::Semi(expr) => {
                self.expression_to_object(env, &expr, create_phi)
            }
            HirStatement::Let(let_stmt) => {
                // let statements are used to declare a higher level object
                self.handle_let_statement(env, let_stmt)
            }
            HirStatement::Assign(assign_stmt) => {
                //left hand is already declared
                //TODO clarify what is an HirStatement::Assign vs HirStatement::Let vs HirStatement::Priv
                //e.g 'let a=x+2;' could be let:(assign:(a, x+2)) or assign:(let a, x+2)
                //For now we do all the job here but it should be splitted

                //visiblement le = classique est traite dans le handle_private_statement... mais why??
                //qn comment on differencie let a =3 - vs- a=3; ? je suppose que ca vient avant dans le handle_let_statement..
                //on doit: generer une nouvelle variable
                //assign_stmt.identifier est un ident_id qui identifie ma variable (de gauche)
                //on peut avoir son nom  par:   let variable_name = self.context.def_interner.ident_name(&x.identifier);
                //nous on doit trouver le 'node' qui correspond a cette variable (grace a ident_id donc)
                //puis generer sa form SSA
                //comme c'est un assignement, il faudra (plus tard) gerer les Phi ici

                //pour l'instant on suppose que c'est pas un let
                //lhs = expression =>
                // soit var, soit const, soit instruction
                //instruction devient ins.res=lhs
                //const devient ? lhs est une var de type const => son cur_value est const!
                //var devient? => cur_value de lhs est var

                // It's possible to desugar the assign statement in the type checker.
                // However for clarity, we just match on the type and call the corresponding function.
                // eg if  we are assigning a witness, we call handle_private_statement
                let ident_def = self
                    .context()
                    .def_interner
                    .ident_def(&assign_stmt.identifier)
                    .unwrap();
                let ident_name = self
                    .context
                    .unwrap()
                    .def_interner
                    .ident_name(&assign_stmt.identifier);
                let var = self.find_variable(&ident_name);
                let lhs =
            //TODO - temp need to clarify if the creation of a variable should be done for HirStatement::Assign or for HirStatement::Let
            if var.is_none()
            {
                //var is not defined, 
                //let's do it here for now...TODO
                let obj = env.get(&ident_name);
                let obj_type = node::ObjectType::get_type_from_object(&obj);
                let new_var2 = node::Variable{
                    id : self.dummy(),
                     obj_type: obj_type,  //TODO
                     name: ident_name.clone(),
                     cur_value : self.dummy(),
                     root :  None,
                     witness: node::get_witness_from_object(&obj),
                 };
                 let new_var2_id= self.add_variable(new_var2, None);
                 self.get_variable(new_var2_id).unwrap()
            }
            else {
                var.unwrap()
            };

                let lhs_id = lhs.id;
                let new_var = node::Variable {
                    id: lhs.id,
                    obj_type: lhs.obj_type,
                    name: String::new(),
                    cur_value: lhs.cur_value,
                    root: None,
                    witness: None,
                };
                let ls_root = self.get_root(lhs);

                //ssa: we create a new variable a1 linked to a
                let new_var_id = self.add_variable(new_var, Some(ls_root));

                let rhs_id = self.expression_to_object(env, &assign_stmt.expression, create_phi)?;
                let mut rhs = self.get_object(rhs_id).unwrap(); //todo error..
                let rtype = rhs.get_type();
                let mut result;
                result = self.new_instruction(
                    new_var_id,
                    rhs.get_id(),
                    node::Operation::ass,
                    rhs.get_type(),
                );

                // match rhs {
                //     node::NodeObj::Instr(ins) => {
                //         result = self.new_instruction(  new_var_id, rhs_id, node::Operation::ass, ins.res_type);
                //     },
                //     node::NodeObj::Const(cst) => {
                //        result = self.new_instruction(  new_var_id, cst.id, node::Operation::ass, cst.value_type);
                //     }
                //     node::NodeObj::Obj(v) => {
                //         result = self.new_instruction(  new_var_id, v.id, node::Operation::ass, v.get_type());
                //     }
                //     _ => todo!(),
                // }
                //todo c'est pas ls_root MAIS plutot la 'premiere instance' de la var dans le block !!!!!!XXX
                if create_phi.is_some() {
                    let phi = self.generate_one_phi(
                        create_phi.unwrap(),
                        new_var_id,
                        new_var_id, //TODO to check ls_root,
                        rhs_id,
                        self.current_block,
                    );
                    if phi.is_some() {
                        result = phi.unwrap();
                    }
                    //  self.new_instruction(  new_var_id, create_phi.unwrap(), node::Operation::phi, rtype);
                }
                self.update_variable_id(ls_root, new_var_id, result); //update the name and the value array
                Ok(result)
            }
        }
    }

    fn create_new_variable(&mut self, var_name: String, env: &mut Environment) -> arena::Index {
        let obj = env.get(&var_name);
        let obj_type = node::ObjectType::get_type_from_object(&obj);
        let new_var = node::Variable {
            id: self.dummy(),
            obj_type: obj_type,
            name: var_name,
            cur_value: self.dummy(), //Not used
            root: None,
            witness: node::get_witness_from_object(&obj),
        };
        self.add_variable(new_var, None)
    }

    //create a variable for a new assignement of var
    fn new_ssa_variable(&mut self, var_id: arena::Index) -> arena::Index {
        if let Ok(var)  = self.get_variable(var_id) {
            let obj_type = var.get_type();
            let root  = self.get_root(var);
            let new_var = node::Variable {
                id: self.dummy(),
                obj_type: obj_type,
                name: String::new(),
                cur_value: self.dummy(), //Not used
                root: Some(root),
                witness: None,
            };
            let id = self.add_variable(new_var, Some(root));
            self.update_variable_id(root, id, id);
            return id;
        }
        unreachable!("TODO: error");
    }

       // Add a constraint to constrain two expression together
       fn handle_constrain_statement(
        &mut self,
        env: &mut Environment,
        constrain_stmt: HirConstrainStatement,
    ) -> Result<arena::Index, RuntimeError> {
        let lhs = self.expression_to_object( env, &constrain_stmt.0.lhs, None)?;
        let rhs = self.expression_to_object(env, &constrain_stmt.0.rhs, None)?;

        let result =
        match constrain_stmt.0.operator.kind  {
            // HirBinaryOpKind::Add => binary_op::handle_add_op(lhs, rhs, self),
            // HirBinaryOpKind::Subtract => binary_op::handle_sub_op(lhs, rhs, self),
            // HirBinaryOpKind::Multiply => binary_op::handle_mul_op(lhs, rhs, self),
            // HirBinaryOpKind::Divide => binary_op::handle_div_op(lhs, rhs, self),
            HirBinaryOpKind::NotEqual => todo!(),
            HirBinaryOpKind::Equal => Ok(self.new_instruction(lhs, rhs, node::Operation::eq_gate, node::ObjectType::none)),
            // HirBinaryOpKind::And => binary_op::handle_and_op(lhs, rhs, self),
            // HirBinaryOpKind::Xor => binary_op::handle_xor_op(lhs, rhs, self),
            HirBinaryOpKind::Less =>todo!(),
            HirBinaryOpKind::LessEqual => todo!(),
            HirBinaryOpKind::Greater => todo!(),
            HirBinaryOpKind::GreaterEqual => {
                todo!();
            }
            HirBinaryOpKind::Assign => {
                let err = RuntimeErrorKind::Spanless(
                    "The Binary operation `=` can only be used in declaration statements"
                        .to_string(),
                );
                Err(err)
            }
            HirBinaryOpKind::Or => {
                let err = RuntimeErrorKind::Unimplemented("The Or operation is currently not implemented. First implement in Barretenberg.".to_owned());
                Err(err)
            }
            _ => {
                let err = RuntimeErrorKind::Unimplemented("The operation is currently not supported in a constrain statement".to_owned());
                Err(err)
            }
        }.map_err(|kind|kind.add_span(constrain_stmt.0.operator.span));

        if constrain_stmt.0.operator.kind == HirBinaryOpKind::Equal {
           //TODO; the truncate strategy should benefit from this.
           //if one of them is a const, them we update the value array of the other to the same const
           // and we should even do it retro-actively!  mais comment?? => deuxieme pass?
           // we should replace one with the other retro-actively
           // we should merge their property; min(max), min(bitsize),etc..

        };
        result
    }

    //TODO: refactor properly so that one function handle the creation of a new variable and generates the ass opcode, and use it in priv,let,assign
    //then add the priv feature: a priv variable should never be assigned to a const value (n.b. because apparently this would indicate a bug in a user program)
    //so handle_private_statement should add the 'priv' attribute to the variable, and the handle_assign should check for it when assigning a const to a 'priv'var.
    fn handle_private_statement(
        &mut self,
        env: &mut Environment,
        priv_stmt: HirPrivateStatement,
    ) -> Result<arena::Index, RuntimeError> {
        // Create a new variable
        let variable_name = self
            .context()
            .def_interner
            .ident_name(&priv_stmt.identifier);
        let new_var = node::Variable {
            id: self.dummy(),
            obj_type: node::ObjectType::native_field, //TODO
            name: variable_name,
            cur_value: self.dummy(),
            root: None,
            witness: None, //TODO
        };
        let new_var_id = self.add_variable(new_var, None);
        // Create assign instruction
        let rhs_id = self.expression_to_object(env, &priv_stmt.expression, None)?;
        let rhs = self.get_object(rhs_id).unwrap(); //todo error..
        let result;
        match rhs {
            node::NodeObj::Instr(ins) => {
                // let mut_ins =self.get_mut_instruction(rhs_id);
                // mut_ins.res = Some(new_var_id); //TODO should delete previous temp res
                // result = mut_ins.idx;
                result =
                    self.new_instruction(new_var_id, rhs_id, node::Operation::ass, ins.res_type);
            }
            node::NodeObj::Const(cst) => {
                result =
                    self.new_instruction(new_var_id, cst.id, node::Operation::ass, cst.value_type);
            }
            node::NodeObj::Obj(v) => {
                result = self.new_instruction(new_var_id, v.id, node::Operation::ass, v.get_type());
            }
            _ => todo!(),
        }
        //Should we update the value array? TODO!
        //self.update_variable_id(lhs_id, new_var_id); //update the name and the value array
        Ok(result)
    }

    // Let statements are used to declare higher level objects
    //TODO cf. above, clarify roles of priv, let and assign...
    fn handle_let_statement(
        &mut self,
        env: &mut Environment,
        let_stmt: HirLetStatement,
    ) -> Result<arena::Index, RuntimeError> {
        //TODO this code is not relevant
        //we should create a variable from the left side of the statement, evaluate the right and generate an assign instruction.
        //TODO how to handle arrays?

        // Convert the LHS into an identifier
        let variable_name = self.context().def_interner.ident_name(&let_stmt.identifier);

        // XXX: Currently we only support arrays using this, when other types are introduced
        // we can extend into a separate (generic) module

        // Extract the array
        let rhs_poly = self.expression_to_object(env, &let_stmt.expression, None)?;

        // match rhs_poly {
        //     Object::Array(arr) => {
        //         env.store(variable_name, Object::Array(arr));
        //     }
        //     _ => unimplemented!(
        //         "logic for types that are not arrays in a let statement, not implemented yet!"
        //     ),
        // };
        let todo_id = self.dummy();
        Ok(todo_id)
    }

    pub(crate) fn expression_to_object(
        &mut self,
        env: &mut Environment,
        expr_id: &ExprId,
        create_phi: Option<arena::Index>,
    ) -> Result<arena::Index, RuntimeError> {
        let expr = self.context().def_interner.expression(expr_id);
        let span = self.context().def_interner.expr_span(expr_id);
        match expr {
            HirExpression::Literal(HirLiteral::Integer(x)) =>
            Ok(self.new_constant(x)),
            HirExpression::Literal(HirLiteral::Array(arr_lit)) => {
                //TODO!!! how to handle arrays?
                todo!();
                Ok(self.new_constant(FieldElement::zero()))
                //Ok(Object::Array(Array::from(self, env, arr_lit)?)) 
            },
            HirExpression::Ident(x) =>  {
                Ok(self.evaluate_identifier(env, &x, create_phi))
                //n.b this creates a new variable if it does not exist, may be we should delegate this to explicit statements (let) - TODO
            },
            HirExpression::Infix(infx) => {
                let lhs = self.expression_to_object(env, &infx.lhs, create_phi)?;
                let rhs = self.expression_to_object(env, &infx.rhs, create_phi)?;
                let ins = self.evaluate_infix_expression(lhs, rhs, infx.operator);
                //si lhs est une var + creatphi=> create phi
                ins
            },
            HirExpression::Cast(cast_expr) => {
                let lhs = self.expression_to_object(env, &cast_expr.lhs, create_phi)?;
                todo!();
                //We should generate a cast instruction and handle properly type conversion:
                // unsigned integer to field ; ok, just checks if bit size over FieldElement::max_num_bits()
                // signed integer to field; ok; check bit size N, retrieve sign bit s and returns x*(1-s)+s*(p-2^N+x)
                // field to unsigned integer; returns x mod 2^N when N is the bit size of the result type
                // field to signed integer; ??
                // bool to integer or field, ok: returns if (x is true) 1 else 0
                // integer to field vers bool: ok, returns (x neq 0)
                // integer to other integer type: checks rust rules TODO
                // else... Not supported (for now).
                //binary_op::handle_cast_op(self,lhs, cast_expr.r#type).map_err(|kind|kind.add_span(span))
            },
            HirExpression::Index(indexed_expr) => {
                todo!();
                // Currently these only happen for arrays
                let arr_name = self.context().def_interner.ident_name(&indexed_expr.collection_name);
                let ident_span = self.context().def_interner.ident_span(&indexed_expr.collection_name);
                let arr = env.get_array(&arr_name).map_err(|kind|kind.add_span(ident_span))?;
                //
                // Evaluate the index expression
                //TODO should check whether it is an assignment or not to generate the proper instruction
                // let index_as_obj = self.expression_to_object(env, &indexed_expr.index)?;
                // let index_as_constant = match index_as_obj.constant() {
                //     Ok(v) => v,
                //     Err(_) => panic!("Indexed expression does not evaluate to a constant")
                // };
                // let index_as_u128 = index_as_constant.to_u128();
                // arr.get(index_as_u128).map_err(|kind|kind.add_span(span))
            },
            HirExpression::Call(call_expr) => {
                todo!();
                let func_meta = self.context().def_interner.function_meta(&call_expr.func_id);
                //TODO generate a new block and checks whether how arguments should be passed (copy or ref)?
                // Choices are a low level func or an imported library function
                // If low level, then we use it's func name to find out what function to call
                // If not then we just call the library as usual with the function definition
                // todo..match func_meta.kind {
                //     FunctionKind::Normal => self.call_function(env, &call_expr, call_expr.func_id),
                //     FunctionKind::LowLevel => {
                //         let attribute = func_meta.attributes.expect("all low level functions must contain an attribute which contains the opcode which it links to");
                //         let opcode_name = attribute.foreign().expect("ice: function marked as foreign, but attribute kind does not match this");
                //         low_level_function_impl::call_low_level(self, env, opcode_name, (call_expr, span))
                //     },
                //     FunctionKind::Builtin => {
                //         let attribute = func_meta.attributes.expect("all builtin functions must contain an attribute which contains the function name which it links to");
                //         let builtin_name = attribute.builtin().expect("ice: function marked as a builtin, but attribute kind does not match this");
                //         builtin::call_builtin(self, env, builtin_name, (call_expr,span))
                //     },
                // ...todo }
            },
            HirExpression::For(for_expr) => self.handle_for_expr(env,for_expr).map_err(|kind|kind.add_span(span)),
            HirExpression::If(_) => todo!(),
            HirExpression::Prefix(_) => todo!(),
            HirExpression::Predicate(_) => todo!(),
            HirExpression::Literal(_) => todo!(),
            HirExpression::Block(_) => todo!("currently block expressions not in for/if branches are not being evaluated. In the future, we should be able to unify the eval_block and all places which require block_expr here")
        }
    }

    pub fn get_instruction_max(
        &self,
        ins: &node::Instruction,
        max_map: &mut HashMap<arena::Index, BigUint>,
        vmap: &HashMap<arena::Index, arena::Index>,
    ) -> BigUint {
        let r_max = self.get_obj_max_value(None, ins.rhs, max_map, vmap);
        let l_max = self.get_obj_max_value(None, ins.lhs, max_map, vmap);
        let i_max = ins.get_max_value(l_max, r_max);

        i_max
    }

    // Retrieve max possible value of a node; from the max_map if it was already computed
    // or else we compute it.
    // we use the value array (get_current_value2) in order to handle truncate instructions
    // we need to do it because rust did not allow to modify the instruction in block_overflow..
    pub fn get_obj_max_value(
        &self,
        obj: Option<&node::NodeObj>,
        idx: arena::Index,
        max_map: &mut HashMap<arena::Index, BigUint>,
        vmap: &HashMap<arena::Index, arena::Index>,
    ) -> BigUint {
        let id = get_current_value2(idx, vmap); //block.get_current_value(idx);
        if max_map.contains_key(&id) {
            return max_map[&id].clone();
        }
        let obj_;
        if obj.is_none() {
            obj_ = self.get_object(id).unwrap();
        } else {
            obj_ = obj.unwrap();
        }

        let result: BigUint;
        result = match obj_ {
            node::NodeObj::Obj(v) => BigUint::from((1_u128 << v.bits()) - 1), //TODO check for signed type
            node::NodeObj::Instr(i) => self.get_instruction_max(&i, max_map, vmap),
            node::NodeObj::Const(c) => c.value.clone(), //TODO panic for string constants
        };
        max_map.insert(id, result.clone());
        result
    }

    pub fn truncate(
        &mut self,
        obj_id: arena::Index,
        bit_size: u32,
        max_map: &mut HashMap<arena::Index, BigUint>,
    ) -> arena::Index {
        // get type
        let obj = self.get_object(obj_id).unwrap();
        let obj_type = obj.get_type();
        let obj_name = obj.print();
        //ensure truncate is needed:
        let v_max = &max_map[&obj_id];
        if *v_max >= BigUint::from(1_u128 << bit_size) {
            let rhs_bitsize = self.new_constant(FieldElement::from(bit_size as i128)); //TODO is this leaking some info????

            //Create a new truncate instruction '(idx): obj trunc bit_size'
            //set current value of obj to idx
            let mut i =
                node::Instruction::new(node::Operation::trunc, obj_id, rhs_bitsize, obj_type, None);
            if i.res_name.ends_with("_t") {
                //TODO we should use %t so that we can check for this substring (% is not a valid char for a variable name) in the name and then write name%t[number+1]
            }
            i.res_name = obj_name + "_t";
            i.bit_size = v_max.bits() as u32;
            let i_id = self.nodes.insert(node::NodeObj::Instr(i));
            max_map.insert(i_id, BigUint::from((1_u128 << bit_size) - 1));
            return i_id;
            //we now need to call fix_truncate(), it is done in a separate function in order to not overwhelm the
            //arguments list!
        }
        self.dummy()
    }

    //Set the id and parent block of the truncate instruction
    //This is needed because the instruction is inserted into a block and not added in the current block like regular instructions
    //We also update the value array
    pub fn fix_truncate(
        &mut self,
        idx: arena::Index,
        prev_id: arena::Index,
        block_idx: arena::Index,
        vmap: &mut HashMap<arena::Index, arena::Index>,
    ) {
        if let Some(ins) = self.get_as_mut_instruction(idx) {
            ins.idx = idx;
            ins.parent_block = block_idx;
            vmap.insert(prev_id, idx); //block.update_variable(prev_id, idx);
        }
    }

    fn add_to_truncate(
        &self,
        obj_id: arena::Index,
        bit_size: u32,
        to_truncate: &mut HashMap<arena::Index, u32>,
        max_map: &HashMap<arena::Index, BigUint>,
    ) -> BigUint {
        let v_max = &max_map[&obj_id];
        if *v_max >= BigUint::from(1_u128 << bit_size) {
            let obj = self.get_object(obj_id).unwrap();
            match obj {
                node::NodeObj::Const(_) => {
                    return v_max.clone(); //a constant cannot be truncated, so we exit the function gracefully
                }
                _ => {}
            }
            let truncate_bits;
            if to_truncate.contains_key(&obj_id) {
                truncate_bits = u32::min(to_truncate[&obj_id], bit_size);
                to_truncate.insert(obj_id, truncate_bits);
            } else {
                to_truncate.insert(obj_id, bit_size);
                truncate_bits = bit_size;
            }
            return BigUint::from(truncate_bits - 1);
        }
        return v_max.clone();
    }

    fn process_to_truncate(
        &mut self,
        new_list: &mut Vec<arena::Index>,
        to_truncate: &mut HashMap<arena::Index, u32>,
        max_map: &mut HashMap<arena::Index, BigUint>,
        block_idx: arena::Index,
        vmap: &mut HashMap<arena::Index, arena::Index>,
    ) {
        for (id, bit_size) in to_truncate.iter() {
            let truncate_idx = self.truncate(*id, *bit_size, max_map); //TODO properly handle signed arithmetic...
            self.fix_truncate(truncate_idx, *id, block_idx, vmap);
            new_list.push(truncate_idx);
        }
        to_truncate.clear();
    }

    pub fn update_ins_parameters(
        &mut self,
        idx: arena::Index,
        lhs: arena::Index,
        rhs: arena::Index,
        max_value: Option<BigUint>,
    ) {
        let mut ins = self.get_as_mut_instruction(idx).unwrap();
        ins.lhs = lhs;
        ins.rhs = rhs;
        if max_value.is_some() {
            ins.max_value = max_value.unwrap();
        }
    }

    pub fn overflow_strategy(&mut self) {
        let mut max_map: HashMap<arena::Index, BigUint> = HashMap::new();
        self.block_overflow(self.first_block, &mut max_map);
    }

    //overflow strategy over a block
    //TODO - check the type; we MUST NOT truncate or overflow field elements!!
    //TODO - to work properly with the CFG, we must propagte the value_map through-out the blocks
    pub fn block_overflow(
        &mut self,
        b_idx: arena::Index,
        //block: &mut node::BasicBlock,
        max_map: &mut HashMap<arena::Index, BigUint>,
    ) {
        //for each instruction, we compute the resulting max possible value (in term of the field representation of the operation)
        //when it is over the field charac, or if the instruction requires it, then we insert truncate instructions
        // The instructions are insterted in a duplicate list( because of rust ownership..), which we use for
        // processing another cse round for the block because the truncates may have added duplicate.
        let block = self.blocks.get(b_idx).unwrap();
        let mut b: Vec<node::Instruction> = Vec::new();
        let mut new_list: Vec<arena::Index> = Vec::new();
        let mut truncate_map: HashMap<arena::Index, u32> = HashMap::new();
        //RIA...
        for iter in &block.instructions {
            b.push((*self.get_as_instruction(*iter).unwrap()).clone());
        }
        let mut value_map: HashMap<arena::Index, arena::Index> = HashMap::new(); //since we process the block from the start, the block value array is not relevant
                                                                                 //block.value_array.clone();     //RIA - we need to modify it and to use it
                                                                                 //TODO we should try to make another simplify round here, or at least after copy propagation, we should do it at the best convenient place....TODO
        for mut ins in b {
            //TODO:. temp..
            if ins.operator == node::Operation::nop {
                continue;
            }
            //let mut ins = (*self.get_as_instruction(*ins_id).unwrap()).clone();
            //We retrieve get_current_value() in case a previous truncate has updated the value map
            let r_id = get_current_value2(ins.rhs, &value_map); //block.get_current_value(ins.rhs);
            let mut update_instruction = false;
            if r_id != ins.rhs {
                ins.rhs = r_id;
                update_instruction = true;
            }
            let l_id = get_current_value2(ins.lhs, &value_map); //block.get_current_value(ins.lhs);
            if l_id != ins.lhs {
                ins.lhs = l_id;
                update_instruction = true;
            }

            let r_obj = self.get_node(r_id).unwrap();
            let l_obj = self.get_node(l_id).unwrap();
            let r_max = self.get_obj_max_value(Some(r_obj), r_id, max_map, &value_map);
            self.get_obj_max_value(Some(l_obj), l_id, max_map, &value_map);
            //insert required truncates
            let to_truncate = ins.truncate_required(l_obj.bits(), r_obj.bits());
            if to_truncate.0 {
                //adds a new truncate(lhs) instruction
                self.add_to_truncate(l_id, l_obj.bits(), &mut truncate_map, max_map);
            }
            if to_truncate.1 {
                //adds a new truncate(rhs) instruction
                self.add_to_truncate(r_id, r_obj.bits(), &mut truncate_map, max_map);
            }
            if ins.operator == node::Operation::cast {
                //TODO for cast, we may need to reduce rhs into the bit size of lhs
                //this can change the max value of the cast so its need to be done here
                //(or we update the get_max_bits() for casts)
                let lhs_bits = l_obj.bits();
                if r_max.bits() as u32 > lhs_bits {
                    self.add_to_truncate(r_id, l_obj.bits(), &mut truncate_map, max_map);
                }
            }
            let mut ins_max = self.get_instruction_max(&ins, max_map, &value_map);
            if ins_max.bits() >= (FieldElement::max_num_bits() as u64) {
                //let's truncate a and b:
                //- insert truncate(lhs) dans la list des instructions
                //- insert truncate(rhs) dans la list des instructions
                //- update r_max et l_max
                //n.b we could try to truncate only one of them, but then we should check if rhs==lhs.
                let l_trunc_max =
                    self.add_to_truncate(l_id, l_obj.bits(), &mut truncate_map, max_map);
                let r_trunc_max =
                    self.add_to_truncate(r_id, r_obj.bits(), &mut truncate_map, max_map);
                ins_max = ins.get_max_value(l_trunc_max.clone(), r_trunc_max.clone());
                if ins_max.bits() >= FieldElement::max_num_bits().into() {
                    let message = format!(
                        "Require big int implementation, the bit size is too big for the field: {}, {}",
                        l_trunc_max.clone().bits(), r_trunc_max.clone().bits()
                    );
                    panic!("{}", message);
                }
            }
            self.process_to_truncate(
                &mut new_list,
                &mut truncate_map,
                max_map,
                b_idx,
                &mut value_map,
            );
            new_list.push(ins.idx);
            let l_new = get_current_value2(l_id, &value_map); //block.get_current_value(l_id);
            let r_new = get_current_value2(r_id, &value_map); //block.get_current_value(r_id);
            if l_new != l_id || r_new != r_id || ins.is_sub() {
                update_instruction = true;
            }
            if update_instruction {
                let mut max_r_value = None;
                if ins.is_sub() {
                    max_r_value = Some(max_map[&r_new].clone()); //for now we pass the max value to the instruction, we could also keep the max_map e.g in the block (or max in each nodeobj)
                                                                 //we may do that in future when the max_map becomes more used elsewhere (for other optim)
                }
                self.update_ins_parameters(ins.idx, l_new, r_new, max_r_value);
            }
        }
        self.update_value_array(b_idx, &value_map);
        let mut anchor: HashMap<node::Operation, VecDeque<arena::Index>> = HashMap::new();
        //We run another round of CSE for the block in order to remove possible duplicated truncates
        //let bb3 = self.blocks.get_mut(b_idx).unwrap();
        //bb3.instructions = new_list;
        self.block_cse(b_idx, &mut anchor, &mut new_list);
    }

    fn update_value_array(
        &mut self,
        b_id: arena::Index,
        vmap: &HashMap<arena::Index, arena::Index>,
    ) {
        let block = self.get_block_mut(b_id).unwrap();
        for (old, new) in vmap {
            block.value_array.insert(*old, *new); //TODO we must merge rather than update
        }
    }

    //TODO generate phi instructions
    fn handle_for_expr(
        &mut self,
        env: &mut Environment,
        for_expr: HirForExpression,
    ) -> Result<arena::Index, RuntimeErrorKind> {
        //we add the ' i = start' instruction (in the block before the join)
        let start_idx = self
            .expression_to_object(env, &for_expr.start_range, None)
            .map_err(|err| err.remove_span())
            .unwrap();
        let end_idx = self
            .expression_to_object(env, &for_expr.end_range, None)
            .map_err(|err| err.remove_span())
            .unwrap();
        //We support only const range for now
        let start = self.get_as_constant(start_idx).unwrap();
        //TODO how should we handle scope (cf. start/end_for_loop)?
        let iter_name = self
            .context
            .unwrap()
            .def_interner
            .ident_name(&for_expr.identifier);
        env.store(iter_name.clone(), Object::Constants(start));
        let iter_id = self.create_new_variable(iter_name, env); //TODO do we need to store and retrieve it ?
        let iter_type = self.get_object_type(iter_id);
        let iter_ass = self.new_instruction(iter_id, start_idx, node::Operation::ass, iter_type);

        //join block
        let prev_block = self.current_block;

        let join_idx = self.new_block(node::BlockType::ForJoin);
        //should parse a for_expr.condition statement that should evaluate to bool, but
        //we only supports i=start;i!=end for now
        //i1=phi(start);
        let i1 = node::Variable {
            id: iter_id,
            obj_type: iter_type,
            name: String::new(),
            cur_value: iter_id,
            root: None,
            witness: None,
        };
        let i1_id = self.add_variable(i1, Some(iter_id));
        let phi = self
            .generate_one_phi(join_idx, i1_id, iter_id, iter_ass, prev_block)
            .unwrap();
        self.update_variable_id(iter_id, i1_id, phi);
        let cond =
            self.new_instruction(phi, end_idx, node::Operation::ne, node::ObjectType::boolean);
        let to_fix = self.new_instruction(
            cond,
            self.dummy(),
            node::Operation::jeq,
            node::ObjectType::none,
        );

        //Body
        let body_idx = self.new_block(node::BlockType::Normal);
        let block = match self
            .context
            .unwrap()
            .def_interner
            .expression(&for_expr.block)
        {
            HirExpression::Block(block_expr) => block_expr,
            _ => panic!("ice: expected a block expression"),
        };
        let body_block1 = self.get_block_mut(body_idx).unwrap();
        body_block1.update_variable(iter_id, phi);
        let statements = block.statements();
        for stmt in statements {
            //evaluate_statement will generate phi instructions for all assignments
            self.evaluate_statement(env, stmt, Some(join_idx));
        }
        //increment iter
        let one = self.get_const(FieldElement::one(), iter_type);
        let incr = self.new_instruction(phi, one, node::Operation::add, iter_type);
        let body_block = self.get_block_mut(body_idx).unwrap();
        body_block.update_variable(iter_id, incr);
        //body.left = join
        body_block.left = Some(join_idx);
        //generate phi
        let value_array = body_block.value_array.clone(); //TODO can we avoid this clone??
        self.generate_phi(join_idx, &value_array, body_idx);

        //jump back to join
        self.new_instruction(
            self.dummy(),
            self.get_block(join_idx).unwrap().get_first_instruction(),
            node::Operation::jmp,
            node::ObjectType::none,
        );

        //exit block
        self.current_block = join_idx;
        let exit_id = self.new_block(node::BlockType::Normal);

        let exit_first = self.get_current_block().get_first_instruction();
        self.fixup(join_idx, Some(exit_id), Some(body_idx));
        let to_fix_ins = self.get_as_mut_instruction(to_fix);
        to_fix_ins.unwrap().rhs = body_idx;

        //for the dominator tree
        let join2 = self.get_block_mut(join_idx).unwrap();
        join2.dominated.push(body_idx);
        let mut va_tmp : HashMap<arena::Index, arena::Index> = HashMap::new();
        for i in &join2.instructions.clone() {
            if let Some(ins) = self.get_as_instruction(*i) {
                if ins.operator == node::Operation::phi {
                    va_tmp.insert(ins.rhs, ins.lhs);
                }
            }
        }
        //update value array of join block to expose the phi variables
        let join2 = self.get_block_mut(join_idx).unwrap();
        for mapped in va_tmp {
            join2.value_array.insert(mapped.0,mapped.1);
        }
        Ok(exit_first) //TODO what should we return???
    }

    pub fn acir(&self, evaluator: &mut Evaluator) {
        let mut acir = Acir::new();
        let mut fb = self.get_block(self.first_block);
        while  fb.is_some() {
            for iter in &fb.unwrap().instructions {
                let ins = self.get_instruction(*iter);
                acir.evaluate_instruction(ins, evaluator, self);
            }
             //TODO we should rather follow the jumps!
            if fb.unwrap().left.is_some() {
                fb = self.get_block(fb.unwrap().left.unwrap());
            }
            else {
                fb = None;
            }

        }

    }

    //If the Phi already exists, we merge, else we create a new one and update the value array.
    pub fn generate_phi(
        &mut self,
        target_block: arena::Index,
        value_array: &HashMap<arena::Index, arena::Index>,
        from: arena::Index,
    ) {
        let target = self.get_block(target_block).unwrap();
        let mut phi_list: Vec<node::Instruction> = Vec::new();
        let mut phi_merge_list: Vec<node::Instruction> = Vec::new();
        for v in value_array.keys() {
            //look for a phi for v:
            let mut to_insert = true;
            let root = self.get_root_id(*v);
            for i in &target.instructions {
                if let Some(ins) = self.get_as_instruction(*i) {
                    let mut phi_merge = ins.clone();
                   // if phi_merge.lhs == *v {
                       if phi_merge.rhs == root {
                        for arg in &mut phi_merge.phi_arguments {
                            if arg.1 == from {
                                arg.0 = value_array[v];
                                to_insert = false;
                            }
                        }
                        if to_insert == true {
                            phi_merge.phi_arguments.push((value_array[v], from));
                            to_insert = false;
                        }
                        phi_merge_list.push(phi_merge);
                    }
                }
                // let opt_ins = self.get_as_instruction(*i);
                // if opt_ins.is_some() && opt_ins.unwrap().operator == node::Operation::phi {
                //     dbg!("PHI MERGE!!!!");
                //     let mut phi_merge = opt_ins.unwrap().clone();
                //     if phi_merge.lhs == *v {
                //         for arg in &mut phi_merge.phi_arguments {
                //             if arg.1 == from {
                //                 arg.0 = value_array[v];
                //                 to_insert = false;
                //                 dbg!("we merge");
                //             }
                //         }
                //         if to_insert == true {
                //             phi_merge.phi_arguments.push((value_array[v], from));
                //             to_insert = false;
                //             dbg!(&phi_merge.phi_arguments);
                //         }
                //         phi_merge_list.push(phi_merge);
                //     }
                // }
            }
            if to_insert {
                let v_type = self.get_object_type(*v);
                let mut phi_ins = node::Instruction::new(
                    node::Operation::phi,
                    *v,
                    *v,
                    v_type,
                    Some(target_block),
                );
                phi_ins.phi_arguments.push((value_array[v], from));
                phi_list.push(phi_ins);
            }
        }
        //add new instructions to the arena
        let mut phi_list_id: Vec<arena::Index> = Vec::new();
        for i in phi_merge_list {
            let mut ins = self.get_as_mut_instruction(i.idx).unwrap();
            ins.phi_arguments = i.phi_arguments;
        }
        for i in phi_list {
            let idx = self.nodes.insert(node::NodeObj::Instr(i));
            let ins = self.get_as_mut_instruction(idx).unwrap();
            ins.idx = idx;
            phi_list_id.push(idx);
        }
        //add the instructions to the block
        let target = self.get_block_mut(target_block).unwrap();
        for i in phi_list_id {
            target.instructions.insert(1, i);
        }
    }

    //If the Phi already exists, we merge, else we create a new one and update the value array.
    pub fn generate_one_phi(
        &mut self,
        target_block: arena::Index,
        v: arena::Index,
        v2: arena::Index,        //  root: arena::Index,
        value: arena::Index,
        from: arena::Index,
    ) -> Option<arena::Index> {
        let target = self.get_block(target_block).unwrap();
        let mut result = self.dummy();
        //look for a phi for root:
        let root = self.get_root_id(v2);
        let mut to_insert = true;
        let mut to_update = false;
        for i in &target.instructions {
            let opt_ins = self.get_as_instruction(*i);

            if opt_ins.is_some()
                && opt_ins.unwrap().operator == node::Operation::phi
                && opt_ins.unwrap().rhs == root
            {
                let mut phi_merge = opt_ins.unwrap().clone();
                for arg in &mut phi_merge.phi_arguments {
                    if arg.1 == from {
                        if arg.0 != value {
                            arg.0 = value;
                            to_update = true;
                        }
                        to_insert = false;
                    }
                }
                if to_insert == true {
                    to_update = true;
                    phi_merge.phi_arguments.push((value, from));
                    to_insert = false;
                }
                result = *i;
            }
        }
        if to_insert {
            let v_type = self.get_object_type(v);
            let mut phi_ins =
                node::Instruction::new(node::Operation::phi, v, root, v_type, Some(target_block));
            phi_ins.phi_arguments.push((value, from));
            if let Some(prev_block) = target.predecessor.first() {
                if *prev_block != from {
                    phi_ins.phi_arguments.push((v2, *prev_block));
                }
            }            
            //phi_list.push(phi_ins);
            let phi_id = self.nodes.insert(node::NodeObj::Instr(phi_ins));
            //ria
            //we create a new variable representing the PHI result, we store it as PHI.lhs
            let new_v = self.new_ssa_variable(v);
            let mut phi_ins2 = self.get_as_mut_instruction(phi_id).unwrap();
            phi_ins2.idx = phi_id;
            phi_ins2.lhs = new_v;
            dbg!(phi_ins2);
            let target2 = self.get_block_mut(target_block).unwrap();
            target2.instructions.insert(1, phi_id); //We insert after the dummy instruction
            result = phi_id;
        }
        //update value array todo??
        if to_insert {
            let from_block = self.get_block_mut(from);
            if from_block.is_some() {
               //from_block.unwrap().update_variable(v, value)   //todo??
            }
            return Some(result);
        }
        None
    }
}

pub fn get_current_value2(
    idx: arena::Index,
    vmap: &HashMap<arena::Index, arena::Index>,
) -> arena::Index {
    match vmap.get(&idx) {
        Some(cur_idx) => *cur_idx,
        None => idx,
    }
}
