mod binary_op;

mod builtin;
mod environment;
mod low_level_function_impl;
mod object;

mod errors;
use acvm::BackendPointer;
use blake2::Blake2s;
use errors::{RuntimeError, RuntimeErrorKind};

use std::collections::HashMap;

use environment::Environment;
use object::{Array, Integer, Object, RangedObject};

use acvm::acir::circuit::Circuit;
use acvm::acir::circuit::{
    gate::{AndGate, Gate, XorGate},
    PublicInputs,
};
use acvm::acir::native_types::{Arithmetic, Linear, Witness};

use noirc_frontend::node_interner::{IdentId,ExprId, FuncId, StmtId};
use noirc_frontend::hir::lower::{
    stmt::{HirConstrainStatement, HirLetStatement},
    HirBinaryOp, HirCallExpression, HirForExpression,
};
use noirc_frontend::{
    hir::lower::{HirBinaryOpKind, HirBlockExpression},
    FunctionKind, Signedness, Type,
};

use noirc_frontend::hir::lower::{
    stmt::{HirPrivateStatement, HirStatement},
    HirExpression, HirLiteral,
};
use noirc_frontend::hir::Context;

use noir_field::FieldElement;
pub struct Evaluator<'a> {
    file_id: usize,
    // XXX: This is doing two things. We should split it up so that Witnesses
    // are local to their execution context and not the global context.
    // Also there should be a separate map to map variable names to witness indices.
    // This will allow us to remove the `String` component from Witness in ACIR 
    //
    // XXX: The Type is here from the lang4 refactor, where we were going to allow noir to optimise public inputs
    // We may be able to remove, Kobi and Ariel mentioned that this was too restrictive.
    pub(crate) witnesses: HashMap<Witness, Type>,
    context: &'a Context,
    public_inputs: Vec<Witness>,
    main_function: FuncId,
    gates: Vec<Gate>,
    counter: usize,   // This is so that we can get a unique number
}

impl<'a> Evaluator<'a> {
    pub fn new(file_id: usize, main_function: FuncId, context: &Context) -> Evaluator {
        Evaluator {
            file_id,
            public_inputs: Vec::new(),
            witnesses: HashMap::new(),
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

    // Creates a new Witness index
    fn add_witness_to_cs(&mut self) -> Witness {
        let witness = Witness(self.num_witnesses() + 1);
        self.witnesses.insert(witness.clone(), Type::Witness);
        witness
    }

    // Maps a variable name to a witness index
    fn add_witness_to_env(&mut self, variable_name : String, witness: Witness, env: &mut Environment) -> Object {
        let value = Object::from_witness(witness.clone());
        env.store(variable_name, value.clone());
        value
    }

    fn make_unique(&mut self, string: &str) -> String {
        format!("{}{}", string, self.get_unique_value())
    }

    pub fn num_witnesses(&self) -> u32 {
        self.witnesses.len() as u32
    }

    /// Compiles the Program into ACIR and applies optimisations to the arithmetic gates
    // XXX: We return the num_witnesses, but this is the max number of witnesses
    // Some of these could have been removed due to optimisations. We need this number because the
    // Standard format requires the number of witnesses. The max number is also fine.
    // If we had a composer object, we would not need it
    pub fn compile(mut self, backend: BackendPointer) -> Result<Circuit, RuntimeError> {
        // create a new environment
        let mut env = Environment::new();

        // First evaluate the main function
        self.evaluate_main(&mut env)
            .map_err(|err| err.into_err(self.file_id))?;

        let num_witness = self.num_witnesses();
        let optimised_circuit = acvm::compiler::compile(
            Circuit {
                num_witnesses: num_witness as u32,
                gates: self.gates,
                public_inputs: PublicInputs(self.public_inputs),
            },
            backend,
        );

        Ok(optimised_circuit)
    }

    // When we are multiplying arithmetic gates by each other, if one gate has too many terms
    // It is better to create an intermediate variable which links to the gate and then multiply by that intermediate variable
    // instead
    //
    // XXX: Check logic for when we create intermediate variables 
    // for Integer objects, the type passed in, is not witness
    pub fn create_intermediate_variable(
        &mut self,
        env: &mut Environment,
        arithmetic_gate: Arithmetic,
        typ: Type,
    ) -> (Object, Witness) {
        // Create a unique witness name and add witness to the constraint system
        let inter_var_unique_name = self.make_unique("_inter");
        let inter_var_witness = self.add_witness_to_cs();
        let inter_var_object = self.add_witness_to_env(inter_var_unique_name,inter_var_witness.clone(), env);

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
        match op.kind {
            HirBinaryOpKind::Add => binary_op::handle_add_op(lhs, rhs, env, self),
            HirBinaryOpKind::Subtract => binary_op::handle_sub_op(lhs, rhs, env, self),
            HirBinaryOpKind::Multiply => binary_op::handle_mul_op(lhs, rhs, env, self),
            HirBinaryOpKind::Divide => binary_op::handle_div_op(lhs, rhs, env, self),
            HirBinaryOpKind::NotEqual => binary_op::handle_neq_op(lhs, rhs, env, self),
            HirBinaryOpKind::Equal => binary_op::handle_equal_op(lhs, rhs, env, self),
            HirBinaryOpKind::And => binary_op::handle_and_op(lhs, rhs, env, self),
            HirBinaryOpKind::Xor => binary_op::handle_xor_op(lhs, rhs, env, self),
            HirBinaryOpKind::Less => binary_op::handle_less_than_op(lhs, rhs, env, self),
            HirBinaryOpKind::LessEqual => binary_op::handle_less_than_equal_op(lhs, rhs, env, self),
            HirBinaryOpKind::Greater => binary_op::handle_greater_than_op(lhs, rhs, env, self),
            HirBinaryOpKind::GreaterEqual => {
                binary_op::handle_greater_than_equal_op(lhs, rhs, env, self)
            }
            HirBinaryOpKind::Assign => {
                let err = RuntimeErrorKind::Spanless(
                    "The Binary operation `=` can only be used in declaration statements"
                        .to_string(),
                );
                return Err(err);
            }
            HirBinaryOpKind::Or => {
                let err = RuntimeErrorKind::Unimplemented("The Or operation is currently not implemented. First implement in Barretenberg.".to_owned());
                return Err(err);
            }
        }
    }

    // When we evaluate an identifier , it will be a linear polynomial
    // This is because, we currently do not have support for optimisations with polynomials of higher degree or higher fan-ins
    // XXX: One way to configure this in the future, is to count the fan-in/out and check if it is lower than the configured width
    // Either it is 1 * x + 0 or it is ax+b
    fn evaluate_identifier(&mut self, ident_id: &IdentId, env: &mut Environment) -> Object {
        let ident_name = self.context.def_interner.ident_name(ident_id);
        env.get(&ident_name)
    }

    /// Compiles the AST into the intermediate format by evaluating the main function
    pub fn evaluate_main(&mut self, env: &mut Environment) -> Result<(), RuntimeErrorKind> {
  
        self.parse_abi(env)?;

        // Now call the main function
        // XXX: We should be able to replace this with call_function in the future,
        // It is not possible now due to the aztec standard format requiring a particular ordering of inputs in the ABI
        let main_func_body = self.context.def_interner.function(&self.main_function);
        let block = main_func_body.block(&self.context.def_interner);
        for stmt_id in block.statements() {
            self.evaluate_statement(env, stmt_id)?;
        }
        Ok(())
    }

    /// The ABI is the intermediate representation between Noir and types like Toml
    /// Noted in the noirc_abi, it is possible to convert Toml -> NoirTypes
    /// However, this intermediate representation is useful as it allows us to have 
    /// intermediate Types which the core type system does not know about like Strings. 
    fn parse_abi(&mut self, env : &mut Environment) -> Result<(), RuntimeErrorKind>{
        // XXX: Currently, the syntax only supports public witnesses
        // u8 and arrays are assumed to be private
        // This is not a short-coming of the ABI, but of the grammar
        // The new grammar has been conceived, adn will be implemented.

        let func_meta = self.context.def_interner.function_meta(&self.main_function);

        let abi = func_meta.parameters.to_abi(&self.context.def_interner);

        for (param_name, param_type) in abi.parameters.into_iter() {
            match param_type {
                noirc_abi::AbiType::Array { length, typ } => {
                    let mut elements = Vec::with_capacity(length as usize);
                    for i in 0..length as usize {
                        let mangled_name = mangle_array_element_name(&param_name, i);
                        let witness = self.add_witness_to_cs();

                        // Constrain each element in the array to be equal to the type declared in the parameter
                        let object = match *typ {
                            noirc_abi::AbiType::Integer { sign, width } => {
                                // Currently we do not support signed integers
                                assert!(
                                    sign != noirc_abi::Sign::Signed,
                                    "signed integers are currently not supported"
                                );

                                let integer = Integer::from_witness(witness, width);
                                integer.constrain(self)?;
                                Object::Integer(integer)
                            }
                            noirc_abi::AbiType::Private => self.add_witness_to_env(mangled_name,witness, env),
                            _ => unimplemented!(
                                "currently we only support arrays of integer and witness types"
                            ),
                        };

                        elements.push(object);
                    }
                    let arr = Array {
                        contents: elements,
                        length,
                    };
                    env.store(param_name, Object::Array(arr));
                }
                noirc_abi::AbiType::Private => {
                    let witness = self.add_witness_to_cs();
                    self.add_witness_to_env(param_name, witness, env);
                }
                noirc_abi::AbiType::Integer { sign, width } => {
                    let witness = self.add_witness_to_cs();

                    // Currently we do not support signed integers
                    assert!(
                        sign != noirc_abi::Sign::Signed,
                        "signed integers are currently not supported"
                    );

                    let integer = Integer::from_witness(witness, width);
                    integer.constrain(self)?;

                    env.store(param_name, Object::Integer(integer));
                }
                noirc_abi::AbiType::Public => {
                    let witness = self.add_witness_to_cs();
                    self.public_inputs.push(witness.clone());
                    self.add_witness_to_env(param_name,witness, env);
                }
            }
        }

        Ok(())
    }

    fn evaluate_statement(
        &mut self,
        env: &mut Environment,
        stmt_id: &StmtId,
    ) -> Result<Object, RuntimeErrorKind> {
        let statement = self.context.def_interner.statement(stmt_id);
        match statement {
            HirStatement::Private(x) => self.handle_private_statement(env, x),
            HirStatement::Constrain(constrain_stmt) => self.handle_constrain_statement(env, constrain_stmt),
            HirStatement::Const(x) => {

                let variable_name: String = self.context.def_interner.ident_name(&x.identifier);
                // const can only be integers/Field elements, cannot involve the witness, so we can possibly move this to
                // analysis. Right now it would not make a difference, since we are not compiling to an intermediate Noir format
                let value = self.evaluate_integer(env, &x.expression)?; 

                env.store(variable_name, value);
                Ok(Object::Null)
            }
            HirStatement::Expression(expr) | HirStatement::Semi(expr) => self.expression_to_object(env, &expr),
            HirStatement::Let(let_stmt) => {
                // let statements are used to declare a higher level object
                self.handle_let_statement(env, let_stmt)?;

                Ok(Object::Null)
            }
            HirStatement::Public(_) => todo!("This may be deprecated. We do however want a way to keep track of linear transformations between private variable and public/constants"),
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
        let rhs_poly = self.expression_to_object(env, &x.expression)?;

        let variable_name = self.context.def_interner.ident_name(&x.identifier);
        // XXX: We do not store it in the environment yet, because it may need to be casted to an integer
        let witness = self.add_witness_to_cs();

        // There are two ways to add the variable to the environment. We can add the variable and link it to itself,
        // This is fine since we constrain the RHS to be equal to the LHS.
        // The other way is to check if the RHS is a linear polynomial, and link the variable to the RHS instead
        // This second way is preferred because it allows for more optimisation options.
        // If the RHS is not a linear polynomial, then we do the first option.
        if rhs_poly.can_defer_constraint() {
            env.store(variable_name, rhs_poly.clone());
        } else {
            self.add_witness_to_env(variable_name,witness.clone(), env);
        }

        // This is a private statement, which is why we extract only a witness type from the object
        let rhs_as_witness =
            rhs_poly
                .extract_private_witness()
                .ok_or(RuntimeErrorKind::UnstructuredError {
                    span: Default::default(),
                    message: format!("only witnesses can be used in a private statement"),
                })?;
        self.gates
            .push(Gate::Arithmetic(&rhs_as_witness - &witness));

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

    // Add a constraint to constrain two expression together 
    fn handle_constrain_statement(
        &mut self,
        env: &mut Environment,
        constrain_stmt: HirConstrainStatement,
    ) -> Result<Object, RuntimeErrorKind> {
        let lhs_poly = self.expression_to_object(env, &constrain_stmt.0.lhs)?;
        let rhs_poly = self.expression_to_object(env, &constrain_stmt.0.rhs)?;

        // Evaluate the constrain infix statement
        let _ = self.evaluate_infix_expression(
            env,
            lhs_poly.clone(),
            rhs_poly.clone(),
            constrain_stmt.0.operator,
        )?;

        // The code below is an optimisation strategy for when either side is of the form
        //
        // constrain x == 4
        // constrain y == 4t + m
        //
        // In the above extracts, we can use interpret x as a constant and y as a constant.
        //
        // We should also check for unused witnesses and transform the circuit, so
        // that you do not need to compute them.
        //
        // XXX: We could probably move this into equal folder, as it is an optimisation that only applies to it
        // Moreover: This could be moved to ACVM.
        if constrain_stmt.0.operator.kind == HirBinaryOpKind::Equal {
            // Check if we have any lone variables and then if the other side is a linear/constant
            let (lhs_unit_witness, rhs) = match (lhs_poly.is_unit_witness(), rhs_poly.is_unit_witness()) {
                (true, _) => (lhs_poly.witness(), rhs_poly),
                (_, true) => (rhs_poly.witness(), lhs_poly),
                (_, _) => (None, Object::Null),
            };
            
            if let Some(unit_wit) = lhs_unit_witness {
                // Check if the RHS is linear or constant
                if rhs.is_linear() | rhs.is_constant() {
                    let var_name = env.find_with_value(&unit_wit).expect("ice: could not find corresponding variable name");
                    env.store(var_name, rhs)
                }
            }
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
        let variable_name = self.context.def_interner.ident_name(&let_stmt.identifier);

        // XXX: Currently we only support arrays using this, when other types are introduced
        // we can extend into a separate (generic) module

        // Extract the array
        let rhs_poly = self.expression_to_object(env, &let_stmt.expression)?;

        match rhs_poly {
            Object::Array(arr) => {
                env.store(variable_name.into(), Object::Array(arr));
            }
            _ => unimplemented!(
                "logic for types that are not arrays in a let statement, not implemented yet!"
            ),
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
        let start = self
            .expression_to_object(env, &for_expr.start_range)?
            .constant()?;
        let end = self
            .expression_to_object(env, &for_expr.end_range)?
            .constant()?;
        let ranged_object = RangedObject::new(start, end)?;

        let mut contents: Vec<Object> = Vec::new();

        for indice in ranged_object {
            env.start_for_loop();

            // Add indice to environment
            let variable_name = self.context.def_interner.ident_name(&for_expr.identifier);
            env.store(variable_name, Object::Constants(indice));

            let block = self.expression_to_block(&for_expr.block);
            let statements = block.statements();
            let return_typ = self.eval_block(env, statements)?;
            contents.push(return_typ);

            env.end_for_loop();
        }
        let length = contents.len() as u128;

        Ok(Object::Array(Array { contents, length }))
    }

    fn expression_to_block(&mut self, expr_id: &ExprId) -> HirBlockExpression {
        match self.context.def_interner.expression(expr_id) {
            HirExpression::Block(block_expr) => block_expr,
            _ => panic!("ice: expected a block expression"),
        }
    }

    pub fn evaluate_integer(
        &mut self,
        env: &mut Environment,
        expr_id: &ExprId,
    ) -> Result<Object, RuntimeErrorKind> {
        let polynomial = self.expression_to_object(env, expr_id)?;

        if polynomial.is_constant() {
            return Ok(polynomial);
        }
        return Err(RuntimeErrorKind::expected_type(
            "constant",
            polynomial.r#type(),
        ));
    }

    pub fn expression_to_object(
        &mut self,
        env: &mut Environment,
        expr_id: &ExprId,
    ) -> Result<Object, RuntimeErrorKind> {
        let expr = self.context.def_interner.expression(expr_id);
        match expr {
            HirExpression::Literal(HirLiteral::Integer(x)) => Ok(Object::Constants(x.into())),
            HirExpression::Literal(HirLiteral::Array(arr_lit)) => {
                Ok(Object::Array(Array::from(self, env, arr_lit)?))
            }
            HirExpression::Ident(x) => Ok(self.evaluate_identifier(&x, env)),
            HirExpression::Infix(infx) => {
                let lhs = self.expression_to_object(env, &infx.lhs)?;
                let rhs = self.expression_to_object(env, &infx.rhs)?;
                self.evaluate_infix_expression(env, lhs, rhs, infx.operator)
            }
            HirExpression::Cast(cast_expr) => {
                let lhs = self.expression_to_object(env, &cast_expr.lhs)?;
                binary_op::handle_cast_op(lhs, cast_expr.r#type, env, self)
            }
            HirExpression::Index(indexed_expr) => {
                // Currently these only happen for arrays
                let arr_name = self.context.def_interner.ident_name(&indexed_expr.collection_name);
                let ident_span = self.context.def_interner.ident_span(&indexed_expr.collection_name);
                let arr = env.get_array(&arr_name)?;
                //
                // Evaluate the index expression
                let index_as_obj = self.expression_to_object(env, &indexed_expr.index)?;
                let index_as_constant = match index_as_obj.constant() {
                    Ok(v) => v,
                    Err(_) => panic!("Indexed expression does not evaluate to a constant")
                };
                //
                let index_as_u128 = index_as_constant.to_u128();
                arr.get(index_as_u128, ident_span)
            }
            HirExpression::Call(call_expr) => {

                let func_meta = self.context.def_interner.function_meta(&call_expr.func_id);
                //
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
            HirExpression::Literal(_) => todo!(),
            HirExpression::Block(block) => todo!("currently block expressions not in for/if branches are not being evaluated. In the future, we should be able to unify the eval_block and all places which require block_expr here")
        }
    }

    fn call_function(
        &mut self,
        env: &mut Environment,
        call_expr: &HirCallExpression,
        func_id: FuncId,
    ) -> Result<Object, RuntimeErrorKind> {
        // Create a new environment for this function
        // This is okay because functions are not stored in the environment
        // We need to add the arguments into the environment though
        // Note: The arguments are evaluated in the old environment
        let mut new_env = Environment::new();
        let (arguments, mut errors) = self.expression_list_to_objects(env, &call_expr.arguments);
        if !errors.is_empty() {
            // XXX: We could have an error variant to return multiple errors here
            // As long as we can guarantee that each expression does not affect the proceeding, this should be fine
            return Err(errors.pop().unwrap());
        }

        let func_meta = self.context.def_interner.function_meta(&func_id);

        for (param, argument) in func_meta.parameters.iter().zip(arguments.into_iter()) {
            let param_id = param.0;
            let param_name = self.context.def_interner.ident_name(&param_id);

            new_env.store(param_name, argument);
        }

        let return_val = self.apply_func(&mut new_env, &func_id)?;

        Ok(return_val)
    }

    fn apply_func(
        &mut self,
        env: &mut Environment,
        func_id: &FuncId,
    ) -> Result<Object, RuntimeErrorKind> {
        let function = self.context.def_interner.function(func_id);
        let block = function.block(&self.context.def_interner);
        self.eval_block(env, block.statements())
    }

    fn eval_block(
        &mut self,
        env: &mut Environment,
        block: &[StmtId],
    ) -> Result<Object, RuntimeErrorKind> {
        let mut result = Object::Null;
        for stmt in block {
            result = self.evaluate_statement(env, stmt)?;
        }
        Ok(result)
    }

    fn expression_list_to_objects(
        &mut self,
        env: &mut Environment,
        exprs: &[ExprId],
    ) -> (Vec<Object>, Vec<RuntimeErrorKind>) {
        let (objects, errors): (Vec<_>, Vec<_>) = exprs
            .iter()
            .map(|expr| self.expression_to_object(env, expr))
            .partition(Result::is_ok);

        let objects: Vec<_> = objects.into_iter().map(Result::unwrap).collect();
        let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
        (objects, errors)
    }
}

/// We allow users to input an array in the ABI
/// Each element must be mapped to a unique identifier
/// XXX: At the moment, the evaluator uses String, in particular the variable name
/// This function ensures that each element in the array is assigned a unique identifier
pub fn mangle_array_element_name(array_name: &str, element_index: usize) -> String {
    use blake2::Digest;

    let mut hasher = Blake2s::new();
    hasher.update(array_name);

    // use u128 so we do not get different hashes depending on the computer
    // architecture
    let index_u128 = element_index as u128;
    hasher.update(index_u128.to_be_bytes());

    let res = hasher.finalize();

    // If a variable is named array_0_1f4a
    // Then it will be certain, that the user
    // is trying to be malicious
    // For regular users, they will never encounter a name conflict
    // We could probably use MD5 here, as we do not need a crypto hash
    let checksum = &res[0..4];

    format!("{}__{}__{:x?}", array_name, element_index, checksum)
}
