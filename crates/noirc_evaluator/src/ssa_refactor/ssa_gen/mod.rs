mod context;
mod value;

use acvm::FieldElement;
use context::SharedContext;
use iter_extended::vecmap;
use noirc_errors::Location;
use noirc_frontend::monomorphization::ast::{self, Expression, Program};

use self::{
    context::FunctionContext,
    value::{Tree, Values},
};

use super::{
    ir::{instruction::BinaryOp, types::Type, value::ValueId},
    ssa_builder::SharedBuilderContext,
};

pub(crate) fn generate_ssa(program: Program) {
    let context = SharedContext::new(program);
    let builder_context = SharedBuilderContext::default();

    let main = context.program.main();
    let mut function_context =
        FunctionContext::new(main.name.clone(), &main.parameters, &context, &builder_context);

    function_context.codegen_expression(&main.body);

    while let Some((src_function_id, _new_id)) = context.pop_next_function_in_queue() {
        let function = &context.program[src_function_id];
        // TODO: Need to ensure/assert the new function's id == new_id
        function_context.new_function(function.name.clone(), &function.parameters);
        function_context.codegen_expression(&function.body);
    }
}

impl<'a> FunctionContext<'a> {
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
        match self.codegen_expression(expr) {
            Tree::Branch(branches) => {
                panic!("codegen_non_tuple_expression called on tuple {branches:?}")
            }
            Tree::Leaf(value) => value.eval(),
        }
    }

    fn codegen_ident(&mut self, _ident: &ast::Ident) -> Values {
        todo!()
    }

    fn codegen_literal(&mut self, literal: &ast::Literal) -> Values {
        match literal {
            ast::Literal::Array(array) => {
                let elements = vecmap(&array.contents, |element| self.codegen_expression(element));
                let element_type = Self::convert_type(&array.element_type);
                self.codegen_array(elements, element_type)
            }
            ast::Literal::Integer(value, typ) => {
                let typ = Self::convert_non_tuple_type(typ);
                self.builder.numeric_constant(*value, typ).into()
            }
            ast::Literal::Bool(value) => {
                // Booleans are represented as u1s with 0 = false, 1 = true
                let typ = Type::unsigned(1);
                let value = FieldElement::from(*value as u128);
                self.builder.numeric_constant(value, typ).into()
            }
            ast::Literal::Str(string) => {
                let elements = vecmap(string.as_bytes(), |byte| {
                    let value = FieldElement::from(*byte as u128);
                    self.builder.numeric_constant(value, Type::field()).into()
                });
                self.codegen_array(elements, Tree::Leaf(Type::field()))
            }
        }
    }

    fn codegen_array(&mut self, elements: Vec<Values>, element_type: Tree<Type>) -> Values {
        let size = element_type.size_of_type() * elements.len();
        let array = self.builder.insert_allocate(size.try_into().unwrap_or_else(|_| {
            panic!("Cannot allocate {size} bytes for array, it does not fit into a u32")
        }));

        // Now we must manually store all the elements into the array
        let mut i = 0;
        for element in elements {
            element.for_each(|value| {
                let address = if i == 0 {
                    array
                } else {
                    let offset = self.builder.field_constant(i as u128);
                    self.builder.insert_binary(array, BinaryOp::Add, offset)
                };
                self.builder.insert_store(address, value.eval());
                i += 1;
            });
        }

        array.into()
    }

    fn codegen_block(&mut self, block: &[Expression]) -> Values {
        let mut result = self.unit_value();
        for expr in block {
            result = self.codegen_expression(expr);
        }
        result
    }

    fn codegen_unary(&mut self, _unary: &ast::Unary) -> Values {
        todo!()
    }

    fn codegen_binary(&mut self, binary: &ast::Binary) -> Values {
        let lhs = self.codegen_non_tuple_expression(&binary.lhs);
        let rhs = self.codegen_non_tuple_expression(&binary.rhs);
        self.insert_binary(lhs, binary.operator, rhs)
    }

    fn codegen_index(&mut self, _index: &ast::Index) -> Values {
        todo!()
    }

    fn codegen_cast(&mut self, _cast: &ast::Cast) -> Values {
        todo!()
    }

    fn codegen_for(&mut self, _for_expr: &ast::For) -> Values {
        todo!()
    }

    fn codegen_if(&mut self, _if_expr: &ast::If) -> Values {
        todo!()
    }

    fn codegen_tuple(&mut self, tuple: &[Expression]) -> Values {
        Tree::Branch(vecmap(tuple, |expr| self.codegen_expression(expr)))
    }

    fn codegen_extract_tuple_field(&mut self, tuple: &Expression, index: usize) -> Values {
        match self.codegen_expression(tuple) {
            Tree::Branch(mut trees) => trees.remove(index),
            Tree::Leaf(value) => {
                unreachable!("Tried to extract tuple index {index} from non-tuple {value:?}")
            }
        }
    }

    fn codegen_call(&mut self, _call: &ast::Call) -> Values {
        todo!()
    }

    fn codegen_let(&mut self, _let_expr: &ast::Let) -> Values {
        todo!()
    }

    fn codegen_constrain(&mut self, _constrain: &Expression, _location: Location) -> Values {
        todo!()
    }

    fn codegen_assign(&mut self, _assign: &ast::Assign) -> Values {
        todo!()
    }

    fn codegen_semi(&mut self, expr: &Expression) -> Values {
        self.codegen_expression(expr);
        self.unit_value()
    }
}
