mod context;
mod value;

use context::Context;
use noirc_errors::Location;
use noirc_frontend::monomorphization::ast::{self, Expression, Program};

use self::{context::FunctionContext, value::Value};

use super::ssa_builder::Builder;

pub(crate) fn generate_ssa(program: Program) {
    let context = Context::new(program);
    let builder_context = Builder::default();

    let main = context.program.main();
    // TODO struct parameter counting
    let parameter_count = main.parameters.len();

    let mut function_context = FunctionContext::new(parameter_count, &context, &builder_context);
    function_context.codegen_expression(&main.body);

    while let Some((src_function_id, _new_id)) = context.pop_next_function_in_queue() {
        let function = &context.program[src_function_id];
        // TODO: Need to ensure/assert the new function's id == new_id
        function_context.new_function(function.parameters.iter().map(|(id, ..)| *id));
        function_context.codegen_expression(&function.body);
    }
}

impl<'a> FunctionContext<'a> {
    fn codegen_expression(&mut self, expr: &Expression) -> Value {
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

    fn codegen_ident(&mut self, _ident: &ast::Ident) -> Value {
        todo!()
    }

    fn codegen_literal(&mut self, _literal: &ast::Literal) -> Value {
        todo!()
    }

    fn codegen_block(&mut self, _block: &[Expression]) -> Value {
        todo!()
    }

    fn codegen_unary(&mut self, _unary: &ast::Unary) -> Value {
        todo!()
    }

    fn codegen_binary(&mut self, _binary: &ast::Binary) -> Value {
        todo!()
    }

    fn codegen_index(&mut self, _index: &ast::Index) -> Value {
        todo!()
    }

    fn codegen_cast(&mut self, _cast: &ast::Cast) -> Value {
        todo!()
    }

    fn codegen_for(&mut self, _for_expr: &ast::For) -> Value {
        todo!()
    }

    fn codegen_if(&mut self, _if_expr: &ast::If) -> Value {
        todo!()
    }

    fn codegen_tuple(&mut self, _tuple: &[Expression]) -> Value {
        todo!()
    }

    fn codegen_extract_tuple_field(&mut self, _tuple: &Expression, _index: usize) -> Value {
        todo!()
    }

    fn codegen_call(&mut self, _call: &ast::Call) -> Value {
        todo!()
    }

    fn codegen_let(&mut self, _let_expr: &ast::Let) -> Value {
        todo!()
    }

    fn codegen_constrain(&mut self, _constrain: &Expression, _location: Location) -> Value {
        todo!()
    }

    fn codegen_assign(&mut self, _assign: &ast::Assign) -> Value {
        todo!()
    }

    fn codegen_semi(&mut self, _semi: &Expression) -> Value {
        todo!()
    }
}
