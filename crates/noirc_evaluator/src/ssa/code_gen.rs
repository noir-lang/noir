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
use noirc_frontend::hir_def::{
    expr::{HirBinaryOp, HirBinaryOpKind, HirExpression, HirForExpression, HirLiteral},
    stmt::{HirConstrainStatement, HirLetStatement, HirPrivateStatement, HirStatement},
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

    pub first_block: arena::Index,
    pub current_block: arena::Index,
    pub blocks: arena::Arena<block::BasicBlock>,
    pub nodes: arena::Arena<node::NodeObj>,
    pub id0: arena::Index, //dummy index.. should we put a dummy object somewhere?
    pub value_name: HashMap<arena::Index, u32>,
    pub sealed_blocks: HashSet<arena::Index>,
}

impl<'a> IRGenerator<'a> {
    pub fn new(context: &Context) -> IRGenerator {
        let mut pc = IRGenerator {
            context: Some(context),
            id0: IRGenerator::dummy_id(),
            first_block: IRGenerator::dummy_id(),
            current_block: IRGenerator::dummy_id(),
            blocks: arena::Arena::new(),
            nodes: arena::Arena::new(),
            // dummy_instruction: ParsingContext::dummy_id(),
            value_name: HashMap::new(),
            sealed_blocks: HashSet::new(),
        }; //, objects: arena::Arena::new()
        block::create_first_block(&mut pc);
        pc
    }

    //Display an object for debugging puposes
    fn to_string(&self, idx: arena::Index) -> String {
        if let Some(var) = self.get_object(idx) {
            return format!("{}", var);
        } else {
            return format!("unknown {:?}", idx.into_raw_parts().0);
        }
    }

    pub fn print_block(&self, b: &block::BasicBlock) {
        for idx in &b.instructions {
            let ins = self.get_instruction(*idx);
            let mut str_res = if ins.res_name.is_empty() {
                format!("{:?}", idx.into_raw_parts().0)
            } else {
                ins.res_name.clone()
            };
            if ins.is_deleted {
                str_res += " -DELETED";
            }
            let lhs_str = self.to_string(ins.lhs);
            let rhs_str = self.to_string(ins.rhs);
            let mut ins_str = format!("{} op:{:?} {}", lhs_str, ins.operator, rhs_str);

            if ins.operator == node::Operation::Phi {
                ins_str += "(";
                for (v, b) in &ins.phi_arguments {
                    ins_str += &format!("{:?}:{:?}, ", v.into_raw_parts().0, b.into_raw_parts().0);
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

    //Add an object to the nodes arena and set its id
    pub fn add_object(&mut self, obj: node::NodeObj) -> arena::Index {
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

    pub fn find_const(&self, value: &BigUint) -> Option<arena::Index> {
        //TODO We should map constant values to id
        for (idx, o) in &self.nodes {
            if let node::NodeObj::Const(c) = o {
                if c.value == *value {
                    return Some(idx);
                }
            }
        }
        None
    }

    pub fn dummy_id() -> arena::Index {
        arena::Index::from_raw_parts(std::usize::MAX, 0)
    }

    pub fn dummy(&self) -> arena::Index {
        IRGenerator::dummy_id()
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

    //Returns the object value if it is a constant, None if not. TODO: handle types
    pub fn get_as_constant(&self, idx: arena::Index) -> Option<FieldElement> {
        if let Some(node::NodeObj::Const(c)) = self.get_object(idx) {
            return Some(FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()));
        }
        None
    }

    //todo handle errors
    fn get_instruction(&self, idx: arena::Index) -> &node::Instruction {
        self.try_get_instruction(idx)
            .expect("Index not found or not an instruction")
    }

    pub fn get_mut_instruction(&mut self, idx: arena::Index) -> &mut node::Instruction {
        self.try_get_mut_instruction(idx)
            .expect("Index not found or not an instruction")
    }

    pub fn try_get_instruction(&self, idx: arena::Index) -> Option<&node::Instruction> {
        if let Some(node::NodeObj::Instr(i)) = self.get_object(idx) {
            return Some(i);
        }
        None
    }

    pub fn try_get_mut_instruction(&mut self, idx: arena::Index) -> Option<&mut node::Instruction> {
        if let Some(node::NodeObj::Instr(i)) = self.get_mut_object(idx) {
            return Some(i);
        }
        None
    }

    pub fn get_variable(&self, idx: arena::Index) -> Result<&node::Variable, &str> {
        //TODO proper error handling
        match self.nodes.get(idx) {
            Some(t) => match t {
                node::NodeObj::Obj(o) => Ok(o),
                _ => Err("Not an object"),
            },
            _ => Err("Invalid id"),
        }
    }

    pub fn get_mut_variable(&mut self, idx: arena::Index) -> Result<&mut node::Variable, &str> {
        //TODO proper error handling
        match self.nodes.get_mut(idx) {
            Some(t) => match t {
                node::NodeObj::Obj(o) => Ok(o),
                _ => Err("Not an object"),
            },
            _ => Err("Invalid id"),
        }
    }

    pub fn get_root_id(&self, var_id: arena::Index) -> arena::Index {
        let var = self.get_variable(var_id).unwrap();
        var.get_root()
    }

    pub fn add_variable(
        &mut self,
        obj: node::Variable,
        root: Option<arena::Index>,
    ) -> arena::Index {
        let idx = self.nodes.insert(node::NodeObj::Obj(obj));
        let obj2 = self.nodes.get_mut(idx).unwrap();
        match obj2 {
            node::NodeObj::Obj(v) => {
                v.id = idx;
                v.root = root;
            }
            _ => unreachable!(),
        }
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
            return i.rhs;
        }
        self.add_object(node::NodeObj::Instr(i))
    }

    //Retrieve the object conresponding to the const value given in argument
    // If such object does not exist, we create one
    //TODO: handle type
    pub fn get_const(&mut self, x: FieldElement, t: node::ObjectType) -> arena::Index {
        let value = BigUint::from_bytes_be(&x.to_bytes()); //TODO a const should be a field element
        if let Some(obj) = self.find_const(&value)
        //todo type
        {
            return obj;
        }
        let obj_cst = node::Constant {
            id: self.dummy(),
            value,
            value_str: String::new(),
            value_type: t,
        };
        let obj = node::NodeObj::Const(obj_cst);
        self.add_object(obj)
    }

    //TODO the type should be provided by previous step so we can use get_const() instead
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
        if num_bits < 32 {
            let obj_cst = node::Constant {
                id: self.id0,
                value,
                value_type: node::ObjectType::Signed(32),
                value_str: String::new(),
            };
            let obj = node::NodeObj::Const(obj_cst);
            self.add_object(obj)
        } else {
            //idx = self.id0;
            todo!();
            //we should support integer of size <  integer::short_integer_max_bit_size(), because else we cannot do multiplication!
            //for bigger size, we will need to represent an integer using several field elements, it may be easier to implement them in Noir! (i.e as a Noir library)
        }
    }

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
        self.value_name.entry(var_id).or_insert(1);
        self.value_name.insert(var_id, self.value_name[&var_id] + 1);
        //  let vname = cb.get_value_name(var_id).to_string();
        let vname = if self.value_name.contains_key(&var_id) {
            self.value_name[&var_id]
        } else {
            0
        }
        .to_string();
        if let Ok(nvar) = self.get_mut_variable(new_var) {
            nvar.name = root_name + &vname;
        }
    }
    //blocks/////////////////////////

    pub fn get_block_mut(&mut self, idx: arena::Index) -> Option<&mut block::BasicBlock> {
        self.blocks.get_mut(idx)
    }

    pub fn get_current_block_mut(&mut self) -> &mut block::BasicBlock {
        self.blocks.get_mut(self.current_block).unwrap()
    }

    pub fn try_get_block(&self, idx: arena::Index) -> Option<&block::BasicBlock> {
        self.blocks.get(idx)
    }
    pub fn get_block(&self, idx: arena::Index) -> &block::BasicBlock {
        self.blocks.get(idx).unwrap()
    }

    pub fn get_current_block(&self) -> &block::BasicBlock {
        self.blocks.get(self.current_block).unwrap()
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

    fn evaluate_identifier(&mut self, env: &mut Environment, ident_id: &IdentId) -> arena::Index {
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
            id: self.id0,
            name: ident_name.clone(),
            obj_type,
            root: None,
            def: ident_def,
            witness: node::get_witness_from_object(&obj),
            parent_block: self.current_block,
        };

        let v_id = self.add_variable(obj, None);
        self.get_block_mut(self.current_block)
            .unwrap()
            .update_variable(v_id, v_id);
        v_id
    }

    //Cast lhs into type rtype. a cast b means (a) b
    fn new_cast_expression(&mut self, lhs: arena::Index, rtype: node::ObjectType) -> arena::Index {
        //generate instruction 'a cast a', with result type rtype
        let i = node::Instruction::new(
            node::Operation::Cast,
            lhs,
            lhs,
            rtype,
            Some(self.current_block),
        );
        self.add_object(node::NodeObj::Instr(i))
    }

    fn evaluate_infix_expression(
        &mut self,
        lhs: arena::Index,
        rhs: arena::Index,
        op: HirBinaryOp,
    ) -> Result<arena::Index, RuntimeError> {
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
    ) -> Result<arena::Index, RuntimeError> {
        let statement = self.context().def_interner.statement(stmt_id);
        match statement {
            HirStatement::Private(x) => self.handle_private_statement(env, x),
            HirStatement::Constrain(constrain_stmt) => {
                self.handle_constrain_statement(env, constrain_stmt)
            }
            HirStatement::Const(x) => {
                //let variable_name: String = self.context().def_interner.ident_name(&x.identifier);
                // const can only be integers/Field elements, cannot involve the witness, so we can possibly move this to
                // analysis. Right now it would not make a difference, since we are not compiling to an intermediate Noir format
                //let span = self.context().def_interner.expr_span(&x.expression);
                //TODO the result of expression_to_object should be an assignement, we should modify the lhs to specify it is a const
                // and then forbid any other assignement with the same variable during the SSA phase (and instead of applying the SSA form of it).
                self.expression_to_object(env, &x.expression)
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
                let lhs = //self.find_variable(&ident_def).unwrap(); //left hand must be already declared
                if  let Some(var) = self.find_variable(&ident_def) {
                    var
                } else {
                    //var is not defined,
                    //let's do it here for now...TODO
                    let obj = env.get(&ident_name);
                    let obj_type = node::ObjectType::get_type_from_object(&obj);
                    let new_var2 = node::Variable {
                        id: self.dummy(),
                        obj_type,
                        name: ident_name.clone(),
                        root: None,
                        def: ident_def,
                        witness: node::get_witness_from_object(&obj),
                        parent_block: self.current_block,
                    };
                    let new_var2_id = self.add_variable(new_var2, None);
                    self.get_block_mut(self.current_block)
                        .unwrap()
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
                let rhs = self.get_object(rhs_id).unwrap();
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
    ) -> arena::Index {
        let obj = env.get(&var_name);
        let obj_type = node::ObjectType::get_type_from_object(&obj);
        let new_var = node::Variable {
            id: self.dummy(),
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
    ) -> Result<arena::Index, RuntimeError> {
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
        let ident_def = self.context().def_interner.ident_def(&priv_stmt.identifier);
        let new_var = node::Variable {
            id: self.dummy(),
            obj_type: node::ObjectType::NativeField, //TODO
            name: variable_name,
            root: None,
            def: ident_def,
            witness: None, //TODO
            parent_block: self.current_block,
        };
        let new_var_id = self.add_variable(new_var, None);
        // Create assign instruction
        let rhs_id = self.expression_to_object(env, &priv_stmt.expression)?;
        let rhs = self.get_object(rhs_id).unwrap();
        let r_type = rhs.get_type();
        let result = self.new_instruction(new_var_id, rhs_id, node::Operation::Ass, r_type);
        //self.update_variable_id(lhs_id, new_var_id); //update the name and the value array
        Ok(result)
    }

    // Let statements are used to declare higher level objects
    fn handle_let_statement(
        &mut self,
        env: &mut Environment,
        let_stmt: HirLetStatement,
    ) -> Result<arena::Index, RuntimeError> {
        //create a variable from the left side of the statement, evaluate the right and generate an assign instruction.

        // Extract the expression
        let rhs_id = self.expression_to_object(env, &let_stmt.expression)?;
        //TODO: is there always an expression? if not, how can we get the type of the variable?
        let rhs = self.get_object(rhs_id).unwrap();
        let rtype = rhs.get_type();

        // Convert the LHS into an identifier
        let variable_name = self.context().def_interner.ident_name(&let_stmt.identifier);
        let ident_def = self.context().def_interner.ident_def(&let_stmt.identifier);
        //Create a new variable;
        //TODO in the name already exists, we should use something else (from env) to find a variable (identid?)

        let new_var = node::Variable {
            id: self.dummy(),
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
    ) -> Result<arena::Index, RuntimeError> {
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
            HirExpression::Constructor(_) | HirExpression::MemberAccess(_) => todo!(),
            HirExpression::Error => todo!(),
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
        let cond =
            self.new_instruction(phi, end_idx, node::Operation::Ne, node::ObjectType::Boolean);
        let to_fix = self.new_instruction(
            cond,
            self.dummy(),
            node::Operation::Jeq,
            node::ObjectType::NotAnObject,
        );

        //Body
        let body_idx = block::new_sealed_block(self, block::BlockType::Normal);
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
        body_block1.update_variable(iter_id, phi); //TODO try with just a get_current_value(iter)
        let statements = block.statements();
        for stmt in statements {
            self.evaluate_statement(env, stmt).unwrap(); //TODO return the error
        }

        //increment iter
        let one = self.get_const(FieldElement::one(), iter_type);
        let incr = self.new_instruction(phi, one, node::Operation::Add, iter_type);
        let cur_block_id = self.current_block; //It should be the body block, except if the body has CFG statements
        let cur_block = self.get_block_mut(cur_block_id).unwrap();
        cur_block.update_variable(iter_id, incr);

        //body.left = join
        cur_block.left = Some(join_idx);
        let join_mut = self.get_block_mut(join_idx).unwrap();
        join_mut.predecessor.push(cur_block_id);
        //jump back to join
        self.new_instruction(
            self.dummy(),
            self.get_block(join_idx).get_first_instruction(),
            node::Operation::Jmp,
            node::ObjectType::NotAnObject,
        );
        //seal join
        ssa_form::seal_block(self, join_idx);

        //exit block
        self.current_block = exit_id;
        let exit_first = self.get_current_block().get_first_instruction();
        block::link_with_target(self, join_idx, Some(exit_id), Some(body_idx));
        let to_fix_ins = self.try_get_mut_instruction(to_fix);
        to_fix_ins.unwrap().rhs = body_idx;

        Ok(exit_first) //TODO what should we return???
    }

    pub fn acir(&self, evaluator: &mut Evaluator) {
        let mut acir = Acir::new();
        let mut fb = self.try_get_block(self.first_block);
        while fb.is_some() {
            for iter in &fb.unwrap().instructions {
                let ins = self.get_instruction(*iter);
                acir.evaluate_instruction(ins, evaluator, self);
            }
            //TODO we should rather follow the jumps
            if fb.unwrap().left.is_some() {
                fb = self.try_get_block(fb.unwrap().left.unwrap());
            } else {
                fb = None;
            }
        }
        //   dbg!(acir.arith_cache);
    }

    pub fn generate_empty_phi(
        &mut self,
        target_block: arena::Index,
        root: arena::Index,
    ) -> arena::Index {
        //Ensure there is not already a phi for the variable (n.b. probably not usefull)
        let block = self.get_block(target_block);
        for i in &block.instructions {
            if let Some(ins) = self.try_get_instruction(*i) {
                if ins.operator == node::Operation::Phi && ins.rhs == root {
                    return *i;
                }
            }
        }
        let v_type = self.get_object_type(root);
        let new_phi =
            node::Instruction::new(node::Operation::Phi, root, root, v_type, Some(target_block));
        let phi_id = self.nodes.insert(node::NodeObj::Instr(new_phi));
        //ria
        let mut phi_ins = self.try_get_mut_instruction(phi_id).unwrap();
        phi_ins.idx = phi_id;
        let block = self.get_block_mut(target_block).unwrap();
        block.instructions.insert(1, phi_id);
        phi_id
    }
}
