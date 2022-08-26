use std::fmt::Formatter;

use crate::util::vecmap;

use super::ast::{Expression, Function, LValue};

#[derive(Default)]
pub struct AstPrinter {
    indent_level: u32,
}

impl AstPrinter {
    pub fn print_function(&mut self, function: &Function, f: &mut Formatter) -> std::fmt::Result {
        let params = vecmap(&function.parameters, |(id, typ, name)| {
            format!("{}${}: {}",name, id.0, typ)
        }).join(", ");

        write!(f, "fn {}${}({}) -> {} {{", function.name, function.id.0, params, function.return_type)?;
        self.indent_level += 1;
        self.print_expr_expect_block(&function.body, f)?;
        self.indent_level -= 1;
        self.next_line(f)?;
        write!(f, "}}\n")
    }

    pub fn print_expr(&mut self, expr: &Expression, f: &mut Formatter) -> std::fmt::Result {
        match expr {
            Expression::Ident(ident) => write!(f, "{}${}", ident.name, ident.id.0),
            Expression::Literal(literal) => self.print_literal(literal, f),
            Expression::Block(exprs) => self.print_block(exprs, f),
            Expression::Unary(unary) => self.print_unary(unary, f),
            Expression::Binary(binary) => self.print_binary(binary, f),
            Expression::Index(index) => {
                self.print_expr(&index.collection, f)?;
                write!(f, "[")?;
                self.print_expr(&index.index, f)?;
                write!(f, "]")
            }
            Expression::Cast(cast) => {
                write!(f, "(")?;
                self.print_expr(&cast.lhs, f)?;
                write!(f, " as {})", cast.r#type)
            }
            Expression::For(for_expr) => self.print_for(for_expr, f),
            Expression::If(if_expr) => self.print_if(if_expr, f),
            Expression::Tuple(tuple) => self.print_tuple(tuple, f),
            Expression::ExtractTupleField(expr, index) => {
                self.print_expr(expr, f)?;
                write!(f, ".{}", index)
            }
            Expression::Call(call) => self.print_call(call, f),
            Expression::CallBuiltin(call) => self.print_lowlevel(call, f),
            Expression::CallLowLevel(call) => self.print_builtin(call, f),
            Expression::Let(let_expr) => {
                write!(f, "let {}${} = ", let_expr.name, let_expr.id.0)?;
                self.print_expr(&let_expr.expression, f)
            }
            Expression::Constrain(expr, _) => {
                write!(f, "constrain ")?;
                self.print_expr(expr, f)
            }
            Expression::Assign(assign) => {
                self.print_lvalue(&assign.lvalue, f)?;
                write!(f, " = ")?;
                self.print_expr(&assign.expression, f)
            },
            Expression::Semi(expr) => {
                self.print_expr(expr, f)?;
                write!(f, ";")
            },
        }
    }

    fn next_line(&mut self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "\n")?;
        for _ in 0 .. self.indent_level {
            write!(f, "    ")?;
        }
        Ok(())
    }

    fn print_literal(&mut self, literal: &super::ast::Literal, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match literal {
            super::ast::Literal::Array(array) => {
                write!(f, "[")?;
                self.print_comma_separated(&array.contents, f)?;
                write!(f, "]")
            },
            super::ast::Literal::Integer(x, _) => write!(f, "{}", x),
            super::ast::Literal::Bool(x) => write!(f, "{}", x),
            super::ast::Literal::Str(s) => write!(f, "{}", s),
        }
    }

    fn print_block(&mut self, exprs: &[Expression], f: &mut Formatter) -> Result<(), std::fmt::Error> {
        if exprs.is_empty() {
            write!(f, "{{}}")
        } else {
            write!(f, "{{")?;
            self.indent_level += 1;
            for (i, expr) in exprs.iter().enumerate() {
                self.next_line(f)?;
                self.print_expr(expr, f)?;

                if i != exprs.len() - 1 {
                    write!(f, ";")?;
                }
            }
            self.indent_level -= 1;
            self.next_line(f)?;
            write!(f, "}}")
        }
    }

    /// Print an expression, but expect that we've already printed a {} block, so don't print
    /// out those twice. Also decrements the current indent level and prints out the next line when
    /// finished.
    fn print_expr_expect_block(&mut self, expr: &Expression, f: &mut Formatter) -> std::fmt::Result {
        match expr {
            Expression::Block(exprs) => {
                for (i, expr) in exprs.iter().enumerate() {
                    self.next_line(f)?;
                    self.print_expr(expr, f)?;

                    if i != exprs.len() - 1 {
                        write!(f, ";")?;
                    }
                }
                Ok(())
            },
            other => {
                self.next_line(f)?;
                self.print_expr(other, f)
            },
        }
    }

    fn print_unary(&mut self, unary: &super::ast::Unary, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "({}", unary.operator)?;
        self.print_expr(&unary.rhs, f)?;
        write!(f, ")")
    }

    fn print_binary(&mut self, binary: &super::ast::Binary, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "(")?;
        self.print_expr(&binary.lhs, f)?;
        write!(f, " {} ", binary.operator)?;
        self.print_expr(&binary.rhs, f)?;
        write!(f, ")")
    }

    fn print_for(&mut self, for_expr: &super::ast::For, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "for {}${} in ", for_expr.index_name, for_expr.index_variable.0)?;
        self.print_expr(&for_expr.start_range, f)?;
        write!(f, " .. ")?;
        self.print_expr(&for_expr.end_range, f)?;
        write!(f, " {{")?;

        self.indent_level += 1;
        self.print_expr_expect_block(&for_expr.block, f)?;
        self.indent_level -= 1;
        self.next_line(f)?;
        write!(f, "}}")
    }

    fn print_if(&mut self, if_expr: &super::ast::If, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "if ")?;
        self.print_expr(&if_expr.condition, f)?;

        write!(f, " {{")?;
        self.indent_level += 1;
        self.print_expr_expect_block(&if_expr.consequence, f)?;
        self.indent_level -= 1;
        self.next_line(f)?;

        if let Some(alt) = &if_expr.alternative {
            write!(f, " else {{")?;
            self.indent_level += 1;
            self.print_expr_expect_block(&alt, f)?;
            self.indent_level -= 1;
            self.next_line(f)?;
        }
        Ok(())
    }

    fn print_comma_separated(&mut self, exprs: &[Expression], f: &mut Formatter) -> std::fmt::Result {
        for (i, elem) in exprs.iter().enumerate() {
            self.print_expr(elem, f)?;
            if i != exprs.len() - 1 {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }

    fn print_tuple(&mut self, tuple: &[Expression], f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "(")?;
        self.print_comma_separated(tuple, f)?;
        write!(f, ")")
    }

    fn print_call(&mut self, call: &super::ast::Call, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "${}(", call.func_id.0)?;
        self.print_comma_separated(&call.arguments, f)?;
        write!(f, ")")
    }

    fn print_lowlevel(&mut self, call: &super::ast::CallBuiltin, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}$lowlevel(", call.opcode)?;
        self.print_comma_separated(&call.arguments, f)?;
        write!(f, ")")
    }

    fn print_builtin(&mut self, call: &super::ast::CallLowLevel, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}$builtin(", call.opcode)?;
        self.print_comma_separated(&call.arguments, f)?;
        write!(f, ")")
    }

    fn print_lvalue(&mut self, lvalue: &LValue, f: &mut Formatter) -> std::fmt::Result {
        match lvalue {
            LValue::Ident(ident) => write!(f, "{}${}", ident.name, ident.id.0),
            LValue::Index { array, index } => {
                self.print_lvalue(&array, f)?;
                write!(f, "[")?;
                self.print_expr(&index, f)?;
                write!(f, "]")
            },
            LValue::MemberAccess { object, field_index } => {
                self.print_lvalue(&object, f)?;
                write!(f, ".{}", field_index)
            },
        }
    }
}
