pub(crate) mod context;
mod program;
mod value;

use noirc_frontend::token::FmtStrFragment;
pub(crate) use program::Ssa;

use context::SharedContext;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::Location;
use noirc_frontend::ast::{UnaryOp, Visibility};
use noirc_frontend::hir_def::types::Type as HirType;
use noirc_frontend::monomorphization::ast::{self, Expression, Program};

use crate::{
    errors::RuntimeError,
    ssa::{function_builder::data_bus::DataBusBuilder, ir::instruction::Intrinsic},
};

use self::{
    context::FunctionContext,
    value::{Tree, Values},
};

use super::ir::instruction::ErrorType;
use super::ir::types::NumericType;
use super::{
    function_builder::data_bus::DataBus,
    ir::{
        function::RuntimeType,
        instruction::{BinaryOp, ConstrainError, TerminatorInstruction},
        types::Type,
        value::ValueId,
    },
};

pub(crate) const SSA_WORD_SIZE: u32 = 32;

/// Generates SSA for the given monomorphized program.
///
/// This function will generate the SSA but does not perform any optimizations on it.
pub(crate) fn generate_ssa(
    program: Program,
    force_brillig_runtime: bool,
) -> Result<Ssa, RuntimeError> {
    // see which parameter has call_data/return_data attribute
    let is_databus = DataBusBuilder::is_databus(&program.main_function_signature);

    let is_return_data = matches!(program.return_visibility, Visibility::ReturnData);

    let return_location = program.return_location;
    let context = SharedContext::new(program);

    let main_id = Program::main_id();
    let main = context.program.main();

    // Queue the main function for compilation
    context.get_or_queue_function(main_id);
    let mut function_context = FunctionContext::new(
        main.name.clone(),
        &main.parameters,
        if force_brillig_runtime || main.unconstrained {
            RuntimeType::Brillig(main.inline_type)
        } else {
            RuntimeType::Acir(main.inline_type)
        },
        &context,
    );

    // Generate the call_data bus from the relevant parameters. We create it *before* processing the function body
    let call_data = function_context.builder.call_data_bus(is_databus);

    function_context.codegen_function_body(&main.body)?;

    let mut return_data = DataBusBuilder::new();
    if let Some(return_location) = return_location {
        let block = function_context.builder.current_block();
        if function_context.builder.current_function.dfg[block].terminator().is_some()
            && is_return_data
        {
            // initialize the return_data bus from the return values
            let return_data_values =
                match function_context.builder.current_function.dfg[block].unwrap_terminator() {
                    TerminatorInstruction::Return { return_values, .. } => return_values.to_owned(),
                    _ => unreachable!("ICE - expect return on the last block"),
                };

            return_data = function_context.builder.initialize_data_bus(
                &return_data_values,
                return_data,
                None,
            );
        }
        let return_instruction =
            function_context.builder.current_function.dfg[block].unwrap_terminator_mut();
        match return_instruction {
            TerminatorInstruction::Return { return_values, call_stack } => {
                call_stack.clear();
                call_stack.push_back(return_location);
                // replace the returned values with the return data array
                if let Some(return_data_bus) = return_data.databus {
                    return_values.clear();
                    return_values.push(return_data_bus);
                }
            }
            _ => unreachable!("ICE - expect return on the last block"),
        }
    }
    // we save the data bus inside the dfg
    function_context.builder.current_function.dfg.data_bus =
        DataBus::get_data_bus(call_data, return_data);

    // Main has now been compiled and any other functions referenced within have been added to the
    // function queue as they were found in codegen_ident. This queueing will happen each time a
    // previously-unseen function is found so we need now only continue popping from this queue
    // to generate SSA for each function used within the program.
    while let Some((src_function_id, dest_id)) = context.pop_next_function_in_queue() {
        let function = &context.program[src_function_id];
        function_context.new_function(dest_id, function, force_brillig_runtime);
        function_context.codegen_function_body(&function.body)?;
    }

    Ok(function_context.builder.finish())
}

impl<'a> FunctionContext<'a> {
    /// Codegen a function's body and set its return value to that of its last parameter.
    /// For functions returning nothing, this will be an empty list.
    fn codegen_function_body(&mut self, body: &Expression) -> Result<(), RuntimeError> {
        let incremented_params = self.increment_parameter_rcs();
        let return_value = self.codegen_expression(body)?;
        let results = return_value.into_value_list(self);
        self.end_scope(incremented_params, &results);

        self.builder.terminate_with_return(results);
        Ok(())
    }

    fn codegen_expression(&mut self, expr: &Expression) -> Result<Values, RuntimeError> {
        match expr {
            Expression::Ident(ident) => Ok(self.codegen_ident(ident)),
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
            Expression::Constrain(expr, location, assert_payload) => {
                self.codegen_constrain(expr, *location, assert_payload)
            }
            Expression::Assign(assign) => self.codegen_assign(assign),
            Expression::Semi(semi) => self.codegen_semi(semi),
            Expression::Break => Ok(self.codegen_break()),
            Expression::Continue => Ok(self.codegen_continue()),
        }
    }

    /// Codegen any non-tuple expression so that we can unwrap the Values
    /// tree to return a single value for use with most SSA instructions.
    fn codegen_non_tuple_expression(&mut self, expr: &Expression) -> Result<ValueId, RuntimeError> {
        Ok(self.codegen_expression(expr)?.into_leaf().eval(self))
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

    fn codegen_literal(&mut self, literal: &ast::Literal) -> Result<Values, RuntimeError> {
        match literal {
            ast::Literal::Array(array) => {
                let elements = self.codegen_array_elements(&array.contents)?;

                let typ = Self::convert_type(&array.typ).flatten();
                Ok(match array.typ {
                    ast::Type::Array(_, _) => {
                        self.codegen_array_checked(elements, typ[0].clone())?
                    }
                    _ => unreachable!("ICE: unexpected array literal type, got {}", array.typ),
                })
            }
            ast::Literal::Slice(array) => {
                let elements = self.codegen_array_elements(&array.contents)?;

                let typ = Self::convert_type(&array.typ).flatten();
                Ok(match array.typ {
                    ast::Type::Slice(_) => {
                        let slice_length =
                            self.builder.length_constant(array.contents.len() as u128);
                        let slice_contents =
                            self.codegen_array_checked(elements, typ[1].clone())?;
                        Tree::Branch(vec![slice_length.into(), slice_contents])
                    }
                    _ => unreachable!("ICE: unexpected slice literal type, got {}", array.typ),
                })
            }
            ast::Literal::Integer(value, negative, typ, location) => {
                self.builder.set_location(*location);
                let typ = Self::convert_non_tuple_type(typ).unwrap_numeric();
                self.checked_numeric_constant(*value, *negative, typ).map(Into::into)
            }
            ast::Literal::Bool(value) => {
                // Don't need to call checked_numeric_constant here since `value` can only be true or false
                Ok(self.builder.numeric_constant(*value as u128, NumericType::bool()).into())
            }
            ast::Literal::Str(string) => Ok(self.codegen_string(string)),
            ast::Literal::FmtStr(fragments, number_of_fields, fields) => {
                let mut string = String::new();
                for fragment in fragments {
                    match fragment {
                        FmtStrFragment::String(value) => {
                            // Escape curly braces in non-interpolations
                            let value = value.replace('{', "{{").replace('}', "}}");
                            string.push_str(&value);
                        }
                        FmtStrFragment::Interpolation(value, _span) => {
                            string.push('{');
                            string.push_str(value);
                            string.push('}');
                        }
                    }
                }

                // A caller needs multiple pieces of information to make use of a format string
                // The message string, the number of fields to be formatted, and the fields themselves
                let string = self.codegen_string(&string);
                let field_count = self.builder.length_constant(*number_of_fields as u128);
                let fields = self.codegen_expression(fields)?;

                Ok(Tree::Branch(vec![string, field_count.into(), fields]))
            }
            ast::Literal::Unit => Ok(Self::unit_value()),
        }
    }

    fn codegen_array_elements(
        &mut self,
        elements: &[Expression],
    ) -> Result<Vec<(Values, bool)>, RuntimeError> {
        try_vecmap(elements, |element| {
            let value = self.codegen_expression(element)?;
            Ok((value, element.is_array_or_slice_literal()))
        })
    }

    fn codegen_string(&mut self, string: &str) -> Values {
        let elements = vecmap(string.as_bytes(), |byte| {
            let char = self.builder.numeric_constant(*byte as u128, NumericType::char());
            (char.into(), false)
        });
        let typ = Self::convert_non_tuple_type(&ast::Type::String(elements.len() as u32));
        self.codegen_array(elements, typ)
    }

    // Codegen an array but make sure that we do not have a nested slice
    ///
    /// The bool aspect of each array element indicates whether the element is an array constant
    /// or not. If it is, we avoid incrementing the reference count because we consider the
    /// constant to be moved into this larger array constant.
    fn codegen_array_checked(
        &mut self,
        elements: Vec<(Values, bool)>,
        typ: Type,
    ) -> Result<Values, RuntimeError> {
        if typ.is_nested_slice() {
            return Err(RuntimeError::NestedSlice { call_stack: self.builder.get_call_stack() });
        }
        Ok(self.codegen_array(elements, typ))
    }

    /// Codegen an array by allocating enough space for each element and inserting separate
    /// store instructions until each element is stored. The store instructions will be separated
    /// by add instructions to calculate the new offset address to store to next.
    ///
    /// In the case of arrays of structs, the structs are flattened such that each field will be
    /// stored next to the other fields in memory. So an array such as [(1, 2), (3, 4)] is
    /// stored the same as the array [1, 2, 3, 4].
    ///
    /// The bool aspect of each array element indicates whether the element is an array constant
    /// or not. If it is, we avoid incrementing the reference count because we consider the
    /// constant to be moved into this larger array constant.
    ///
    /// The value returned from this function is always that of the allocate instruction.
    fn codegen_array(&mut self, elements: Vec<(Values, bool)>, typ: Type) -> Values {
        let mut array = im::Vector::new();

        for (element, is_array_constant) in elements {
            element.for_each(|element| {
                let element = element.eval(self);

                // If we're referencing a sub-array in a larger nested array we need to
                // increase the reference count of the sub array. This maintains a
                // pessimistic reference count (since some are likely moved rather than shared)
                // which is important for Brillig's copy on write optimization. This has no
                // effect in ACIR code.
                if !is_array_constant {
                    self.builder.increment_array_reference_count(element);
                }

                array.push_back(element);
            });
        }

        self.builder.insert_make_array(array, typ).into()
    }

    fn codegen_block(&mut self, block: &[Expression]) -> Result<Values, RuntimeError> {
        let mut result = Self::unit_value();
        for expr in block {
            result = self.codegen_expression(expr)?;
        }
        Ok(result)
    }

    fn codegen_unary(&mut self, unary: &ast::Unary) -> Result<Values, RuntimeError> {
        match unary.operator {
            UnaryOp::Not => {
                let rhs = self.codegen_expression(&unary.rhs)?;
                let rhs = rhs.into_leaf().eval(self);
                Ok(self.builder.insert_not(rhs).into())
            }
            UnaryOp::Minus => {
                let rhs = self.codegen_expression(&unary.rhs)?;
                let rhs = rhs.into_leaf().eval(self);
                let typ = self.builder.type_of_value(rhs).unwrap_numeric();
                let zero = self.builder.numeric_constant(0u128, typ);
                Ok(self.insert_binary(
                    zero,
                    noirc_frontend::ast::BinaryOpKind::Subtract,
                    rhs,
                    unary.location,
                ))
            }
            UnaryOp::MutableReference => {
                Ok(self.codegen_reference(&unary.rhs)?.map(|rhs| {
                    match rhs {
                        value::Value::Normal(value) => {
                            let rhs_type = self.builder.current_function.dfg.type_of_value(value);
                            let alloc = self.builder.insert_allocate(rhs_type);
                            self.builder.insert_store(alloc, value);
                            Tree::Leaf(value::Value::Normal(alloc))
                        }
                        // The `.into()` here converts the Value::Mutable into
                        // a Value::Normal so it is no longer automatically dereferenced.
                        value::Value::Mutable(reference, _) => reference.into(),
                    }
                }))
            }
            UnaryOp::Dereference { .. } => {
                let rhs = self.codegen_expression(&unary.rhs)?;
                Ok(self.dereference(&rhs, &unary.result_type))
            }
        }
    }

    fn dereference(&mut self, values: &Values, element_type: &ast::Type) -> Values {
        let element_types = Self::convert_type(element_type);
        values.map_both(element_types, |value, element_type| {
            let reference = value.eval(self);
            self.builder.insert_load(reference, element_type).into()
        })
    }

    fn codegen_reference(&mut self, expr: &Expression) -> Result<Values, RuntimeError> {
        match expr {
            Expression::Ident(ident) => Ok(self.codegen_ident_reference(ident)),
            Expression::ExtractTupleField(tuple, index) => {
                let tuple = self.codegen_reference(tuple)?;
                Ok(Self::get_field(tuple, *index))
            }
            other => self.codegen_expression(other),
        }
    }

    fn codegen_binary(&mut self, binary: &ast::Binary) -> Result<Values, RuntimeError> {
        let lhs = self.codegen_non_tuple_expression(&binary.lhs)?;
        let rhs = self.codegen_non_tuple_expression(&binary.rhs)?;
        Ok(self.insert_binary(lhs, binary.operator, rhs, binary.location))
    }

    fn codegen_index(&mut self, index: &ast::Index) -> Result<Values, RuntimeError> {
        let array_or_slice = self.codegen_expression(&index.collection)?.into_value_list(self);
        let index_value = self.codegen_non_tuple_expression(&index.index)?;
        // Slices are represented as a tuple in the form: (length, slice contents).
        // Thus, slices require two value ids for their representation.
        let (array, slice_length) = if array_or_slice.len() > 1 {
            (array_or_slice[1], Some(array_or_slice[0]))
        } else {
            (array_or_slice[0], None)
        };

        self.codegen_array_index(
            array,
            index_value,
            &index.element_type,
            index.location,
            slice_length,
        )
    }

    /// This is broken off from codegen_index so that it can also be
    /// used to codegen a LValue::Index.
    ///
    /// Set load_result to true to load from each relevant index of the array
    /// (it may be multiple in the case of tuples). Set it to false to instead
    /// return a reference to each element, for use with the store instruction.
    fn codegen_array_index(
        &mut self,
        array: ValueId,
        index: ValueId,
        element_type: &ast::Type,
        location: Location,
        length: Option<ValueId>,
    ) -> Result<Values, RuntimeError> {
        // base_index = index * type_size
        let index = self.make_array_index(index);
        let type_size = Self::convert_type(element_type).size_of_type();
        let type_size =
            self.builder.numeric_constant(type_size as u128, NumericType::length_type());
        let base_index =
            self.builder.set_location(location).insert_binary(index, BinaryOp::Mul, type_size);

        let mut field_index = 0u128;
        Ok(Self::map_type(element_type, |typ| {
            let offset = self.make_offset(base_index, field_index);
            field_index += 1;

            let array_type = &self.builder.type_of_value(array);
            match array_type {
                Type::Slice(_) => {
                    self.codegen_slice_access_check(index, length);
                }
                Type::Array(..) => {
                    // Nothing needs to done to prepare an array access on an array
                }
                _ => unreachable!("must have array or slice but got {array_type}"),
            }

            // Reference counting in brillig relies on us incrementing reference
            // counts when nested arrays/slices are constructed or indexed. This
            // has no effect in ACIR code.
            let result = self.builder.insert_array_get(array, offset, typ);
            self.builder.increment_array_reference_count(result);
            result.into()
        }))
    }

    /// Prepare a slice access.
    /// Check that the index being used to access a slice element
    /// is less than the dynamic slice length.
    fn codegen_slice_access_check(&mut self, index: ValueId, length: Option<ValueId>) {
        let index = self.make_array_index(index);
        // We convert the length as an array index type for comparison
        let array_len = self
            .make_array_index(length.expect("ICE: a length must be supplied for indexing slices"));

        let is_offset_out_of_bounds = self.builder.insert_binary(index, BinaryOp::Lt, array_len);
        let true_const = self.builder.numeric_constant(true, NumericType::bool());

        self.builder.insert_constrain(
            is_offset_out_of_bounds,
            true_const,
            Some("Index out of bounds".to_owned().into()),
        );
    }

    fn codegen_cast(&mut self, cast: &ast::Cast) -> Result<Values, RuntimeError> {
        let lhs = self.codegen_non_tuple_expression(&cast.lhs)?;
        let typ = Self::convert_non_tuple_type(&cast.r#type).unwrap_numeric();

        Ok(self.insert_safe_cast(lhs, typ, cast.location).into())
    }

    /// Codegens a for loop, creating three new blocks in the process.
    /// The return value of a for loop is always a unit literal.
    ///
    /// For example, the loop `for i in start .. end { body }` is codegen'd as:
    ///
    /// ```text
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
    /// ```
    fn codegen_for(&mut self, for_expr: &ast::For) -> Result<Values, RuntimeError> {
        let loop_entry = self.builder.insert_block();
        let loop_body = self.builder.insert_block();
        let loop_end = self.builder.insert_block();

        // this is the 'i' in `for i in start .. end { block }`
        let index_type = Self::convert_non_tuple_type(&for_expr.index_type);
        let loop_index = self.builder.add_block_parameter(loop_entry, index_type);

        // Remember the blocks and variable used in case there are break/continue instructions
        // within the loop which need to jump to them.
        self.enter_loop(loop_entry, loop_index, loop_end);

        self.builder.set_location(for_expr.start_range_location);
        let start_index = self.codegen_non_tuple_expression(&for_expr.start_range)?;

        self.builder.set_location(for_expr.end_range_location);
        let end_index = self.codegen_non_tuple_expression(&for_expr.end_range)?;

        // Set the location of the initial jmp instruction to the start range. This is the location
        // used to issue an error if the start range cannot be determined at compile-time.
        self.builder.set_location(for_expr.start_range_location);
        self.builder.terminate_with_jmp(loop_entry, vec![start_index]);

        // Compile the loop entry block
        self.builder.switch_to_block(loop_entry);

        // Set the location of the ending Lt instruction and the jmpif back-edge of the loop to the
        // end range. These are the instructions used to issue an error if the end of the range
        // cannot be determined at compile-time.
        self.builder.set_location(for_expr.end_range_location);
        let jump_condition = self.builder.insert_binary(loop_index, BinaryOp::Lt, end_index);
        self.builder.terminate_with_jmpif(jump_condition, loop_body, loop_end);

        // Compile the loop body
        self.builder.switch_to_block(loop_body);
        self.define(for_expr.index_variable, loop_index.into());
        self.codegen_expression(&for_expr.block)?;
        let new_loop_index = self.make_offset(loop_index, 1);
        self.builder.terminate_with_jmp(loop_entry, vec![new_loop_index]);

        // Finish by switching back to the end of the loop
        self.builder.switch_to_block(loop_end);
        self.exit_loop();
        Ok(Self::unit_value())
    }

    /// Codegens an if expression, handling the case of what to do if there is no 'else'.
    ///
    /// For example, the expression `if cond { a } else { b }` is codegen'd as:
    ///
    /// ```text
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
    /// ```
    ///
    /// As another example, the expression `if cond { a }` is codegen'd as:
    ///
    /// ```text
    ///   v0 = ... codegen cond ...
    ///   brif v0, then: then_block, else: end_if
    /// then_block:
    ///   v1 = ... codegen a ...
    ///   br end_if()
    /// end_if:  // No block parameter is needed. Without an else, the unit value is always returned.
    ///   ... This is the current insert point after codegen_if finishes ...
    /// ```
    fn codegen_if(&mut self, if_expr: &ast::If) -> Result<Values, RuntimeError> {
        let condition = self.codegen_non_tuple_expression(&if_expr.condition)?;

        let then_block = self.builder.insert_block();
        let else_block = self.builder.insert_block();

        self.builder.terminate_with_jmpif(condition, then_block, else_block);

        self.builder.switch_to_block(then_block);
        let then_value = self.codegen_expression(&if_expr.consequence)?;

        let mut result = Self::unit_value();

        if let Some(alternative) = &if_expr.alternative {
            let end_block = self.builder.insert_block();
            let then_values = then_value.into_value_list(self);
            self.builder.terminate_with_jmp(end_block, then_values);

            self.builder.switch_to_block(else_block);
            let else_value = self.codegen_expression(alternative)?;
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

        Ok(result)
    }

    fn codegen_tuple(&mut self, tuple: &[Expression]) -> Result<Values, RuntimeError> {
        Ok(Tree::Branch(try_vecmap(tuple, |expr| self.codegen_expression(expr))?))
    }

    fn codegen_extract_tuple_field(
        &mut self,
        tuple: &Expression,
        field_index: usize,
    ) -> Result<Values, RuntimeError> {
        let tuple = self.codegen_expression(tuple)?;
        Ok(Self::get_field(tuple, field_index))
    }

    /// Generate SSA for a function call. Note that calls to built-in functions
    /// and intrinsics are also represented by the function call instruction.
    fn codegen_call(&mut self, call: &ast::Call) -> Result<Values, RuntimeError> {
        let function = self.codegen_non_tuple_expression(&call.func)?;
        let mut arguments = Vec::with_capacity(call.arguments.len());

        for argument in &call.arguments {
            let mut values = self.codegen_expression(argument)?.into_value_list(self);
            arguments.append(&mut values);
        }

        // Don't need to increment array reference counts when passed in as arguments
        // since it is done within the function to each parameter already.

        self.codegen_intrinsic_call_checks(function, &arguments, call.location);
        Ok(self.insert_call(function, arguments, &call.return_type, call.location))
    }

    fn codegen_intrinsic_call_checks(
        &mut self,
        function: ValueId,
        arguments: &[ValueId],
        location: Location,
    ) {
        if let Some(intrinsic) =
            self.builder.set_location(location).get_intrinsic_from_value(function)
        {
            match intrinsic {
                Intrinsic::SliceInsert => {
                    let one = self.builder.length_constant(1u128);

                    // We add one here in the case of a slice insert as a slice insert at the length of the slice
                    // can be converted to a slice push back
                    let len_plus_one = self.builder.insert_binary(arguments[0], BinaryOp::Add, one);

                    self.codegen_slice_access_check(arguments[2], Some(len_plus_one));
                }
                Intrinsic::SliceRemove => {
                    self.codegen_slice_access_check(arguments[2], Some(arguments[0]));
                }
                _ => {
                    // Do nothing as the other intrinsics do not require checks
                }
            }
        }
    }

    /// Generate SSA for the given variable.
    /// If the variable is immutable, no special handling is necessary and we can return the given
    /// ValueId directly. If it is mutable, we'll need to allocate space for the value and store
    /// the initial value before returning the allocate instruction.
    fn codegen_let(&mut self, let_expr: &ast::Let) -> Result<Values, RuntimeError> {
        let mut values = self.codegen_expression(&let_expr.expression)?;

        // Don't mutate the reference count if we're assigning an array literal to a Let:
        // `let mut foo = [1, 2, 3];`
        // we consider the array to be moved, so we should have an initial rc of just 1.
        let should_inc_rc = !let_expr.expression.is_array_or_slice_literal();

        values = values.map(|value| {
            let value = value.eval(self);

            Tree::Leaf(if let_expr.mutable {
                self.new_mutable_variable(value, should_inc_rc)
            } else {
                // `new_mutable_variable` increments rcs internally so we have to
                // handle it separately for the immutable case
                if should_inc_rc {
                    self.builder.increment_array_reference_count(value);
                }
                value::Value::Normal(value)
            })
        });

        self.define(let_expr.id, values);
        Ok(Self::unit_value())
    }

    fn codegen_constrain(
        &mut self,
        expr: &Expression,
        location: Location,
        assert_payload: &Option<Box<(Expression, HirType)>>,
    ) -> Result<Values, RuntimeError> {
        let expr = self.codegen_non_tuple_expression(expr)?;
        let true_literal = self.builder.numeric_constant(true, NumericType::bool());

        // Set the location here for any errors that may occur when we codegen the assert message
        self.builder.set_location(location);

        let assert_payload = self.codegen_constrain_error(assert_payload)?;

        self.builder.insert_constrain(expr, true_literal, assert_payload);

        Ok(Self::unit_value())
    }

    // This method does not necessary codegen the full assert message expression, thus it does not
    // return a `Values` object. Instead we check the internals of the expression to make sure
    // we have an `Expression::Call` as expected. An `Instruction::Call` is then constructed but not
    // inserted to the SSA as we want that instruction to be atomic in SSA with a constrain instruction.
    fn codegen_constrain_error(
        &mut self,
        assert_message: &Option<Box<(Expression, HirType)>>,
    ) -> Result<Option<ConstrainError>, RuntimeError> {
        let Some(assert_message_payload) = assert_message else { return Ok(None) };
        let (assert_message_expression, assert_message_typ) = assert_message_payload.as_ref();

        if let Expression::Literal(ast::Literal::Str(static_string)) = assert_message_expression {
            Ok(Some(ConstrainError::StaticString(static_string.clone())))
        } else {
            let error_type = ErrorType::Dynamic(assert_message_typ.clone());
            let selector = error_type.selector();
            let values = self.codegen_expression(assert_message_expression)?.into_value_list(self);
            let is_string_type = matches!(assert_message_typ, HirType::String(_));
            // Record custom types in the builder, outside of SSA instructions
            // This is made to avoid having Hir types in the SSA code.
            if !is_string_type {
                self.builder.record_error_type(selector, assert_message_typ.clone());
            }

            Ok(Some(ConstrainError::Dynamic(selector, is_string_type, values)))
        }
    }

    fn codegen_assign(&mut self, assign: &ast::Assign) -> Result<Values, RuntimeError> {
        let lhs = self.extract_current_value(&assign.lvalue)?;
        let rhs = self.codegen_expression(&assign.expression)?;
        let should_inc_rc = !assign.expression.is_array_or_slice_literal();

        rhs.clone().for_each(|value| {
            let value = value.eval(self);

            if should_inc_rc {
                self.builder.increment_array_reference_count(value);
            }
        });

        self.assign_new_value(lhs, rhs);
        Ok(Self::unit_value())
    }

    fn codegen_semi(&mut self, expr: &Expression) -> Result<Values, RuntimeError> {
        self.codegen_expression(expr)?;
        Ok(Self::unit_value())
    }

    fn codegen_break(&mut self) -> Values {
        let loop_end = self.current_loop().loop_end;
        self.builder.terminate_with_jmp(loop_end, Vec::new());
        Self::unit_value()
    }

    fn codegen_continue(&mut self) -> Values {
        let loop_ = self.current_loop();

        // Must remember to increment i before jumping
        let new_loop_index = self.make_offset(loop_.loop_index, 1);
        self.builder.terminate_with_jmp(loop_.loop_entry, vec![new_loop_index]);
        Self::unit_value()
    }
}
