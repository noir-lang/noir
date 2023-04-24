mod context;
mod value;

use context::SharedContext;
use noirc_errors::Location;
use noirc_frontend::monomorphization::ast::{self, Expression, Program};

use self::{context::FunctionContext, value::Values};

use super::ssa_builder::SharedBuilderContext;

pub(crate) fn generate_ssa(program: Program) {
    let context = SharedContext::new(program);
    let builder_context = SharedBuilderContext::default();

    let main = context.program.main();

    let mut function_context = FunctionContext::new(&main.parameters, &context, &builder_context);
    function_context.codegen_expression(&main.body);

    while let Some((src_function_id, _new_id)) = context.pop_next_function_in_queue() {
        let function = &context.program[src_function_id];
        // TODO: Need to ensure/assert the new function's id == new_id
        function_context.new_function(&function.parameters);
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

    fn codegen_ident(&mut self, _ident: &ast::Ident) -> Values {
        todo!()
    }

    fn codegen_literal(&mut self, _literal: &ast::Literal) -> Values {
        todo!()
    }

    fn codegen_block(&mut self, _block: &[Expression]) -> Values {
        todo!()
    }

    fn codegen_unary(&mut self, _unary: &ast::Unary) -> Values {
        todo!()
    }

    fn codegen_binary(&mut self, _binary: &ast::Binary) -> Values {
        todo!()
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

    fn codegen_tuple(&mut self, _tuple: &[Expression]) -> Values {
        todo!()
    }

    fn codegen_extract_tuple_field(&mut self, _tuple: &Expression, _index: usize) -> Values {
        todo!()
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

    fn codegen_semi(&mut self, _semi: &Expression) -> Values {
        todo!()
    }
}
