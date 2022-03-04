use super::block::{BasicBlock, BlockId};
use super::node::{Instruction, NodeId, NodeObj, Operation};
use super::{block, flatten, integer, node, optim, ssa_form};
use std::collections::HashMap;
use std::collections::HashSet;

//use std::io;

use super::super::environment::Environment;
use super::super::errors::{RuntimeError, RuntimeErrorKind};
use crate::object::Object;
use crate::ssa::acir_gen::Acir;
use crate::ssa::node::Node;
use crate::Evaluator;
use acvm::FieldElement;
use arena;
use noirc_frontend::hir::Context;
use noirc_frontend::hir_def::function::HirFunction;
use noirc_frontend::hir_def::stmt::HirPattern;
use noirc_frontend::hir_def::{
    expr::{HirBinaryOp, HirBinaryOpKind, HirExpression, HirForExpression, HirLiteral},
    stmt::{HirConstrainStatement, HirLetStatement, HirStatement},
};
use noirc_frontend::node_interner::{ExprId, IdentId, StmtId};
//use noirc_frontend::{FunctionKind, Type};
use num_bigint::BigUint;

// This is a 'master' class for generating the SSA IR from the AST
// It contains all the data; the node objects representing the source code in the nodes arena
// and The CFG in the blocks arena
// everything else just reference objects from these two arena using their index.
pub struct IRGenerator<'a> {
    pub context: Option<&'a Context>,

    pub first_block: BlockId,
    pub current_block: BlockId,
    blocks: arena::Arena<block::BasicBlock>,
    pub nodes: arena::Arena<node::NodeObj>,
    pub id0: arena::Index, //dummy index.. should we put a dummy object somewhere?
    pub value_name: HashMap<NodeId, u32>,
    pub sealed_blocks: HashSet<BlockId>,
}

impl<'a> IRGenerator<'a> {
    pub fn new(context: &Context) -> IRGenerator {
        let mut pc = IRGenerator {
            context: Some(context),
            id0: IRGenerator::dummy_id(),
            first_block: BlockId::dummy(),
            current_block: BlockId::dummy(),
            blocks: arena::Arena::new(),
            nodes: arena::Arena::new(),
            // dummy_instruction: ParsingContext::dummy_id(),
            value_name: HashMap::new(),
            sealed_blocks: HashSet::new(),
        };
        block::create_first_block(&mut pc);
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
                    ins_str += &format!(
                        "{:?}:{:?}, ",
                        v.0.into_raw_parts().0,
                        b.0.into_raw_parts().0
                    );
                }
                ins_str += ")";
            }
            println!("{}: {}", str_res, ins_str);
        }
    }

    pub fn print(&self) {
        for (i, (_, b)) in self.blocks.iter().enumerate() {
            println!("************* Block n.{}", i);
            self.print_block(b);
        }
    }

    pub fn context(&self) -> &Context {
        self.context.unwrap()
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

    pub fn find_variable(&self, definition: &Option<IdentId>) -> Option<&node::Variable> {
        if definition.is_none() {
            return None;
        }
        for (_, o) in &self.nodes {
            if let node::NodeObj::Obj(v) = o {
                if v.def == *definition {
                    return Some(v);
                }
            }
        }
        None
    }

    pub fn find_const(&self, value: &BigUint) -> Option<NodeId> {
        //TODO We should map constant values to id
        for (idx, o) in &self.nodes {
            if let node::NodeObj::Const(c) = o {
                if c.value == *value {
                    return Some(NodeId(idx));
                }
            }
        }
        None
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

    fn get_object_type(&self, id: NodeId) -> node::ObjectType {
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

    pub fn new_instruction(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        opcode: node::Operation,
        optype: node::ObjectType,
    ) -> NodeId {
        //Add a new instruction to the nodes arena
        let cb = self.get_current_block();

        let mut i = node::Instruction::new(opcode, lhs, rhs, optype, Some(cb.id));
        //Basic simplification
        optim::simplify(self, &mut i);
        if i.is_deleted {
            return i.rhs;
        }
        self.push_instruction(i)
    }

    //Retrieve the object conresponding to the const value given in argument
    // If such object does not exist, we create one
    //TODO: handle type
    pub fn get_or_create_const(&mut self, x: FieldElement, t: node::ObjectType) -> NodeId {
        let value = BigUint::from_bytes_be(&x.to_bytes()); //TODO a const should be a field element
        if let Some(obj) = self.find_const(&value)
        //todo type
        {
            return obj;
        }

        self.add_const(node::Constant {
            id: NodeId::dummy(),
            value,
            value_str: String::new(),
            value_type: t,
        })
    }

    //TODO the type should be provided by previous step so we can use get_const() instead
    pub fn new_constant(&mut self, x: FieldElement) -> NodeId {
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
        if num_bits < 32 {
            self.add_const(node::Constant {
                id: NodeId::dummy(),
                value,
                value_type: node::ObjectType::Signed(32),
                value_str: String::new(),
            })
        } else {
            //idx = self.id0;
            todo!();
            //we should support integer of size <  integer::short_integer_max_bit_size(), because else we cannot do multiplication!
            //for bigger size, we will need to represent an integer using several field elements, it may be easier to implement them in Noir! (i.e as a Noir library)
        }
    }

    //same as update_variable but using the var index instead of var
    pub fn update_variable_id(&mut self, var_id: NodeId, new_var: NodeId, new_value: NodeId) {
        let root_id = self.get_root_value(var_id);
        let root = self.get_variable(root_id).unwrap();
        let root_name = root.name.clone();
        let cb = self.get_current_block_mut();
        cb.update_variable(var_id, new_value);
        let vname = self.value_name.entry(var_id).or_insert(0);
        *vname += 1;
        let variable_id = *vname;

        if let Ok(nvar) = self.get_mut_variable(new_var) {
            nvar.name = format!("{root_name}{variable_id}");
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

    ////////////////PARSING THE AST//////////////////////////////////////////////
    /// Compiles the AST into the intermediate format by evaluating the main function
    pub fn evaluate_main(
        &mut self,
        env: &mut Environment,
        context: &'a Context,
        main_func_body: HirFunction, //main function
    ) -> Result<(), RuntimeError> {
        self.context = Some(context);

        let block = main_func_body.block(&context.def_interner);
        for stmt_id in block.statements() {
            self.evaluate_statement(env, stmt_id)?;
        }

        Ok(())
    }

    //Optimise, flatten and truncate IR and then generates ACIR representation from it
    pub fn ir_to_acir(&mut self, evaluator: &mut Evaluator) -> Result<(), RuntimeError> {
        //let mut number = String::new();

        //SSA
        dbg!("SSA:");
        self.print();
        //Optimisation
        block::compute_dom(self);
        dbg!("CSE:");
        optim::cse(self);
        self.print();
        //Unrolling
        dbg!("unrolling:");
        flatten::unroll_tree(self);
        optim::cse(self);
        self.print();
        //Truncation
        integer::overflow_strategy(self);
        self.print();
        //println!("Press enter to continue");
        //io::stdin().read_line(&mut number);
        //ACIR
        self.acir(evaluator);
        dbg!("DONE");
        Ok(())
    }

    fn evaluate_identifier(&mut self, env: &mut Environment, ident_id: &IdentId) -> NodeId {
        let ident_name = self.context.unwrap().def_interner.ident_name(ident_id);
        let ident_def = self.context.unwrap().def_interner.ident_def(ident_id);
        // let var = self.find_variable(&ident_def); //TODO by name or by id?
        // if let Some(node::Variable { id, .. }) = self.find_variable(&ident_def) {
        if let Some(var) = self.find_variable(&ident_def) {
            let id = var.id;
            return ssa_form::get_current_value(self, id);
        }
        let obj = env.get(&ident_name);
        let obj_type = node::ObjectType::get_type_from_object(&obj);

        //new variable - should be in a let statement? The let statement should set the type
        let obj = node::Variable {
            id: NodeId::dummy(),
            name: ident_name.clone(),
            obj_type,
            root: None,
            def: ident_def,
            witness: node::get_witness_from_object(&obj),
            parent_block: self.current_block,
        };

        let v_id = self.add_variable(obj, None);
        self.get_current_block_mut().update_variable(v_id, v_id);
        v_id
    }

    //Cast lhs into type rtype. a cast b means (a) b
    fn new_cast_expression(&mut self, lhs: NodeId, rtype: node::ObjectType) -> NodeId {
        //generate instruction 'a cast a', with result type rtype
        self.push_instruction(Instruction::new(
            node::Operation::Cast,
            lhs,
            lhs,
            rtype,
            Some(self.current_block),
        ))
    }

    fn evaluate_infix_expression(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        op: HirBinaryOp,
    ) -> Result<NodeId, RuntimeError> {
        let ltype = self.get_object_type(lhs);

        let optype = ltype; //n.b. we do not verify rhs type as it should have been handled by the typechecker.

        // Get the opcode from the infix operator
        let opcode = node::to_operation(op.kind, optype);
        Ok(self.new_instruction(lhs, rhs, opcode, optype))
    }

    pub fn evaluate_statement(
        &mut self,
        env: &mut Environment,
        stmt_id: &StmtId,
    ) -> Result<NodeId, RuntimeError> {
        let statement = self.context().def_interner.statement(stmt_id);
        match statement {
            HirStatement::Constrain(constrain_stmt) => {
                self.handle_constrain_statement(env, constrain_stmt)
            }
            HirStatement::Expression(expr) | HirStatement::Semi(expr) => {
                self.expression_to_object(env, &expr)
            }
            HirStatement::Let(let_stmt) => {
                // let statements are used to declare a higher level object
                self.handle_let_statement(env, let_stmt)
            }
            HirStatement::Assign(assign_stmt) => {
                let ident_def = self
                    .context()
                    .def_interner
                    .ident_def(&assign_stmt.identifier);
                //////////////TODO temp this is needed because we don't parse main arguments
                let ident_name = self
                    .context()
                    .def_interner
                    .ident_name(&assign_stmt.identifier);

                let lhs = if let Some(variable) = self.find_variable(&ident_def) {
                    variable
                } else {
                    //var is not defined,
                    //let's do it here for now...TODO
                    let obj = env.get(&ident_name);
                    let obj_type = node::ObjectType::get_type_from_object(&obj);
                    let new_var2 = node::Variable {
                        id: NodeId::dummy(),
                        obj_type,
                        name: ident_name.clone(),
                        root: None,
                        def: ident_def,
                        witness: node::get_witness_from_object(&obj),
                        parent_block: self.current_block,
                    };
                    let new_var2_id = self.add_variable(new_var2, None);
                    self.get_current_block_mut()
                        .update_variable(new_var2_id, new_var2_id); //DE MEME
                    self.get_variable(new_var2_id).unwrap()
                };

                //////////////////////////////----******************************************
                let new_var = node::Variable {
                    id: lhs.id,
                    obj_type: lhs.obj_type,
                    name: String::new(),
                    root: None,
                    def: ident_def,
                    witness: None,
                    parent_block: self.current_block,
                };
                let ls_root = lhs.get_root();

                //ssa: we create a new variable a1 linked to a
                let new_var_id = self.add_variable(new_var, Some(ls_root));

                let rhs_id = self.expression_to_object(env, &assign_stmt.expression)?;
                let rhs = &self[rhs_id];
                let r_type = rhs.get_type();
                let r_id = rhs.get_id();
                let result = self.new_instruction(new_var_id, r_id, node::Operation::Ass, r_type);
                self.update_variable_id(ls_root, new_var_id, result); //update the name and the value map
                Ok(result)
            }
            HirStatement::Error => unreachable!(
                "ice: compiler did not exit before codegen when a statement failed to parse"
            ),
        }
    }

    fn create_new_variable(
        &mut self,
        var_name: String,
        def: Option<IdentId>,
        env: &mut Environment,
    ) -> NodeId {
        let obj = env.get(&var_name);
        let obj_type = node::ObjectType::get_type_from_object(&obj);
        let new_var = node::Variable {
            id: NodeId::dummy(),
            obj_type,
            name: var_name,
            root: None,
            def,
            witness: node::get_witness_from_object(&obj),
            parent_block: self.current_block,
        };
        self.add_variable(new_var, None)
    }

    // Add a constraint to constrain two expression together
    fn handle_constrain_statement(
        &mut self,
        env: &mut Environment,
        constrain_stmt: HirConstrainStatement,
    ) -> Result<NodeId, RuntimeError> {
        let lhs = self.expression_to_object(env, &constrain_stmt.0.lhs)?;
        let rhs = self.expression_to_object(env, &constrain_stmt.0.rhs)?;

        let result =
        match constrain_stmt.0.operator.kind  {
            // HirBinaryOpKind::Add => binary_op::handle_add_op(lhs, rhs, self),
            // HirBinaryOpKind::Subtract => binary_op::handle_sub_op(lhs, rhs, self),
            // HirBinaryOpKind::Multiply => binary_op::handle_mul_op(lhs, rhs, self),
            // HirBinaryOpKind::Divide => binary_op::handle_div_op(lhs, rhs, self),
            HirBinaryOpKind::NotEqual => todo!(),
            HirBinaryOpKind::Equal => Ok(self.new_instruction(lhs, rhs, node::Operation::EqGate, node::ObjectType::NotAnObject)),
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
            // we should replace one with the other 'everywhere'
            // we should merge their property; min(max), min(bitsize),etc..
        };
        result
    }

    fn pattern_name_and_def(&self, pattern: &HirPattern) -> (String, Option<IdentId>) {
        match pattern {
            HirPattern::Identifier(id) => {
                let interner = &self.context().def_interner;
                (interner.ident_name(id), interner.ident_def(id))
            }
            HirPattern::Mutable(pattern, _) => self.pattern_name_and_def(pattern),
            HirPattern::Tuple(_, _) => todo!("Implement tuples in the backend"),
            HirPattern::Struct(_, _, _) => todo!("Implement structs in the backend"),
        }
    }

    // Let statements are used to declare higher level objects
    fn handle_let_statement(
        &mut self,
        env: &mut Environment,
        let_stmt: HirLetStatement,
    ) -> Result<NodeId, RuntimeError> {
        //create a variable from the left side of the statement, evaluate the right and generate an assign instruction.

        // Extract the expression
        let rhs_id = self.expression_to_object(env, &let_stmt.expression)?;
        //TODO: is there always an expression? if not, how can we get the type of the variable?
        let rtype = self[rhs_id].get_type();

        // Convert the LHS into an identifier
        let (variable_name, ident_def) = self.pattern_name_and_def(&let_stmt.pattern);
        //Create a new variable;
        //TODO in the name already exists, we should use something else (from env) to find a variable (identid?)

        let new_var = node::Variable {
            id: NodeId::dummy(),
            obj_type: rtype, //TODO - what if type is defined on lhs only?
            name: variable_name,
            root: None,
            def: ident_def,
            witness: None,
            parent_block: self.current_block,
        };
        let id = self.add_variable(new_var, None);
        dbg!(id);

        //Assign rhs to lhs
        let result = self.new_instruction(id, rhs_id, node::Operation::Ass, rtype);
        //This new variable should not be available in outer scopes.
        let cb = self.get_current_block_mut();
        cb.update_variable(id, result); //update the value array. n.b. we should not update the name as it is the first assignment (let)
        Ok(result)
    }

    pub(crate) fn expression_to_object(
        &mut self,
        env: &mut Environment,
        expr_id: &ExprId,
    ) -> Result<NodeId, RuntimeError> {
        let expr = self.context().def_interner.expression(expr_id);
        let span = self.context().def_interner.expr_span(expr_id);
        match expr {
            HirExpression::Literal(HirLiteral::Integer(x)) =>
            Ok(self.new_constant(x)),
            HirExpression::Literal(HirLiteral::Array(_arr_lit)) => {
                //TODO - handle arrays
                todo!();
                //Ok(Object::Array(Array::from(self, env, _arr_lit)?)) 
            },
            HirExpression::Ident(x) =>  {
                Ok(self.evaluate_identifier(env, &x))
                //n.b this creates a new variable if it does not exist, may be we should delegate this to explicit statements (let) - TODO
            },
            HirExpression::Infix(infx) => {
                let lhs = self.expression_to_object(env, &infx.lhs)?;
                let rhs = self.expression_to_object(env, &infx.rhs)?;
                self.evaluate_infix_expression(lhs, rhs, infx.operator)
            },
            HirExpression::Cast(cast_expr) => {
                let lhs = self.expression_to_object(env, &cast_expr.lhs)?;
                let rtype = node::ObjectType::from_type(cast_expr.r#type);
                Ok(self.new_cast_expression(lhs, rtype))

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
                // Currently these only happen for arrays
                let arr_name = self.context().def_interner.ident_name(&indexed_expr.collection_name);
                let ident_span = self.context().def_interner.ident_span(&indexed_expr.collection_name);
                let _arr = env.get_array(&arr_name).map_err(|kind|kind.add_span(ident_span))?;
                //
                // Evaluate the index expression
                let index_as_obj = self.expression_to_object(env, &indexed_expr.index)?;
                let index_as_u128 = if let Some(index_as_constant) = self.get_as_constant(index_as_obj) {
                    index_as_constant.to_u128()
                }
                else {
                    panic!("Indexed expression does not evaluate to a constant");
                };
                dbg!(index_as_u128);
                todo!();
                //should return array + index
                // arr.get(index_as_u128).map_err(|kind|kind.add_span(span))
            },
            HirExpression::Call(call_expr) => {
                let _func_meta = self.context().def_interner.function_meta(&call_expr.func_id);
                todo!();
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
            HirExpression::Literal(_) => todo!(),
            HirExpression::Block(_) => todo!("currently block expressions not in for/if branches are not being evaluated. In the future, we should be able to unify the eval_block and all places which require block_expr here"),
            HirExpression::Constructor(_) => todo!(),
            HirExpression::Tuple(_) => todo!(),
            HirExpression::MemberAccess(_) => todo!(),
            HirExpression::Error => todo!(),
        }
    }

    //TODO generate phi instructions
    fn handle_for_expr(
        &mut self,
        env: &mut Environment,
        for_expr: HirForExpression,
    ) -> Result<NodeId, RuntimeErrorKind> {
        //we add the ' i = start' instruction (in the block before the join)
        let start_idx = self
            .expression_to_object(env, &for_expr.start_range)
            .map_err(|err| err.remove_span())
            .unwrap();
        let end_idx = self
            .expression_to_object(env, &for_expr.end_range)
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
        let iter_def = self
            .context
            .unwrap()
            .def_interner
            .ident_def(&for_expr.identifier);
        env.store(iter_name.clone(), Object::Constants(start));
        let iter_id = self.create_new_variable(iter_name, iter_def, env); //TODO do we need to store and retrieve it ?
        let iter_var = self.get_mut_variable(iter_id).unwrap();
        iter_var.obj_type = node::ObjectType::Unsigned(32); //TODO create_new_variable should set the correct type
        let iter_type = self.get_object_type(iter_id);
        dbg!(iter_type);
        let iter_ass = self.new_instruction(iter_id, start_idx, node::Operation::Ass, iter_type);
        //We map the iterator to start_idx so that when we seal the join block, we will get the corrdect value.
        self.update_variable_id(iter_id, iter_ass, start_idx);

        //join block
        let join_idx = block::new_unsealed_block(self, block::BlockType::ForJoin, true);
        let exit_id = block::new_sealed_block(self, block::BlockType::Normal);
        self.current_block = join_idx;
        //should parse a for_expr.condition statement that should evaluate to bool, but
        //we only supports i=start;i!=end for now
        //i1=phi(start);
        let i1 = node::Variable {
            id: iter_id,
            obj_type: iter_type,
            name: String::new(),
            root: None,
            def: None,
            witness: None,
            parent_block: join_idx,
        };
        let i1_id = self.add_variable(i1, Some(iter_id)); //TODO we do not need them
                                                          //we generate the phi for the iterator because the iterator is manually created
        let phi = self.generate_empty_phi(join_idx, iter_id);
        self.update_variable_id(iter_id, i1_id, phi); //j'imagine que y'a plus besoin
        let cond = self.new_instruction(phi, end_idx, Operation::Ne, node::ObjectType::Boolean);
        let to_fix = self.new_instruction(
            cond,
            NodeId::dummy(),
            node::Operation::Jeq,
            node::ObjectType::NotAnObject,
        );

        //Body
        let body_id = block::new_sealed_block(self, block::BlockType::Normal);
        let block = match self.context().def_interner.expression(&for_expr.block) {
            HirExpression::Block(block_expr) => block_expr,
            _ => panic!("ice: expected a block expression"),
        };
        let body_block1 = &mut self[body_id];
        body_block1.update_variable(iter_id, phi); //TODO try with just a get_current_value(iter)
        let statements = block.statements();
        for stmt in statements {
            self.evaluate_statement(env, stmt).unwrap(); //TODO return the error
        }

        //increment iter
        let one = self.get_or_create_const(FieldElement::one(), iter_type);
        let incr = self.new_instruction(phi, one, node::Operation::Add, iter_type);
        let cur_block_id = self.current_block; //It should be the body block, except if the body has CFG statements
        let cur_block = &mut self[cur_block_id];
        cur_block.update_variable(iter_id, incr);

        //body.left = join
        cur_block.left = Some(join_idx);
        let join_mut = &mut self[join_idx];
        join_mut.predecessor.push(cur_block_id);
        //jump back to join
        self.new_instruction(
            NodeId::dummy(),
            self[join_idx].get_first_instruction(),
            node::Operation::Jmp,
            node::ObjectType::NotAnObject,
        );
        //seal join
        ssa_form::seal_block(self, join_idx);

        //exit block
        self.current_block = exit_id;
        let exit_first = self.get_current_block().get_first_instruction();
        block::link_with_target(self, join_idx, Some(exit_id), Some(body_id));
        let first_instruction = self[body_id].get_first_instruction();
        self.try_get_mut_instruction(to_fix).unwrap().rhs = first_instruction;
        Ok(exit_first) //TODO what should we return???
    }

    pub fn acir(&self, evaluator: &mut Evaluator) {
        let mut acir = Acir::new();
        let mut fb = Some(&self[self.first_block]);
        while let Some(block) = fb {
            for iter in &block.instructions {
                let ins = self.get_instruction(*iter);
                acir.evaluate_instruction(ins, evaluator, self);
            }
            //TODO we should rather follow the jumps
            fb = block.left.map(|block_id| &self[block_id]);
        }
        //   dbg!(acir.arith_cache);
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

impl std::ops::Index<BlockId> for IRGenerator<'_> {
    type Output = BasicBlock;

    fn index(&self, index: BlockId) -> &Self::Output {
        &self.blocks[index.0]
    }
}

impl std::ops::IndexMut<BlockId> for IRGenerator<'_> {
    fn index_mut(&mut self, index: BlockId) -> &mut Self::Output {
        &mut self.blocks[index.0]
    }
}

impl std::ops::Index<NodeId> for IRGenerator<'_> {
    type Output = NodeObj;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl std::ops::IndexMut<NodeId> for IRGenerator<'_> {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}
