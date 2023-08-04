mod context;
mod program;
mod value;

pub(crate) use program::Ssa;

use context::SharedContext;
use iter_extended::vecmap;
use noirc_errors::Location;
use noirc_frontend::monomorphization::ast::{self, Expression, Program};

use self::{
    context::FunctionContext,
    value::{Tree, Values},
};

use super::ir::{function::RuntimeType, instruction::BinaryOp, types::Type, value::ValueId};

/// Generates SSA for the given monomorphized program.
///
/// This function will generate the SSA but does not perform any optimizations on it.
pub(crate) fn generate_ssa(program: Program) -> Ssa {
    let context = SharedContext::new(program);

    let main_id = Program::main_id();
    let main = context.program.main();

    // Queue the main function for compilation
    context.get_or_queue_function(main_id);

    let mut function_context = FunctionContext::new(
        main.name.clone(),
        &main.parameters,
        if main.unconstrained { RuntimeType::Brillig } else { RuntimeType::Acir },
        &context,
    );
    function_context.codegen_function_body(&main.body);

    // Main has now been compiled and any other functions referenced within have been added to the
    // function queue as they were found in codegen_ident. This queueing will happen each time a
    // previously-unseen function is found so we need now only continue popping from this queue
    // to generate SSA for each function used within the program.
    while let Some((src_function_id, dest_id)) = context.pop_next_function_in_queue() {
        let function = &context.program[src_function_id];
        function_context.new_function(dest_id, function);
        function_context.codegen_function_body(&function.body);
    }

    function_context.builder.finish()
}

impl<'a> FunctionContext<'a> {
    /// Codegen a function's body and set its return value to that of its last parameter.
    /// For functions returning nothing, this will be an empty list.
    fn codegen_function_body(&mut self, body: &Expression) {
        let return_value = self.codegen_expression(body);
        let results = return_value.into_value_list(self);
        self.builder.terminate_with_return(results);
    }

    fn codegen_expression(&mut self, expr: &Expression) -> Values {
        match expr {
            Expression::Ident(ident) => self.codegen_ident(ident),
            Expression::Literal(literal) => self.codegen_literal(literal),
            Expression::Block(block) => self.codegen_block(block),
            Expression::Unary(unary) => self.codegen_unary(unary),
            Expression::Binary(binary) => self.codegen_binary(binary),
            Expression::Index(index) => self.codegen_index(index),
            Expression::Cast(cast) => self.codegen_cast(cast),
            Expression::For(for_expr) => self.codegen_for(for_expr),
            Expression::If(if_expr) => self.codegen_if(if_expr),
            Expression::Tuple(tuple) => self.codegen_tuple(tuple),
            Expression::ExtractTupleField(tuple, index) => {
                self.codegen_extract_tuple_field(tuple, *index)
            }
            Expression::Call(call) => self.codegen_call(call),
            Expression::Let(let_expr) => self.codegen_let(let_expr),
            Expression::Constrain(constrain, location) => {
                self.codegen_constrain(constrain, *location)
            }
            Expression::Assign(assign) => self.codegen_assign(assign),
            Expression::Semi(semi) => self.codegen_semi(semi),
        }
    }

    /// Codegen any non-tuple expression so that we can unwrap the Values
    /// tree to return a single value for use with most SSA instructions.
    fn codegen_non_tuple_expression(&mut self, expr: &Expression) -> ValueId {
        self.codegen_expression(expr).into_leaf().eval(self)
    }

    /// Codegen a reference to an ident.
    /// The only difference between this and codegen_ident is that if the variable is mutable
    /// as in `let mut var = ...;` the `Value::Mutable` will be returned directly instead of
    /// being automatically loaded from. This is needed when taking the reference of a variable
    /// to reassign to it. Note that mutable references `let x = &mut ...;` do not require this
    /// since they are not automatically loaded from and must be explicitly dereferenced.
    fn codegen_ident_reference(&mut self, ident: &ast::Ident) -> Values {
        match &ident.definition {
            ast::Definition::Local(id) => self.lookup(*id),
            ast::Definition::Function(id) => self.get_or_queue_function(*id),
            ast::Definition::Oracle(name) => self.builder.import_foreign_function(name).into(),
            ast::Definition::Builtin(name) | ast::Definition::LowLevel(name) => {
                match self.builder.import_intrinsic(name) {
                    Some(builtin) => builtin.into(),
                    None => panic!("No builtin function named '{name}' found"),
                }
            }
        }
    }

    /// Codegen an identifier, automatically loading its value if it is mutable.
    fn codegen_ident(&mut self, ident: &ast::Ident) -> Values {
        self.codegen_ident_reference(ident).map(|value| value.eval(self).into())
    }

    fn codegen_literal(&mut self, literal: &ast::Literal) -> Values {
        match literal {
            ast::Literal::Array(array) => {
                let elements = vecmap(&array.contents, |element| self.codegen_expression(element));
                let typ = Self::convert_non_tuple_type(&array.typ);
                self.codegen_array(elements, typ)
            }
            ast::Literal::Integer(value, typ) => {
                let typ = Self::convert_non_tuple_type(typ);
                self.builder.numeric_constant(*value, typ).into()
            }
            ast::Literal::Bool(value) => {
                self.builder.numeric_constant(*value as u128, Type::bool()).into()
            }
            ast::Literal::Str(string) => {
                let elements = vecmap(string.as_bytes(), |byte| {
                    self.builder.numeric_constant(*byte as u128, Type::field()).into()
                });
                let typ = Self::convert_non_tuple_type(&ast::Type::String(elements.len() as u64));
                self.codegen_array(elements, typ)
            }
            ast::Literal::FmtStr(string, number_of_fields, fields) => {
                // A caller needs multiple pieces of information to make use of a format string
                // The message string, the number of fields to be formatted, and the fields themselves
                let string = Expression::Literal(ast::Literal::Str(string.clone()));
                let number_of_fields = Expression::Literal(ast::Literal::Integer(
                    (*number_of_fields as u128).into(),
                    ast::Type::Field,
                ));
                let fields = *fields.clone();
                let fmt_str_tuple = &[string, number_of_fields, fields];
                self.codegen_tuple(fmt_str_tuple)
            }
        }
    }

    /// Codegen an array by allocating enough space for each element and inserting separate
    /// store instructions until each element is stored. The store instructions will be separated
    /// by add instructions to calculate the new offset address to store to next.
    ///
    /// In the case of arrays of structs, the structs are flattened such that each field will be
    /// stored next to the other fields in memory. So an array such as [(1, 2), (3, 4)] is
    /// stored the same as the array [1, 2, 3, 4].
    ///
    /// The value returned from this function is always that of the allocate instruction.
    fn codegen_array(&mut self, elements: Vec<Values>, typ: Type) -> Values {
        let mut array = im::Vector::new();

        for element in elements {
            element.for_each(|element| {
                let element = element.eval(self);
                array.push_back(element);
            });
        }

        self.builder.array_constant(array, typ).into()
    }

    fn codegen_block(&mut self, block: &[Expression]) -> Values {
        let mut result = Self::unit_value();
        for expr in block {
            result = self.codegen_expression(expr);
        }
        result
    }

    fn codegen_unary(&mut self, unary: &ast::Unary) -> Values {
        match unary.operator {
            noirc_frontend::UnaryOp::Not => {
                let rhs = self.codegen_expression(&unary.rhs);
                let rhs = rhs.into_leaf().eval(self);
                self.builder.insert_not(rhs).into()
            }
            noirc_frontend::UnaryOp::Minus => {
                let rhs = self.codegen_expression(&unary.rhs);
                let rhs = rhs.into_leaf().eval(self);
                let typ = self.builder.type_of_value(rhs);
                let zero = self.builder.numeric_constant(0u128, typ);
                self.builder.insert_binary(zero, BinaryOp::Sub, rhs).into()
            }
            noirc_frontend::UnaryOp::MutableReference => {
                self.codegen_reference(&unary.rhs).map(|rhs| {
                    match rhs {
                        value::Value::Normal(value) => {
                            let alloc = self.builder.insert_allocate();
                            self.builder.insert_store(alloc, value);
                            Tree::Leaf(value::Value::Normal(alloc))
                        }
                        // NOTE: The `.into()` here converts the Value::Mutable into
                        // a Value::Normal so it is no longer automatically dereferenced.
                        value::Value::Mutable(reference, _) => reference.into(),
                    }
                })
            }
            noirc_frontend::UnaryOp::Dereference { .. } => {
                let rhs = self.codegen_expression(&unary.rhs);
                self.dereference(&rhs, &unary.result_type)
            }
        }
    }

    fn codegen_reference(&mut self, expr: &Expression) -> Values {
        match expr {
            Expression::Ident(ident) => self.codegen_ident_reference(ident),
            Expression::ExtractTupleField(tuple, index) => {
                let tuple = self.codegen_reference(tuple);
                Self::get_field(tuple, *index)
            }
            other => self.codegen_expression(other),
        }
    }

    fn codegen_binary(&mut self, binary: &ast::Binary) -> Values {
        let lhs = self.codegen_non_tuple_expression(&binary.lhs);
        let rhs = self.codegen_non_tuple_expression(&binary.rhs);
        self.insert_binary(lhs, binary.operator, rhs, binary.location)
    }

    fn codegen_index(&mut self, index: &ast::Index) -> Values {
        let array = self.codegen_non_tuple_expression(&index.collection);
        let index_value = self.codegen_non_tuple_expression(&index.index);
        self.codegen_array_index(array, index_value, &index.element_type, index.location)
    }

    /// This is broken off from codegen_index so that it can also be
    /// used to codegen a LValue::Index.
    ///
    /// Set load_result to true to load from each relevant index of the array
    /// (it may be multiple in the case of tuples). Set it to false to instead
    /// return a reference to each element, for use with the store instruction.
    fn codegen_array_index(
        &mut self,
        array: super::ir::value::ValueId,
        index: super::ir::value::ValueId,
        element_type: &ast::Type,
        location: Location,
    ) -> Values {
        // base_index = index * type_size
        let type_size = Self::convert_type(element_type).size_of_type();
        let type_size = self.builder.field_constant(type_size as u128);
        let base_index =
            self.builder.set_location(location).insert_binary(index, BinaryOp::Mul, type_size);

        let mut field_index = 0u128;
        Self::map_type(element_type, |typ| {
            let offset = self.make_offset(base_index, field_index);
            field_index += 1;
            self.builder.insert_array_get(array, offset, typ).into()
        })
    }

    fn codegen_cast(&mut self, cast: &ast::Cast) -> Values {
        let lhs = self.codegen_non_tuple_expression(&cast.lhs);
        let typ = Self::convert_non_tuple_type(&cast.r#type);
        self.builder.insert_cast(lhs, typ).into()
    }

    /// Codegens a for loop, creating three new blocks in the process.
    /// The return value of a for loop is always a unit literal.
    ///
    /// For example, the loop `for i in start .. end { body }` is codegen'd as:
    ///
    ///   v0 = ... codegen start ...
    ///   v1 = ... codegen end ...
    ///   br loop_entry(v0)
    /// loop_entry(i: Field):
    ///   v2 = lt i v1
    ///   brif v2, then: loop_body, else: loop_end
    /// loop_body():
    ///   v3 = ... codegen body ...
    ///   v4 = add 1, i
    ///   br loop_entry(v4)
    /// loop_end():
    ///   ... This is the current insert point after codegen_for finishes ...
    fn codegen_for(&mut self, for_expr: &ast::For) -> Values {
        let loop_entry = self.builder.insert_block();
        let loop_body = self.builder.insert_block();
        let loop_end = self.builder.insert_block();

        // this is the 'i' in `for i in start .. end { block }`
        let index_type = Self::convert_non_tuple_type(&for_expr.index_type);
        let loop_index = self.builder.add_block_parameter(loop_entry, index_type);

        self.builder.set_location(for_expr.start_range_location);
        let start_index = self.codegen_non_tuple_expression(&for_expr.start_range);

        self.builder.set_location(for_expr.end_range_location);
        let end_index = self.codegen_non_tuple_expression(&for_expr.end_range);

        self.builder.terminate_with_jmp(loop_entry, vec![start_index]);

        // Compile the loop entry block
        self.builder.switch_to_block(loop_entry);
        let jump_condition = self.builder.insert_binary(loop_index, BinaryOp::Lt, end_index);
        self.builder.terminate_with_jmpif(jump_condition, loop_body, loop_end);

        // Compile the loop body
        self.builder.switch_to_block(loop_body);
        self.define(for_expr.index_variable, loop_index.into());
        self.codegen_expression(&for_expr.block);
        let new_loop_index = self.make_offset(loop_index, 1);
        self.builder.terminate_with_jmp(loop_entry, vec![new_loop_index]);

        // Finish by switching back to the end of the loop
        self.builder.switch_to_block(loop_end);
        Self::unit_value()
    }

    /// Codegens an if expression, handling the case of what to do if there is no 'else'.
    ///
    /// For example, the expression `if cond { a } else { b }` is codegen'd as:
    ///
    ///   v0 = ... codegen cond ...
    ///   brif v0, then: then_block, else: else_block
    /// then_block():
    ///   v1 = ... codegen a ...
    ///   br end_if(v1)
    /// else_block():
    ///   v2 = ... codegen b ...
    ///   br end_if(v2)
    /// end_if(v3: ?):  // Type of v3 matches the type of a and b
    ///   ... This is the current insert point after codegen_if finishes ...
    ///
    /// As another example, the expression `if cond { a }` is codegen'd as:
    ///
    ///   v0 = ... codegen cond ...
    ///   brif v0, then: then_block, else: end_block
    /// then_block:
    ///   v1 = ... codegen a ...
    ///   br end_if()
    /// end_if:  // No block parameter is needed. Without an else, the unit value is always returned.
    ///   ... This is the current insert point after codegen_if finishes ...
    fn codegen_if(&mut self, if_expr: &ast::If) -> Values {
        let condition = self.codegen_non_tuple_expression(&if_expr.condition);

        let then_block = self.builder.insert_block();
        let else_block = self.builder.insert_block();

        self.builder.terminate_with_jmpif(condition, then_block, else_block);

        self.builder.switch_to_block(then_block);
        let then_value = self.codegen_expression(&if_expr.consequence);

        let mut result = Self::unit_value();

        if let Some(alternative) = &if_expr.alternative {
            let end_block = self.builder.insert_block();
            let then_values = then_value.into_value_list(self);
            self.builder.terminate_with_jmp(end_block, then_values);

            self.builder.switch_to_block(else_block);
            let else_value = self.codegen_expression(alternative);
            let else_values = else_value.into_value_list(self);
            self.builder.terminate_with_jmp(end_block, else_values);

            // Create block arguments for the end block as needed to branch to
            // with our then and else value.
            result = Self::map_type(&if_expr.typ, |typ| {
                self.builder.add_block_parameter(end_block, typ).into()
            });

            // Must also set the then block to jmp to the end now
            self.builder.switch_to_block(end_block);
        } else {
            // In the case we have no 'else', the 'else' block is actually the end block.
            self.builder.terminate_with_jmp(else_block, vec![]);
            self.builder.switch_to_block(else_block);
        }

        result
    }

    fn codegen_tuple(&mut self, tuple: &[Expression]) -> Values {
        Tree::Branch(vecmap(tuple, |expr| self.codegen_expression(expr)))
    }

    fn codegen_extract_tuple_field(&mut self, tuple: &Expression, field_index: usize) -> Values {
        let tuple = self.codegen_expression(tuple);
        Self::get_field(tuple, field_index)
    }

    /// Generate SSA for a function call. Note that calls to built-in functions
    /// and intrinsics are also represented by the function call instruction.
    fn codegen_call(&mut self, call: &ast::Call) -> Values {
        let function = self.codegen_non_tuple_expression(&call.func);
        let arguments = call
            .arguments
            .iter()
            .flat_map(|argument| self.codegen_expression(argument).into_value_list(self))
            .collect();

        self.insert_call(function, arguments, &call.return_type, call.location)
    }

    /// Generate SSA for the given variable.
    /// If the variable is immutable, no special handling is necessary and we can return the given
    /// ValueId directly. If it is mutable, we'll need to allocate space for the value and store
    /// the initial value before returning the allocate instruction.
    fn codegen_let(&mut self, let_expr: &ast::Let) -> Values {
        let mut values = self.codegen_expression(&let_expr.expression);

        if let_expr.mutable {
            values = values.map(|value| {
                let value = value.eval(self);
                Tree::Leaf(self.new_mutable_variable(value))
            });
        }

        self.define(let_expr.id, values);
        Self::unit_value()
    }

    fn codegen_constrain(&mut self, expr: &Expression, location: Location) -> Values {
        let boolean = self.codegen_non_tuple_expression(expr);
        self.builder.set_location(location).insert_constrain(boolean);
        Self::unit_value()
    }

    fn codegen_assign(&mut self, assign: &ast::Assign) -> Values {
        let lhs = self.extract_current_value(&assign.lvalue);
        let rhs = self.codegen_expression(&assign.expression);

        self.assign_new_value(lhs, rhs);
        Self::unit_value()
    }

    fn codegen_semi(&mut self, expr: &Expression) -> Values {
        self.codegen_expression(expr);
        Self::unit_value()
    }
}
