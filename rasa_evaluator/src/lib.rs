pub mod binary_op;
pub mod circuit;
pub mod environment;
pub mod func;
mod infix_evaluator;
pub mod optimise;
pub mod polynomial;

use std::collections::BTreeMap;
use librasac_ast::*;
pub use circuit::{Circuit, Selector, Witness};
pub use environment::Environment;
use rasa_field::FieldElement;
use func::Function;

use librasac_parser::Program;
pub use circuit::gate::Gate;
use optimise::Optimiser;
pub use polynomial::{Arithmetic, Linear, Polynomial};
use std::collections::HashMap;

pub struct Evaluator {
    num_witness: usize,
    num_selectors: usize,
    pub(crate) witnesses: Vec<Witness>,
    selectors: Vec<Selector>,
    statements: Vec<Statement>,
    num_public_inputs : usize,
    main_function: Function,
    gates: Vec<Gate>,                    // Identifier, Polynomial
    functions: HashMap<Ident, Function>, // XXX: We probably want an environment of functions that are available when we introduce imports. Not every file should have access to every function
    counter: usize,                      // This is so that we can get a unique number
}

impl Evaluator {
    pub fn new(program: Program) -> Evaluator {

        let Program {
            statements,
            functions,
            main,
            directives,
            custom_directives,
        } = program;

        let functions = Evaluator::parse_function_declarations(functions);

        // Check that we have a main function
        let main_function = match main {  
            None => panic!("Could not find a main function, currently we do not support library projects"),
            Some(main_func_dec) => {
                let (_, func) = Evaluator::parse_function_declaration(main_func_dec);
                func
            }, 
        };
    
        Evaluator {
            num_witness: 0,
            num_selectors: 0,
            num_public_inputs: 0,
            witnesses: Vec::new(),
            selectors: Vec::new(),
            statements,
            main_function,
            gates: Vec::new(),
            functions,
            counter: 0,
        }
    }

    /// Convert all of the function declarations in the Program
    /// into Functions that the Evaluator can use
    fn parse_function_declarations(func_decs: Vec<FunctionDefinition>) -> HashMap<Ident, Function> {
        let mut functions = HashMap::new();

        for func_dec in func_decs.into_iter() {
            let (func_name, func) = Evaluator::parse_function_declaration(func_dec);

            functions.insert(func_name, func);
        }

        functions
    }
    
    /// Convert a function declarations into a function object
    fn parse_function_declaration(func_dec: FunctionDefinition) -> (Ident, Function) {
            let func_name = func_dec.name;
            let body = func_dec.func.body;
            let parameters = func_dec.func.parameters;

            // Store function in evaluator
            (func_name, Function { body, parameters })
    }



    // Returns the current counter value and then increments the counter
    // This is so that we can have unique variable names when the same function is called multiple times
    fn get_unique_value(&mut self) -> usize {
        self.counter += 1;
        self.counter
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

    pub fn evaluate(mut self, env: &mut Environment) -> (Circuit, usize, usize) {
        self.parse_types(env);
        const WIDTH: usize = 3;


        let optimiser = Optimiser::new(WIDTH);

        let mut intermediate_variables: BTreeMap<Witness, Arithmetic> = BTreeMap::new();

        // Optimise the arithmetic gates by reducing them into the correct width and creating intermediate variables when necessary
        let num_witness = self.num_witnesses();
        let mut optimised_arith_gates: Vec<_> = self
            .gates
            .into_iter()
            .map(|gate| match gate {
                Gate::Arithmetic(arith) => {
                    
                optimiser.optimise(arith, &mut intermediate_variables, num_witness)
                
                },
            })
            .collect();
     

        // The optimiser could have created intermediate variables/witnesses. We need to add these to the circuit
        for (witness, gate) in intermediate_variables {
            // Add intermediate variables as witnesses
            self.num_witness += 1;
            self.witnesses.push(witness);
            // Add gate into the circuit
            optimised_arith_gates.push(gate);

            // XXX: We can additionally check that these arithmetic gates are done correctly via our optimiser -- It should have no effect if passed in twice
        }

        // Print all gates for debug purposes
        for gate in optimised_arith_gates.iter() {
            dbg!(gate);
        }

        for (i, witness) in self.witnesses.iter().enumerate() {
            dbg!(i, witness);
        }

        (Circuit(optimised_arith_gates), self.witnesses.len(), self.num_public_inputs)
    }

    // When we are multiplying arithmetic gates by each other, if one gate has too many terms
    // It is better to create an intermediate variable which links to the gate and then multiply by that intermediate variable
    // instead
    pub fn create_intermediate_variable(
        &mut self,
        env: &mut Environment,
        arithmetic_gate: Arithmetic,
    ) -> Polynomial {
        // Create a new witness variable
        let inter_var_name = format!("{}_{}", "_inter", self.get_unique_value());

        // Add witness to the constraint system
        self.store_witness(inter_var_name.clone());
        let witness_poly = self.store_lone_variable(inter_var_name, env);

        // We know it is a Linear polynomial, so we match on that.
        let linear_poly = match witness_poly.clone() {
            Polynomial::Linear(x) => x,
            _ => unimplemented!("Expected the intermediate variable to be a linear polynomial"),
        };

        // Link that witness to the arithmetic gate
        let constraint = &arithmetic_gate - &linear_poly.into();
        self.gates.push(Gate::Arithmetic(constraint));
        witness_poly
    }

    pub fn evaluate_infix_expression(
        &mut self,
        env: &mut Environment,
        lhs: Polynomial,
        rhs: Polynomial,
        op: BinaryOp,
    ) -> Polynomial {
        match op {
            BinaryOp::Add => binary_op::handle_add_op(lhs, rhs),
            BinaryOp::Subtract => binary_op::handle_sub_op(lhs, rhs),
            BinaryOp::Multiply => binary_op::handle_mul_op(lhs, rhs, env, self),
            BinaryOp::Divide => binary_op::handle_div_op(lhs, rhs, env, self),
            BinaryOp::NotEqual => binary_op::handle_neq_op(lhs, rhs, env, self),
            BinaryOp::Equal => binary_op::handle_equal_op(lhs, rhs, env, self),
            _ => panic!("Currently the {:?} op is not supported", op),
        }
    }

    // When we evaluate an identifier , it will be a linear polynomial
    // This is because, we currently do not have support for optimisations with polynomials of higher degree or higher fan-ins
    // XXX: One way to configure this in the future, is to count the fan-in/out and check if it is lower than the configured width
    // Either it is 1 * x + 0 or it is ax+b
    fn evaluate_identifier(&mut self, ident: String, env: &mut Environment) -> Polynomial {
        let polynomial = env.get(ident.clone());
        assert!(polynomial.is_linear());
        polynomial
    }

    // Converts all `private` types to witnesses
    // `const` types to constant
    // XXX: Should we use Ident for readability?
    pub fn parse_types(&mut self, env: &mut Environment) {
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
        for param_name in pub_inputs.into_iter().chain(witnesses.into_iter()) {
            self.store_witness(param_name.0.clone());
            self.store_lone_variable(param_name.0.clone(), env);
        }


        // Now call the main function
        for statement in self.main_function.body.0.clone().iter() {
            self.evaluate_statement(env, statement);
        }
    }
    // private x = (y == z) + ( k != p);
    // Is this problem only applicable to private statements?
    // constrain x == (x + p) + (z == d)
    //What if we add a flag called predicate, which analyses the AST and checks for statements of the above form
    // then we convert the statement to a predicate statement, which is also a part of the AST

    /*

    struct Predicate {
        op : BinaryOp,
        lhs : Expression
        rhs : Expression
    }

    When the evaluator see a PredicateExpression, it computes the predicate for that BinaryOP

    if op is == , then use the rangeproof predicate in dusk Docs
    if the op is !=, then we can use the inverse function
    if the op is <= then we can use the rangeproof gadget from before wit maybe equal


    so // constrain x == (x + p) + (z == d)

    would convert to constrain x == (x + p) + Predicate(z==d)

    */

    /*
    struct constraint  {
        op
    }
    */
    // Create a concept called delayed constraints
    fn evaluate_statement(&mut self, env: &mut Environment, statement: &Statement) -> Polynomial {
        match statement {
            Statement::Private(x) => {
               self.handle_private_statement(env, *x.clone())
            }
            Statement::Constrain(constrain_stmt) => {
                self.handle_constrain_statement(env, constrain_stmt)
            }
            // constant statements do not create constraints
            Statement::Const(x) => {
                self.num_selectors = self.num_selectors + 1;
                let variable_name: String = x.identifier.clone().0;
                let value = self.evaluate_integer(env, x.expression.clone()); // const can only be integers/Field elements, cannot involve the witness, so we can eval at compile
                self.selectors
                    .push(Selector(variable_name.clone(), value.clone()));
                env.store(variable_name, value);
                Polynomial::Null
            }
            Statement::Expression(expr_stmt) => {
                self.expression_to_polynomial(env, expr_stmt.0.clone())
            }
            _ => {
                panic!("This statement type has not been implemented");
            }
        }
    }

    // XXX(med) : combine these two methods and or rename
    // XXX(bug): If you call store_witness after store_lone_variable, then the Polynomial will have the index of the previous witness
    // XXX: Maybe better to name it `create_witness`
    fn store_witness(&mut self, variable_name: String) -> Witness{
        self.num_witness = self.num_witness + 1;
        let witness = Witness(variable_name, self.num_witness);
        self.witnesses
            .push(witness.clone());
            witness
    }
    fn store_lone_variable(&mut self, variable_name: String, env: &mut Environment) -> Polynomial {
        let value = Polynomial::from_witness(Witness(variable_name.clone(), self.num_witness));
        env.store(variable_name, value.clone());
        value
    }


    // The LHS of a private statement is always a new witness
    // Cannot do `private x + k = z`
    // It is also a new variable, since private is used to derive variables
    fn handle_private_statement(&mut self, env: &mut Environment, x: PrivateStatement) -> Polynomial{
        let variable_name: String = x.identifier.clone().0;
        let witness = self.store_witness(variable_name.clone()); 

        let rhs_poly = self.expression_to_polynomial(env, x.expression.clone());
        
        match rhs_poly.arithmetic() {
            Some(arith) => {
                
                let witness_linear = Linear::from_witness(witness.clone());

                let witness_poly = Polynomial::from_witness(witness);

                // Store in environment, so we can get it by variable name
                // XXX: Can add a method called LINK to wrap this
                env.store(variable_name, witness_poly.clone()); 

                self.gates.push(Gate::Arithmetic(arith - &witness_linear.into()))
            },
            None => {
                env.store(variable_name.clone(), rhs_poly.clone());
                assert!(rhs_poly.is_linear()); // XXX: Cannot do priv x = 5; x is a constant in this case 
            },
        };

        Polynomial::Null
    }

    // The LHS of a private statement is always a new witness
    // Cannot do `private x + k = z`
    // It is also a new variable, since private is used to derive variables
    fn handle_constrain_statement(&mut self, env: &mut Environment, constrain_stmt: &ConstrainStatement) -> Polynomial{
                let lhs_poly = self.expression_to_polynomial(env, constrain_stmt.0.lhs.clone());
                let rhs_poly = self.expression_to_polynomial(env, constrain_stmt.0.rhs.clone());
                
                // Evaluate the constrain infix statement
                let _ = self.evaluate_infix_expression(env, lhs_poly.clone(), rhs_poly.clone(), constrain_stmt.0.operator);

                // XXX: WE could probably move this into equal folder, as it is an optimisation that only applies to it
                if constrain_stmt.0.operator == BinaryOp::Equal {
                    // Check if we have any lone variables and then if the other side is a linear/constant
                    let (witness, rhs ) = match (lhs_poly.is_unit_witness(),rhs_poly.is_unit_witness()) {
                        (true, _) => (lhs_poly.witness(), rhs_poly),
                        (_, true) => (rhs_poly.witness(), lhs_poly),
                        (_, _) => (None, Polynomial::Null)
                    };

                    match witness {
                        Some(wit) => {
                            // Check if the RHS is linear or constant
                            if rhs.is_linear() | rhs.is_constant() {
                                env.store(wit.0, rhs)
                            }

                        },
                        None => { }
                    };
                };
                Polynomial::Null
    }

    pub fn evaluate_integer(&mut self, env: &mut Environment, expr: Expression) -> Polynomial {
        let polynomial = self.expression_to_polynomial(env, expr);

        // Check that it is a constant, currently we only have integer constants
        // XXX: We could possibly add the public inputs logic aswell Polynomial::Public
        // XXX: Think about this some more, as public inputs are ultimately private
        match polynomial {
            Polynomial::Constants(_) => return polynomial,
            _ => panic!("Expected a constant. Only Constants can be integers"),
        }
    }

    pub fn expression_to_polynomial(
        &mut self,
        env: &mut Environment,
        expr: Expression,
    ) -> Polynomial {
        match expr {
            Expression::Literal(Literal::Integer(x)) => Polynomial::Constants(x.into()),
            Expression::Ident(x) => self.evaluate_identifier(x.to_string(), env),
            Expression::Infix(infx) => {
                let lhs = self.expression_to_polynomial(env, infx.lhs);
                let rhs = self.expression_to_polynomial(env, infx.rhs);
                self.evaluate_infix_expression(env, lhs, rhs, infx.operator)
            }
            Expression::Call(call_expr) => {
                // First fetch the function using it's name
                let func = self.functions.get(&call_expr.func_name).unwrap().clone();

                // Create a new environment for this function
                // This is okay because functions are not stored in the environment
                // We need to add the arguments into the environment though
                let mut new_env = Environment::new();
                let arguments: Vec<Polynomial> = call_expr
                    .arguments
                    .iter()
                    .map(|expr| self.expression_to_polynomial(env, expr.clone()))
                    .collect();

                // We also need to check that each argument matches with the correct type and correct number of parameters, this will be done in the type checker

                for ((param_name, param_type), argument) in
                    func.parameters.iter().zip(arguments.iter())
                {
                    new_env.store(param_name.0.clone(), argument.clone());
                }

                let return_val = self.apply_func(&mut new_env, func);

                // Take all of the arithmetic gates from the functions environment and add it to the global environment
                for (ident, polynomial) in new_env.0.into_iter() {
                    env.store(ident, polynomial);
                }

                return_val
            }
            k => {
                dbg!(k);
                todo!()
            }
        }
    }

    fn apply_func(&mut self, env: &mut Environment, func: Function) -> Polynomial {
        self.eval_block(env, func.body)
    }

    // XXX: FixME, eval_block should return a Polynomial which means that eval_stmt should also
    fn eval_block(&mut self, env: &mut Environment, block: BlockStatement) -> Polynomial {
        let mut result = Polynomial::Null;
        for stmt in block.0.iter() {
            result = self.evaluate_statement(env, stmt);
        }
        result
    }
}
