mod binary_op;
mod environment;
mod low_level_function_impl;
mod builtin;
mod object;

use std::collections::{BTreeMap, HashMap};

use object::{Array, Integer, Object, Selector, RangedObject};
use environment::Environment;
use acir::optimiser::CSatOptimiser;

use acir::native_types::{Witness, Arithmetic, Linear};
use acir::circuit::gate::{AndGate, Gate, XorGate};
use acir::circuit::Circuit;

use noirc_frontend::ast::FunctionDefinition as Function;
use noirc_frontend::symbol_table::{SymbolTable, NoirFunction};
use noirc_frontend::ast::*;
use noirc_frontend::lexer::token::Attribute;
use noirc_frontend::parser::Program;

use noir_field::FieldElement;

pub struct Evaluator {
    num_witness: usize,                           // XXX: Can possibly remove
    num_selectors: usize,                         // XXX: Can possibly remove
    pub(crate) witnesses: HashMap<Witness, Type>, //XXX: Move into symbol table/environment -- Check if typing is needed here
    selectors: Vec<Selector>, // XXX: Possibly move into environment
    statements: Vec<Statement>,
    symbol_table: SymbolTable,
    num_public_inputs: usize,
    main_function: Function,
    gates: Vec<Gate>, // Identifier, Object
    counter: usize,   // This is so that we can get a unique number
}

impl Evaluator {
    pub fn new(program: Program, symbol_table: SymbolTable) -> Evaluator {
        let Program {
            statements,
            imports: _,
            functions: _,
            main: _,
            modules : _,
        } = program;

        // Check that we have a main function
        let possible_main = symbol_table.look_up_main_func();
        
        let main_function = match possible_main {
            None => panic!(
                "Could not find a main function, currently we do not support library projects"
            ),
            Some(main_func) => main_func,
        };

        Evaluator {
            num_witness: 0,
            num_selectors: 0,
            num_public_inputs: 0,
            witnesses: HashMap::new(),
            selectors: Vec::new(),
            symbol_table,
            statements,
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

    fn create_fresh_witness(
        &mut self,
        name: String,
        env: &mut Environment,
    ) -> (Witness, Object) {
        let unique_name = format!("{}{}", name, self.get_unique_value(),);
        let witness = self.store_witness(unique_name.clone(), Type::Witness);
        let poly = self.store_lone_variable(unique_name, env);
        (witness, poly)
    }

    // XXX: Fix this later, with Debug trait. Only using it now for REPL
    pub fn debug(&self) {
        for wit in self.witnesses.iter() {
            dbg!(wit);
        }
        for sel in self.selectors.iter() {
            dbg!(sel);
        }
    }

    pub fn num_witnesses(&self) -> usize {
        self.num_witness
    }

    // XXX: We return the num_witnesses, but this is the max number of witnesses
    // Some of these could have been removed due to optimisations. We need this number because the
    // Standard format requires the number of witnesses. The max number is also fine.
    // If we had a composer object, we would not need it
    pub fn evaluate(mut self) -> (Circuit, usize, usize) {

        // create a new environment
        let mut env = Environment::new();

        // First compile
        self.compile(&mut env);

        // Then optimise for a width3 plonk program
        // XXX: We can move all of this stuff into a plonk-backend program
        // which takes the IR as input
        const WIDTH: usize = 3;

        let optimiser = CSatOptimiser::new(WIDTH);

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
        for gate in optimised_arith_gates.iter() {
            dbg!(gate);
        }

        for (i, witness) in self.witnesses.iter().enumerate() {
            dbg!(i, witness);
        }

        (
            Circuit(optimised_arith_gates),
            self.witnesses.len(),
            self.num_public_inputs,
        )
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
        // Create a new witness variable
        let inter_var_name = format!("{}_{}", "_inter", self.get_unique_value());

        // Add witness to the constraint system
        let inter_var_witness = self.store_witness(inter_var_name.clone(), typ);
        let witness_poly = self.store_lone_variable(inter_var_name, env);

        // We know it is a Linear polynomial, so we match on that.
        let linear_poly = match witness_poly.clone() {
            Object::Linear(x) => x,
            _ => unimplemented!("Expected the intermediate variable to be a linear polynomial"),
        };

        // Link that witness to the arithmetic gate
        let constraint = &arithmetic_gate - &linear_poly.into();
        self.gates.push(Gate::Arithmetic(constraint));
        (witness_poly, inter_var_witness)
    }

    pub fn evaluate_infix_expression(
        &mut self,
        env: &mut Environment,
        lhs: Object,
        rhs: Object,
        op: BinaryOp,
    ) -> Object {
        match op {
            BinaryOp::Add => binary_op::handle_add_op(lhs, rhs, env, self),
            BinaryOp::Subtract => binary_op::handle_sub_op(lhs, rhs, env, self),
            BinaryOp::Multiply => binary_op::handle_mul_op(lhs, rhs, env, self),
            BinaryOp::Divide => binary_op::handle_div_op(lhs, rhs, env, self),
            BinaryOp::NotEqual => binary_op::handle_neq_op(lhs, rhs, env, self),
            BinaryOp::Equal => binary_op::handle_equal_op(lhs, rhs, env, self),
            BinaryOp::And => binary_op::handle_and_op(lhs, rhs, env, self),
            BinaryOp::Xor => binary_op::handle_xor_op(lhs, rhs, env, self),
            BinaryOp::Less => binary_op::handle_less_than_op(lhs, rhs, env, self),
            BinaryOp::LessEqual => binary_op::handle_less_than_equal_op(lhs, rhs, env, self),
            BinaryOp::Greater => binary_op::handle_greater_than_op(lhs, rhs, env, self),
            BinaryOp::GreaterEqual => binary_op::handle_greater_than_equal_op(lhs, rhs, env, self),
            _ => panic!("Currently the {:?} op is not supported", op),
        }
    }

    // When we evaluate an identifier , it will be a linear polynomial
    // This is because, we currently do not have support for optimisations with polynomials of higher degree or higher fan-ins
    // XXX: One way to configure this in the future, is to count the fan-in/out and check if it is lower than the configured width
    // Either it is 1 * x + 0 or it is ax+b
    fn evaluate_identifier(&mut self, ident: String, env: &mut Environment) -> Object {
        let polynomial = env.get(ident.clone());
        polynomial
    }


    /// Compiles the AST into the intermediate format, which we call the gates
    pub fn compile(&mut self, env: &mut Environment) {
        // Add the parameters from the main function into the evaluator as witness variables
        // XXX: We are only going to care about Public and Private witnesses for now

        let mut pub_inputs = Vec::new();
        let mut witnesses = Vec::new();

        for (param_name, param_type) in self.main_function.parameters.clone().into_iter() {
            match param_type {
                Type::Public =>{
                    pub_inputs.push(param_name);
                },
                Type::Witness => {
                    witnesses.push(param_name)
                },
                _=> unimplemented!("Currently we only have support for Private and Public inputs in the main function definition")
            }
        }

        self.num_public_inputs = pub_inputs.len();

        // Add all of the public inputs first, then the witnesses
        for param_name in pub_inputs.into_iter() {
            self.store_witness(param_name.0.clone().contents, Type::Public);
            self.store_lone_variable(param_name.0.clone().contents, env);
        }

        for param_name in witnesses.into_iter() {
            self.store_witness(param_name.0.clone().contents, Type::Witness);
            self.store_lone_variable(param_name.0.clone().contents, env);
        }

        // Now call the main function
        for statement in self.main_function.body.0.clone().into_iter() {
            self.evaluate_statement(env, statement);
        }
    }

    fn evaluate_statement(&mut self, env: &mut Environment, statement: Statement) -> Object {
        match statement {
            Statement::Private(x) => self.handle_private_statement(env, x.clone()),
            Statement::Constrain(constrain_stmt) => self.handle_constrain_statement(env, constrain_stmt),
            // constant statements do not create constraints
            Statement::Const(x) => {
                self.num_selectors = self.num_selectors + 1;
                let variable_name: String = x.identifier.0.contents;
                let value = self.evaluate_integer(env, x.expression); // const can only be integers/Field elements, cannot involve the witness, so we can eval at compile
                self.selectors
                    .push(Selector(variable_name.clone(), value.clone()));
                env.store(variable_name, value);
                Object::Null
            }
            Statement::Expression(expr) => self.expression_to_object(env, expr),
            Statement::Let(let_stmt) => {
                // let statements are used to declare a higher level object
                self.handle_let_statement(env, let_stmt);

                Object::Null
            }
            _ => {
                panic!("This statement type has not been implemented");
            }
        }
    }

    // XXX(med) : combine these two methods and or rename
    // XXX(bug) : If you call store_witness after store_lone_variable, then the Object will have the index of the previous witness
    // XXX: Maybe better to name it `create_witness`
    fn store_witness(&mut self, variable_name: String, typ: Type) -> Witness {
        self.num_witness = self.num_witness + 1;
        let witness = Witness(variable_name, self.num_witness);
        self.witnesses.insert(witness.clone(), typ);
        witness
    }
    fn store_lone_variable(&mut self, variable_name: String, env: &mut Environment) -> Object {
        let value = Object::from_witness(Witness(variable_name.clone(), self.num_witness));
        env.store(variable_name, value.clone());
        value
    }

    // The LHS of a private statement is always a new witness
    // Cannot do `private x + k = z`
    // It is also a new variable, since private is used to derive variables
    fn handle_private_statement(
        &mut self,
        env: &mut Environment,
        x: PrivateStatement,
    ) -> Object {
        let variable_name: String = x.identifier.clone().0.contents;
        let witness = self.store_witness(variable_name.clone(), x.r#type.clone());
        let witness_linear = Linear::from_witness(witness.clone());

        let rhs_poly = self.expression_to_object(env, x.expression.clone());

        match rhs_poly.arithmetic() {
            Some(arith) => {
                self.gates
                    .push(Gate::Arithmetic(arith - &witness_linear.into()));
            }
            None => {
                assert!(rhs_poly.is_linear()); // XXX: Cannot do priv x = 5; x is a constant in this case
                                               // XXX: To simplify apply constraint, even if we know it is an linear poly.
                                               // XXX: We can check this in the semantic analyser and modify the AST, so that we always apply a constraint here
                                               // Because the SA will optimise away the linear constraints

                let lhs = Arithmetic::from(witness_linear);
                let rhs = Arithmetic::from(rhs_poly.linear().unwrap());
                self.gates.push(Gate::Arithmetic(&lhs - &rhs));
            }
        };

        // Check the type so we can see if we need to apply an extra constraint to the witness
        let rhs_poly = match &x.r#type {
            //  Check if the type requires us to apply an extra constraint
            Type::Integer(_, num_bits) => {
                let integer = Integer::from_witness(witness, *num_bits);
                integer.constrain(self);
                Object::Integer(integer)
            }
            Type::Witness => Object::from_witness(witness),
            k => panic!("Oops, Expected an integer or Witness type, found {:?}", k),
        };
        env.store(variable_name.clone(), rhs_poly);

        Object::Null
    }

    // The LHS of a private statement is always a new witness
    // Cannot do `private x + k = z`
    // It is also a new variable, since private is used to derive variables
    fn handle_constrain_statement(
        &mut self,
        env: &mut Environment,
        constrain_stmt: ConstrainStatement,
    ) -> Object {
        let lhs_poly = self.expression_to_object(env, constrain_stmt.0.lhs);
        let rhs_poly = self.expression_to_object(env, constrain_stmt.0.rhs);

        // Evaluate the constrain infix statement
        let _ = self.evaluate_infix_expression(
            env,
            lhs_poly.clone(),
            rhs_poly.clone(),
            constrain_stmt.0.operator,
        );

        // XXX: WE could probably move this into equal folder, as it is an optimisation that only applies to it
        if constrain_stmt.0.operator == BinaryOp::Equal {
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
        Object::Null
    }
    // Let statements are used to declare higher level objects
    fn handle_let_statement(
        &mut self,
        env: &mut Environment,
        let_stmt: LetStatement,
    ) -> Object {
        // Convert the LHS into an identifier
        let variable_name: String = let_stmt.identifier.0.contents;

        // XXX: Currently we only support arrays using this, when other types are introduced
        // we can extend into a separate (generic) module

        // Extract the array
        let rhs_poly = self.expression_to_object(env, let_stmt.expression);

        match rhs_poly {
            Object::Array(arr) => {
                env.store(variable_name.into(), Object::Array(arr));
            }
            _ => panic!(
                "logic for types that are not arrays in a let statement, not implemented yet!"
            ),
        };

        Object::Null
    }
    fn handle_for_expr(
        &mut self,
        env: &mut Environment,
        for_expr: ForExpression,
    ) -> Object {
        
        // First create an iterator over all of the for loop identifiers
        // XXX: We preferably need to introduce public integers and private integers, so that we can 
        // loop securely on constants. This requires `constant as u128`, analysis will take care of the rest 
        let start = self.expression_to_object(env, for_expr.start_range).constant().unwrap();
        let end = self.expression_to_object(env, for_expr.end_range).constant().unwrap();
        let ranged_object = RangedObject::new(start, end);
        
        let mut contents : Vec<Object> = Vec::new();

        for indice in ranged_object {
            env.start_for_loop();

            // Add indice to environment
            let variable_name: String = for_expr.identifier.0.clone().contents;
            env.store(variable_name, Object::Constants(indice));

            let return_typ = self.eval_block(env, for_expr.block.clone());
            contents.push(return_typ);

            env.end_for_loop();
        }
        let length = contents.len() as u128;

        Object::Array(Array{contents, length})
    }

    pub fn evaluate_integer(&mut self, env: &mut Environment, expr: Expression) -> Object {
        let polynomial = self.expression_to_object(env, expr);

        // Check that it is a constant, currently we only have integer constants
        // XXX: We could possibly add the public inputs logic aswell Object::Public
        // XXX: Think about this some more, as public inputs are ultimately private
        match polynomial {
            Object::Constants(_) => return polynomial,
            _ => panic!("Expected a constant. Only Constants can be integers"),
        }
    }

    pub fn expression_to_object(
        &mut self,
        env: &mut Environment,
        expr: Expression,
    ) -> Object {
        match expr.kind {
            ExpressionKind::Literal(Literal::Integer(x)) => Object::Constants(x.into()),
            ExpressionKind::Literal(Literal::Array(arr_lit)) => {
                Object::Array(Array::from(self, env, arr_lit))
            }
            ExpressionKind::Ident(x) => self.evaluate_identifier(x.to_string(), env),
            ExpressionKind::Infix(infx) => {
                let lhs = self.expression_to_object(env, infx.lhs);
                let rhs = self.expression_to_object(env, infx.rhs);
                self.evaluate_infix_expression(env, lhs, rhs, infx.operator)
            }
            ExpressionKind::Cast(cast_expr) => {
                let lhs = self.expression_to_object(env, cast_expr.lhs);
                binary_op::handle_cast_op(lhs, cast_expr.r#type, env, self)
            }
            ExpressionKind::Index(indexed_expr) => {
                // Currently these only happen for arrays
                let arr = env.get_array(indexed_expr.collection_name.0.clone().contents);
                arr.get(indexed_expr.index.kind.to_u128())
            }
            // This is currently specific to core library calls
            ExpressionKind::Call(path, call_expr) => {
                let func_name = call_expr.func_name.clone();
                let func_def = self.symbol_table.look_up_func(path.clone(), &func_name);
            
                let noir_func = match func_def {
                    Some(noir_func) => noir_func,
                    None => panic!("Tried to call {}, but function not found", &func_name.0.contents)
                };               
                // Choices are a low level func or an imported library function
                // If low level, then we use it's func name to find out what function to call
                // If not then we just call the library as usual with the function definition
                match noir_func {
                    NoirFunction::Function(compiled_func) => self.call_function(env, &call_expr, compiled_func.clone()),
                    NoirFunction::LowLevelFunction(func) => {
                        let attribute = func.attribute.expect("All low level functions must contain an attribute which contains the opcode which it links to");
                        match attribute {
                            Attribute::Foreign(opcode_name) => low_level_function_impl::call_low_level(self, env, &opcode_name, *call_expr),
                            Attribute::Builtin(builtin_name) => builtin::call_builtin(self, env, &builtin_name, *call_expr)
                        }
                        
                    },
                }
                    
            }
            ExpressionKind::For(for_expr) => {
                self.handle_for_expr(env,*for_expr)
            }
            k => {
                dbg!(k);
                todo!()
            }
        }
    }

    fn call_function(&mut self, env: &mut Environment, call_expr : &CallExpression, func: Function) -> Object {
              // Create a new environment for this function
                // This is okay because functions are not stored in the environment
                // We need to add the arguments into the environment though
                // Note: The arguments are evaluated in the old environment
                let mut new_env = Environment::new();
                let arguments: Vec<Object> = call_expr
                    .arguments
                    .iter()
                    .map(|expr| self.expression_to_object(env, expr.clone()))
                    .collect();

                // We also need to check that each argument matches with the correct type and correct number of parameters
                // XXX: This will be done in the type checker - analysis

                for ((param_name, param_type), argument) in
                    func.parameters.iter().zip(arguments.iter())
                {
                    new_env.store(param_name.0.clone().contents, argument.clone());
                }

                let return_val = self.apply_func(&mut new_env, func);

                // Take all of the arithmetic gates from the functions environment and add it to the global environment
                // XXX: Check this
                // for (ident, polynomial) in new_env.into_iter() {
                //     env.store(ident, polynomial);
                // }

                return_val
    }

    fn apply_func(&mut self, env: &mut Environment, func: Function) -> Object {
        self.eval_block(env, func.body)
    }

    fn eval_block(&mut self, env: &mut Environment, block: BlockStatement) -> Object {
        let mut result = Object::Null;
        for stmt in block.0.into_iter() {
            result = self.evaluate_statement(env, stmt);
        }
        result
    }
}
