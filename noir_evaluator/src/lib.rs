mod binary_op;

mod environment;
mod low_level_function_impl;
mod builtin;
mod object;

mod errors;
pub use errors::{RuntimeErrorKind, RuntimeError};

use std::collections::{BTreeMap, HashMap};

use object::{Array, Integer, Object, Selector, RangedObject};
use environment::Environment;
use acir::optimiser::CSatOptimiser;

use acir::native_types::{Witness, Arithmetic, Linear};
use acir::circuit::gate::{AndGate, Gate, XorGate};
use acir::circuit::Circuit;

// XXX: Remove this once, we have moved to HIR
use noirc_frontend::{ast::*, hir::lower::{HirBinaryOp, HirCallExpression, HirForExpression, node_interner::IdentId, stmt::{HirBlockStatement, HirConstrainStatement, HirLetStatement}}}; 

use noirc_frontend::{hir::{crate_graph::{CrateType, LOCAL_CRATE}, lower::{HirExpression, HirLiteral, node_interner::{ExprId, FuncId, StmtId}, stmt::{HirPrivateStatement, HirStatement}}}};
use noirc_frontend::hir::Context;

use noir_field::FieldElement;
pub struct Evaluator<'a> {
    file_id : usize,
    num_witness: usize,                           // XXX: Can possibly remove
    num_selectors: usize,                         // XXX: Can possibly remove
    pub(crate) witnesses: HashMap<Witness, Type>, //XXX: Move into symbol table/environment -- Check if typing is needed here
    selectors: Vec<Selector>, // XXX: Possibly move into environment
    context: &'a Context,
    num_public_inputs: usize,
    main_function: FuncId,
    gates: Vec<Gate>, // Identifier, Object
    counter: usize,   // This is so that we can get a unique number
}

impl<'a> Evaluator<'a> {

    // XXX: Change from Option to Result
    pub fn new(file_id : usize, main_function : FuncId, context : &Context) -> Evaluator {
        
        Evaluator {
            file_id,
            num_witness: 0,
            num_selectors: 0,
            num_public_inputs: 0,
            witnesses: HashMap::new(),
            selectors: Vec::new(),
            context,
            main_function,
            gates: Vec::new(),
            counter: 0,
        }
    }

    // Returns the current counter value and then increments the counter
    // This is so that we can have unique variable names when the same function is called multiple times
    fn get_unique_value(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }

    // Takes a String which will be the variables name and adds it to the list of known Witnesses
    fn add_witness_to_cs(&mut self, variable_name: String, typ: Type) -> Witness {
        self.num_witness = self.num_witness + 1;
        let witness = Witness(variable_name, self.num_witness);
        self.witnesses.insert(witness.clone(), typ);
        witness
    }

    fn add_witness_to_env(&mut self, witness: Witness, env: &mut Environment) -> Object {
        let value = Object::from_witness(witness.clone());
        env.store(witness.0, value.clone());
        value
    }

    fn make_unique(&mut self, string: &str) -> String {
        format!("{}{}", string, self.get_unique_value())
    }

    pub fn num_witnesses(&self) -> usize {
        self.num_witness
    }

    /// Compiles the Program into ACIR and applies optimisations to the arithmetic gates
    // XXX: We return the num_witnesses, but this is the max number of witnesses
    // Some of these could have been removed due to optimisations. We need this number because the
    // Standard format requires the number of witnesses. The max number is also fine.
    // If we had a composer object, we would not need it
    pub fn compile(mut self) -> Result<(Circuit, usize, usize), RuntimeError> {

        // create a new environment
        let mut env = Environment::new();

        // First evaluate the main function
        self.evaluate_main(&mut env).map_err(|err| err.into_err(self.file_id))?;

        // Then optimise for a width3 plonk program
        // XXX: We can move all of this stuff into a plonk-backend program
        // which takes the IR as input
        const WIdTH: usize = 3;

        let optimiser = CSatOptimiser::new(WIdTH);

        let mut intermediate_variables: BTreeMap<Witness, Arithmetic> = BTreeMap::new();

        // Optimise the arithmetic gates by reducing them into the correct width and creating intermediate variables when necessary
        let num_witness = self.num_witnesses() + 1;
        let mut optimised_arith_gates: Vec<_> = self
            .gates
            .into_iter()
            .map(|gate| match gate {
                Gate::Arithmetic(arith) => {
                    let arith = optimiser.optimise(arith, &mut intermediate_variables, num_witness);
                    Gate::Arithmetic(arith)
                }
                other_gates => other_gates,
            })
            .collect();

        // The optimiser could have created intermediate variables/witnesses. We need to add these to the circuit
        for (witness, gate) in intermediate_variables {
            // Add intermediate variables as witnesses
            self.num_witness += 1;
            self.witnesses.insert(witness, Type::Witness);
            // Add gate into the circuit
            optimised_arith_gates.push(Gate::Arithmetic(gate));

            // XXX: We can additionally check that these arithmetic gates are done correctly via our optimiser -- It should have no effect if passed in twice
        }

        // Print all gates for debug purposes
        // for gate in optimised_arith_gates.iter() {
        //     // dbg!(gate);
        // }

        // for (i, witness) in self.witnesses.iter().enumerate() {
        //     // dbg!(i, witness);
        // }

        Ok((
            Circuit(optimised_arith_gates),
            self.witnesses.len(),
            self.num_public_inputs,
        ))
    }

    // When we are multiplying arithmetic gates by each other, if one gate has too many terms
    // It is better to create an intermediate variable which links to the gate and then multiply by that intermediate variable
    // instead
    pub fn create_intermediate_variable(
        &mut self,
        env: &mut Environment,
        arithmetic_gate: Arithmetic,
        typ: Type,
    ) -> (Object, Witness) {
        
        // Create a unique witness name and add witness to the constraint system        
        let inter_var_unique_name = self.make_unique("_inter");
        let inter_var_witness = self.add_witness_to_cs(inter_var_unique_name, Type::Witness);
        let inter_var_object = self.add_witness_to_env(inter_var_witness.clone(), env);

        // Link that witness to the arithmetic gate
        let constraint = &arithmetic_gate - &inter_var_witness;
        self.gates.push(Gate::Arithmetic(constraint));
        (inter_var_object, inter_var_witness)
    }

    pub fn evaluate_infix_expression(
        &mut self,
        env: &mut Environment,
        lhs: Object,
        rhs: Object,
        op: HirBinaryOp,
    ) -> Result<Object, RuntimeErrorKind> {
        match op {
            HirBinaryOp::Add => binary_op::handle_add_op(lhs, rhs, env, self),
            HirBinaryOp::Subtract => binary_op::handle_sub_op(lhs, rhs, env, self),
            HirBinaryOp::Multiply => binary_op::handle_mul_op(lhs, rhs, env, self),
            HirBinaryOp::Divide => binary_op::handle_div_op(lhs, rhs, env, self),
            HirBinaryOp::NotEqual => binary_op::handle_neq_op(lhs, rhs, env, self),
            HirBinaryOp::Equal => binary_op::handle_equal_op(lhs, rhs, env, self),
            HirBinaryOp::And => binary_op::handle_and_op(lhs, rhs, env, self),
            HirBinaryOp::Xor => binary_op::handle_xor_op(lhs, rhs, env, self),
            HirBinaryOp::Less => binary_op::handle_less_than_op(lhs, rhs, env, self),
            HirBinaryOp::LessEqual => binary_op::handle_less_than_equal_op(lhs, rhs, env, self),
            HirBinaryOp::Greater => binary_op::handle_greater_than_op(lhs, rhs, env, self),
            HirBinaryOp::GreaterEqual => binary_op::handle_greater_than_equal_op(lhs, rhs, env, self),
            HirBinaryOp::Assign => unreachable!("The Binary operation `=` can only be used in declaration statements"),
            HirBinaryOp::Or => todo!("The Or operation is currently not implemented. Coming soon.")
        }
    }

    // When we evaluate an identifier , it will be a linear polynomial
    // This is because, we currently do not have support for optimisations with polynomials of higher degree or higher fan-ins
    // XXX: One way to configure this in the future, is to count the fan-in/out and check if it is lower than the configured width
    // Either it is 1 * x + 0 or it is ax+b
    fn evaluate_identifier(&mut self, ident_id: IdentId, env: &mut Environment) -> Object {
        
        let ident_name = self.context.def_interner.id_name(ident_id);
        let polynomial = env.get(&ident_name);

        polynomial
    }


    /// Compiles the AST into the intermediate format by evaluating the main function
    pub fn evaluate_main(&mut self, env: &mut Environment) -> Result<(), RuntimeErrorKind>{
        // Add the parameters from the main function into the evaluator as witness variables
        // XXX: We are only going to care about Public and Private witnesses for now

        let mut pub_inputs = Vec::new();
        let mut witnesses = Vec::new();

        let func_meta = self.context.def_interner.function_meta(self.main_function);
        for param in func_meta.parameters {
            let param_id = param.0;
            let param_type = param.1;
            let param_name = self.context.def_interner.id_name(param_id);
            
            match param_type {
                Type::Public =>{
                    pub_inputs.push(param_name);
                },
                Type::Witness => {
                    witnesses.push(param_name)
                },
                _=> todo!("Currently we only have support for Private and Public inputs in the main function definition. It has not been decided as to whether we should allow other types")
            }
        }

        self.num_public_inputs = pub_inputs.len();

        // Add all of the public inputs first, then the witnesses
        for param_name in pub_inputs.into_iter() {
            let witness = self.add_witness_to_cs(param_name, Type::Public);
            self.add_witness_to_env(witness, env);
        }
        
        for param_name in witnesses.into_iter() {
            let witness = self.add_witness_to_cs(param_name, Type::Witness);
            self.add_witness_to_env(witness, env);
        }

        // Now call the main function
        // XXX: We should be able to replace this with call_function in the future, 
        // It is not possible now due to the aztec standard format requiring a particular ordering of inputs in the ABI
        let main_func_body = self.context.def_interner.function(self.main_function);
        for stmt_id in main_func_body.statements() {
            self.evaluate_statement(env, stmt_id)?;
        }
        Ok(())
    }
    
    fn evaluate_statement(&mut self, env: &mut Environment, stmt_id : StmtId) -> Result<Object, RuntimeErrorKind> {
        let statement = self.context.def_interner.statement(stmt_id);
        match statement {
            HirStatement::Private(x) => self.handle_private_statement(env, x),
            HirStatement::Constrain(constrain_stmt) => self.handle_constrain_statement(env, constrain_stmt),
            // constant statements do not create constraints
            HirStatement::Const(x) => {
                self.num_selectors = self.num_selectors + 1;


                let variable_name: String = self.context.def_interner.id_name(x.identifier);
                let value = self.evaluate_integer(env, x.expression)?; // const can only be integers/Field elements, cannot involve the witness, so we can eval at compile
                self.selectors
                    .push(Selector(variable_name.clone(), value.clone()));
                env.store(variable_name, value);
                Ok(Object::Null)
            }
            HirStatement::Expression(expr) => self.expression_to_object(env, expr),
            HirStatement::Let(let_stmt) => {
                // let statements are used to declare a higher level object
                self.handle_let_statement(env, let_stmt)?;

                Ok(Object::Null)
            }
            HirStatement::Public(_) => todo!("This may be deprecated. We do however want a way to keep track of linear transformations between private variable and public/constants"),
            HirStatement::Block(_) => todo!("This may be deprecated for block expressions")
        }
    }

    // The LHS of a private statement is always a new witness
    // Cannot do `private x + k = z`
    // It is also a new variable, since private is used to derive variables
    fn handle_private_statement(
        &mut self,
        env: &mut Environment,
        x: HirPrivateStatement,
    ) -> Result<Object, RuntimeErrorKind> {
        let variable_name = self.context.def_interner.id_name(x.identifier);
        let witness = self.add_witness_to_cs(variable_name.clone(), x.r#type.clone()); // XXX: We do not store it in the environment yet, because it may need to be casted to an integer
        
        let rhs_poly = self.expression_to_object(env, x.expression)?;


        // There are two ways to add the variable to the environment. We can add the variable and link it to itself,
        // This is fine since we constrain the RHS to be equal to the LHS.
        // The other way is to check if the RHS is a linear polynomial, and link the variable to the RHS instead
        // This second way is preferred because it allows for more optimisation options.
        // If the RHS is not a linear polynomial, then we do the first option. 
        if rhs_poly.can_defer_constraint() {
            env.store(variable_name, rhs_poly.clone());
        } else {
            self.add_witness_to_env(witness.clone(), env);
        }

        // This is a private statement, which is why we extract only a witness type from the object
        let rhs_as_witness = rhs_poly.extract_private_witness().ok_or(RuntimeErrorKind::UnstructuredError{span : Default::default(), message : format!("only witnesses can be used in a private statement")})?; 
        self.gates.push(Gate::Arithmetic(&rhs_as_witness - &witness));
        
        // Lets go through some possible scenarios to explain why the code is correct
        // 0: priv x = 5;
        //
        // This is not possible since the RHS is not a Witness type. It is constant. 
        //
        // 1: priv x = y + z;
        //
        // Here we apply one gate `y + z - x = 0`
        //
        // 2: priv x : u8 = y + z as u32;
        // 
        // This is not allowed because the lhs says u8 and the rhs says u32
        // 
        // 3: priv x : u32 = y + z as u32
        // 
        // Since the lhs type is the same as the rhs, it will pass analysis.
        // When we constrain the rhs `y + z as u32` we are sure that the RHS is a u32 or it will fail
        // When we then add the constraint that x - y + z = 0
        // We know that x must be a u32 aswell, since the constraint enforces them to be equal
        //
        // TLDR; This works because the RHS is already constrained when we receive it as an object
        // Even if we remove the typing information, the constraint has already been applied, so it is correct.
        // Elaborating a little more. An integer x is a witness which has been constrained to be y num_bits. If we simply remove the type information
        // ie just take x, then apply the constraint z - x' = 0. Then x' is implicitly constrained to be y num bits also.
        Ok(Object::Null)
    }

    // The LHS of a private statement is always a new witness
    // Cannot do `private x + k = z`
    // It is also a new variable, since private is used to derive variables
    fn handle_constrain_statement(
        &mut self,
        env: &mut Environment,
        constrain_stmt: HirConstrainStatement,
    ) -> Result<Object, RuntimeErrorKind> {
        let lhs_poly = self.expression_to_object(env, constrain_stmt.0.lhs)?;
        let rhs_poly = self.expression_to_object(env, constrain_stmt.0.rhs)?;

        // Evaluate the constrain infix statement
        let _ = self.evaluate_infix_expression(
            env,
            lhs_poly.clone(),
            rhs_poly.clone(),
            constrain_stmt.0.operator,
        );

        // XXX: We could probably move this into equal folder, as it is an optimisation that only applies to it
        if constrain_stmt.0.operator == HirBinaryOp::Equal {
            // Check if we have any lone variables and then if the other side is a linear/constant
            let (witness, rhs) = match (lhs_poly.is_unit_witness(), rhs_poly.is_unit_witness()) {
                (true, _) => (lhs_poly.witness(), rhs_poly),
                (_, true) => (rhs_poly.witness(), lhs_poly),
                (_, _) => (None, Object::Null),
            };

            match witness {
                Some(wit) => {
                    // Check if the RHS is linear or constant
                    if rhs.is_linear() | rhs.is_constant() {
                        env.store(wit.0, rhs)
                    }
                }
                None => {}
            };
        };
        Ok(Object::Null)
    }
    // Let statements are used to declare higher level objects
    fn handle_let_statement(
        &mut self,
        env: &mut Environment,
        let_stmt: HirLetStatement,
    ) -> Result<Object, RuntimeErrorKind> {
        // Convert the LHS into an identifier
        let variable_name = self.context.def_interner.id_name(let_stmt.identifier);

        // XXX: Currently we only support arrays using this, when other types are introduced
        // we can extend into a separate (generic) module

        // Extract the array
        let rhs_poly = self.expression_to_object(env, let_stmt.expression)?;

        match rhs_poly {
            Object::Array(arr) => {
                env.store(variable_name.into(), Object::Array(arr));
            }
            _ => unimplemented!("logic for types that are not arrays in a let statement, not implemented yet!"),
        };

        Ok(Object::Null)
    }
    fn handle_for_expr(
        &mut self,
        env: &mut Environment,
        for_expr: HirForExpression,
    ) -> Result<Object, RuntimeErrorKind> {
        
        // First create an iterator over all of the for loop identifiers
        // XXX: We preferably need to introduce public integers and private integers, so that we can 
        // loop securely on constants. This requires `constant as u128`, analysis will take care of the rest 
        let start = self.expression_to_object(env, for_expr.start_range)?.constant()?;
        let end = self.expression_to_object(env, for_expr.end_range)?.constant()?;
        let ranged_object = RangedObject::new(start, end)?;
        
        let mut contents : Vec<Object> = Vec::new();

        for indice in ranged_object {
            env.start_for_loop();

            // Add indice to environment
            let variable_name = self.context.def_interner.id_name(for_expr.identifier);
            env.store(variable_name, Object::Constants(indice));

            let statements = self.statement_to_block(for_expr.block).statements();
            let return_typ = self.eval_block(env, statements)?;
            contents.push(return_typ);

            env.end_for_loop();
        }
        let length = contents.len() as u128;

        Ok(Object::Array(Array{contents, length}))
    }

    fn statement_to_block(&mut self, stmt_id : StmtId) -> HirBlockStatement {
        let stmt = self.context.def_interner.statement(stmt_id);
        match stmt {
            HirStatement::Block(block_stmt) => block_stmt,
            _=> panic!("ice: expected a block statement")
        }
    }

    pub fn evaluate_integer(&mut self, env: &mut Environment, expr_id: ExprId) -> Result<Object, RuntimeErrorKind> {
        let polynomial = self.expression_to_object(env, expr_id)?;

        if polynomial.is_constant() {
            return Ok(polynomial)
        }
        return Err(RuntimeErrorKind::expected_type("constant",polynomial.r#type()));
    }

    pub fn expression_to_object(
        &mut self,
        env: &mut Environment,
        expr_id: ExprId,
    ) -> Result<Object, RuntimeErrorKind> {
        let expr = self.context.def_interner.expression(expr_id);
        match expr {
            HirExpression::Literal(HirLiteral::Integer(x)) => Ok(Object::Constants(x.into())),
            HirExpression::Literal(HirLiteral::Array(arr_lit)) => {
                Ok(Object::Array(Array::from(self, env, arr_lit)?))
            }
            HirExpression::Ident(x) => Ok(self.evaluate_identifier(x, env)),
            HirExpression::Infix(infx) => {
                let lhs = self.expression_to_object(env, infx.lhs)?;
                let rhs = self.expression_to_object(env, infx.rhs)?;
                self.evaluate_infix_expression(env, lhs, rhs, infx.operator)
            }
            HirExpression::Cast(cast_expr) => {
                let lhs = self.expression_to_object(env, cast_expr.lhs)?;
                binary_op::handle_cast_op(lhs, cast_expr.r#type, env, self)
            }
            HirExpression::Index(indexed_expr) => {
                // Currently these only happen for arrays
                let arr_name = self.context.def_interner.id_name(indexed_expr.collection_name);
                let arr = env.get_array(&arr_name)?;
        
                // Evaluate the index expression
                let index_as_obj = self.expression_to_object(env, indexed_expr.index)?;
                let index_as_constant = match index_as_obj.constant() {
                    Ok(v) => v,
                    Err(_) => panic!("Indexed expression does not evaluate to a constant")
                };
                let index_as_u128 = index_as_constant.to_u128();
                
                arr.get(index_as_u128)
            }
            HirExpression::Call(call_expr) => {

                let func_meta = self.context.def_interner.function_meta(call_expr.func_id);
                        
                // Choices are a low level func or an imported library function
                // If low level, then we use it's func name to find out what function to call
                // If not then we just call the library as usual with the function definition
                match func_meta.kind {
                    FunctionKind::Normal => self.call_function(env, &call_expr, call_expr.func_id),
                    FunctionKind::LowLevel => {
                        let attribute = func_meta.attributes.expect("all low level functions must contain an attribute which contains the opcode which it links to");
                        let opcode_name = attribute.foreign().expect("ice: function marked as foreign, but attribute kind does not match this");
                        low_level_function_impl::call_low_level(self, env, opcode_name, call_expr)
                    },
                    FunctionKind::Builtin => {
                        let attribute = func_meta.attributes.expect("all builtin functions must contain an attribute which contains the function name which it links to");
                        let builtin_name = attribute.builtin().expect("ice: function marked as a builtin, but attribute kind does not match this");
                        builtin::call_builtin(self, env, builtin_name, call_expr)
                    },
                }
                    
            }
            HirExpression::For(for_expr) => {
                self.handle_for_expr(env,for_expr)
            }
            HirExpression::If(_) => todo!(),
            HirExpression::Prefix(_) => todo!(),
            HirExpression::Predicate(_) => todo!(),
            HirExpression::Literal(_) => todo!()
        }
    }

    fn call_function(&mut self, env: &mut Environment, call_expr : &HirCallExpression, func_id: FuncId) -> Result<Object, RuntimeErrorKind> {
              // Create a new environment for this function
                // This is okay because functions are not stored in the environment
                // We need to add the arguments into the environment though
                // Note: The arguments are evaluated in the old environment
                let mut new_env = Environment::new();
                let (arguments, mut errors) = self.expression_list_to_objects(env, &call_expr.arguments);
                if !errors.is_empty() {
                    // XXX: We could have an error variant to return multiple errors here
                    // As long as we can guarantee that each expression does not affect the proceeding, this should be fine
                    return Err(errors.pop().unwrap())
                }

                let func_meta = self.context.def_interner.function_meta(func_id);

                for (param, argument) in func_meta.parameters.iter().zip(arguments.into_iter())
                {
                    let param_id = param.0;
                    let param_name = self.context.def_interner.id_name(param_id);
                    
                    new_env.store(param_name, argument);
                }

                let return_val = self.apply_func(&mut new_env, func_id)?;

                Ok(return_val)
    }

    fn apply_func(&mut self, env: &mut Environment, func_id: FuncId) -> Result<Object, RuntimeErrorKind> {
        
        let function = self.context.def_interner.function(func_id);
        self.eval_block(env, function.statements())
    }

    fn eval_block(&mut self, env: &mut Environment, block: Vec<StmtId>) -> Result<Object, RuntimeErrorKind> {
        
        let mut result = Object::Null;
        for stmt in block {
            result = self.evaluate_statement(env, stmt)?;
        }
        Ok(result)
    }

    fn expression_list_to_objects(&mut self, env : &mut Environment, exprs : &[ExprId]) -> (Vec<Object>, Vec<RuntimeErrorKind>) {
        let (objects, errors) : (Vec<_>, Vec<_>) = exprs.iter()
        .map(|expr| self.expression_to_object(env, expr.clone()))
        .partition(Result::is_ok);

        let objects: Vec<_> = objects.into_iter().map(Result::unwrap).collect();
        let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
        (objects, errors)
    }
}