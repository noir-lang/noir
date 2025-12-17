//! This module implements printing of the monomorphized AST, for debugging purposes.

use crate::{
    ast::UnaryOp,
    monomorphization::ast::{Ident, Literal},
};

use super::ast::{
    Definition, Expression, FuncId, Function, GlobalId, InlineType, LValue, LocalId, Program, Type,
    While,
};
use iter_extended::vecmap;
use std::fmt::{Display, Formatter};

#[derive(Default)]
pub struct FunctionPrintOptions {
    /// Wraps function body in a `comptime` block. Used to make
    /// comptime function callers in fuzzing.
    pub comptime_wrap_body: bool,
    /// Marks function as comptime. Used in fuzzing.
    pub comptime: bool,
}

/// Some calls can be printed with the intention of parsing the code.
#[derive(Debug, PartialEq)]
enum SpecialCall {
    Print,
    Object(String),
}

#[derive(Debug)]
pub struct AstPrinter {
    indent_level: u32,
    in_unconstrained: bool,
    pub show_id: bool,
    pub show_clone_and_drop: bool,
    pub show_specials_as_std: bool,
    pub show_type_in_let: bool,
    pub show_type_of_int_literal: bool,
}

impl Default for AstPrinter {
    fn default() -> Self {
        Self {
            indent_level: 0,
            in_unconstrained: false,
            show_id: true,
            show_clone_and_drop: true,
            show_specials_as_std: false,
            show_type_in_let: false,
            show_type_of_int_literal: false,
        }
    }
}

impl AstPrinter {
    fn fmt_ident(&self, name: &str, definition: &Definition) -> String {
        if self.show_id { format!("{name}${definition}") } else { name.to_string() }
    }

    fn fmt_local(&self, name: &str, id: LocalId) -> String {
        self.fmt_ident(name, &Definition::Local(id))
    }

    fn fmt_global(&self, name: &str, id: GlobalId) -> String {
        self.fmt_ident(name, &Definition::Global(id))
    }

    fn fmt_func(&self, name: &str, id: FuncId) -> String {
        self.fmt_ident(name, &Definition::Function(id))
    }

    fn fmt_match(&self, name: &str, id: LocalId) -> String {
        if self.show_id { format!("${}", id.0) } else { self.fmt_local(name, id) }
    }

    pub fn print_program(&mut self, program: &Program, f: &mut Formatter) -> std::fmt::Result {
        for (id, global) in &program.globals {
            self.print_global(id, global, f)?;
        }
        for function in &program.functions {
            let fpo = FunctionPrintOptions::default();
            self.print_function(function, f, fpo)?;
        }
        Ok(())
    }

    pub fn print_global(
        &mut self,
        id: &GlobalId,
        (name, typ, expr): &(String, Type, Expression),
        f: &mut Formatter,
    ) -> std::fmt::Result {
        write!(f, "global {}: {} = ", self.fmt_global(name, *id), typ)?;
        self.print_expr(expr, f)?;
        write!(f, ";")?;
        self.next_line(f)
    }

    pub fn print_function(
        &mut self,
        function: &Function,
        f: &mut Formatter,
        options: FunctionPrintOptions,
    ) -> std::fmt::Result {
        let params = vecmap(&function.parameters, |(id, mutable, name, typ, visibility)| {
            let vis = visibility.to_string();
            let vis = if vis.is_empty() { vis } else { format!("{vis} ") };
            format!(
                "{}{}: {}{}",
                if *mutable { "mut " } else { "" },
                self.fmt_local(name, *id),
                vis,
                typ
            )
        })
        .join(", ");

        let vis = function.return_visibility.to_string();
        let vis = if vis.is_empty() { vis } else { format!("{vis} ") };

        let unconstrained = if function.unconstrained { "unconstrained " } else { "" };
        let comptime = if options.comptime { "comptime " } else { "" };
        let name = self.fmt_func(&function.name, function.id);
        let return_type = &function.return_type;

        if function.inline_type != InlineType::Inline {
            writeln!(f, "#[{}]", function.inline_type)?;
        }
        write!(f, "{comptime}{unconstrained}fn {name}({params}) -> {vis}{return_type} {{",)?;
        self.in_unconstrained = function.unconstrained;
        if options.comptime_wrap_body {
            self.indent_level += 1;
            self.next_line(f)?;
            write!(f, "comptime {{")?;
        }
        self.indent_level += 1;
        self.print_expr_expect_block(&function.body, f)?;
        self.indent_level -= 1;
        if options.comptime_wrap_body {
            self.next_line(f)?;
            self.indent_level -= 1;
            write!(f, "}}")?;
        }
        self.in_unconstrained = false;
        self.next_line(f)?;
        writeln!(f, "}}")?;
        Ok(())
    }

    pub fn print_expr(&mut self, expr: &Expression, f: &mut Formatter) -> std::fmt::Result {
        match expr {
            Expression::Ident(ident) => {
                write!(f, "{}", self.fmt_ident(&ident.name, &ident.definition))
            }
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
            Expression::Loop(block) => self.print_loop(block, f),
            Expression::While(while_) => self.print_while(while_, f),
            Expression::If(if_expr) => self.print_if(if_expr, f),
            Expression::Match(match_expr) => self.print_match(match_expr, f),
            Expression::Tuple(tuple) => self.print_tuple(tuple, f),
            Expression::ExtractTupleField(expr, index) => {
                self.print_expr(expr, f)?;
                write!(f, ".{index}")
            }
            Expression::Call(call) => self.print_call(call, f),
            Expression::Let(let_expr) => {
                let typ = if self.show_type_in_let
                    && let_expr.expression.needs_type_inference_from_literal()
                {
                    &let_expr
                        .expression
                        .return_type()
                        .map(|typ| format!(": {typ}"))
                        .unwrap_or_default()
                } else {
                    ""
                };
                write!(
                    f,
                    "let {}{}{} = ",
                    if let_expr.mutable { "mut " } else { "" },
                    self.fmt_local(&let_expr.name, let_expr.id),
                    typ
                )?;
                self.print_expr(&let_expr.expression, f)
            }
            Expression::Constrain(expr, _, payload) => {
                write!(f, "assert(")?;
                self.print_expr(expr, f)?;
                if let Some(payload) = payload {
                    write!(f, ", ")?;
                    self.print_expr(&payload.as_ref().0, f)?;
                }
                write!(f, ")")?;
                Ok(())
            }
            Expression::Assign(assign) => {
                self.print_lvalue(&assign.lvalue, f)?;
                write!(f, " = ")?;
                self.print_expr(&assign.expression, f)
            }
            Expression::Semi(expr) => {
                self.print_expr(expr, f)?;
                write!(f, ";")
            }
            Expression::Break => write!(f, "break"),
            Expression::Continue => write!(f, "continue"),
            Expression::Clone(expr) => {
                self.print_expr(expr, f)?;
                if self.show_clone_and_drop {
                    write!(f, ".clone()")?;
                }
                Ok(())
            }
            Expression::Drop(expr) => {
                self.print_expr(expr, f)?;
                if self.show_clone_and_drop {
                    write!(f, ".drop()")?;
                }
                Ok(())
            }
        }
    }

    fn next_line(&mut self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f)?;
        for _ in 0..self.indent_level {
            write!(f, "    ")?;
        }
        Ok(())
    }

    pub fn print_literal(
        &mut self,
        literal: &Literal,
        f: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
        match literal {
            Literal::Array(array) => {
                write!(f, "[")?;
                self.print_comma_separated(&array.contents, f)?;
                write!(f, "]")
            }
            Literal::List(array) => {
                write!(f, "&[")?;
                self.print_comma_separated(&array.contents, f)?;
                write!(f, "]")
            }
            Literal::Integer(x, typ, _) => {
                if self.show_type_of_int_literal && *typ != Type::Field {
                    write!(f, "{x}_{typ}")
                } else {
                    x.fmt(f)
                }
            }
            Literal::Bool(x) => x.fmt(f),
            Literal::Str(s) => {
                if s.contains("\"") {
                    write!(f, "r#\"{s}\"#")
                } else {
                    write!(f, "\"{s}\"")
                }
            }
            Literal::FmtStr(fragments, _, _) => {
                write!(f, "f\"")?;
                for fragment in fragments {
                    fragment.fmt(f)?;
                }
                write!(f, "\"")
            }
            Literal::Unit => {
                write!(f, "()")
            }
        }
    }

    fn print_block(
        &mut self,
        exprs: &[Expression],
        f: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
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
    fn print_expr_expect_block(
        &mut self,
        expr: &Expression,
        f: &mut Formatter,
    ) -> std::fmt::Result {
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
            }
            other => {
                self.next_line(f)?;
                self.print_expr(other, f)
            }
        }
    }

    fn print_unary(
        &mut self,
        unary: &super::ast::Unary,
        f: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
        // "(-1)" parses back as the literal -1, so if we are printing with the intention of parsing, omit the (), to avoid ambiguity.
        let print_parens = self.show_id || !matches!(unary.operator, UnaryOp::Minus);
        if print_parens {
            write!(f, "(")?;
        }
        write!(f, "{}", unary.operator)?;
        if matches!(&unary.operator, UnaryOp::Reference { mutable: true }) {
            write!(f, " ")?;
        }
        self.print_expr(&unary.rhs, f)?;
        if print_parens {
            write!(f, ")")?;
        }
        Ok(())
    }

    fn print_binary(
        &mut self,
        binary: &super::ast::Binary,
        f: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
        write!(f, "(")?;
        self.print_expr(&binary.lhs, f)?;
        write!(f, " {} ", binary.operator)?;
        self.print_expr(&binary.rhs, f)?;
        write!(f, ")")
    }

    fn print_for(
        &mut self,
        for_expr: &super::ast::For,
        f: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
        write!(f, "for {} in ", self.fmt_local(&for_expr.index_name, for_expr.index_variable))?;
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

    fn print_loop(&mut self, block: &Expression, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "loop {{")?;
        self.indent_level += 1;
        self.print_expr_expect_block(block, f)?;
        self.indent_level -= 1;
        self.next_line(f)?;
        write!(f, "}}")
    }

    fn print_while(&mut self, while_: &While, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "while ")?;
        self.print_expr(&while_.condition, f)?;
        write!(f, " {{")?;
        self.indent_level += 1;
        self.print_expr_expect_block(&while_.body, f)?;
        self.indent_level -= 1;
        self.next_line(f)?;
        write!(f, "}}")
    }

    fn print_if(
        &mut self,
        if_expr: &super::ast::If,
        f: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
        write!(f, "if ")?;
        self.print_expr(&if_expr.condition, f)?;

        write!(f, " {{")?;
        self.indent_level += 1;
        self.print_expr_expect_block(&if_expr.consequence, f)?;
        self.indent_level -= 1;
        self.next_line(f)?;

        if let Some(alt) = &if_expr.alternative {
            write!(f, "}} else {{")?;
            self.indent_level += 1;
            self.print_expr_expect_block(alt, f)?;
            self.indent_level -= 1;
            self.next_line(f)?;
        }
        write!(f, "}}")
    }

    fn print_match(
        &mut self,
        match_expr: &super::ast::Match,
        f: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
        let (var_id, var_name) = &match_expr.variable_to_match;
        write!(f, "match {} {{", self.fmt_match(var_name, *var_id))?;
        self.indent_level += 1;
        self.next_line(f)?;

        for (i, case) in match_expr.cases.iter().enumerate() {
            write!(f, "{}", case.constructor)?;
            let args = vecmap(&case.arguments, |(id, name)| self.fmt_match(name, *id)).join(", ");
            if !args.is_empty() {
                write!(f, "({args})")?;
            }
            write!(f, " => ")?;
            self.print_expr(&case.branch, f)?;
            write!(f, ",")?;

            if i != match_expr.cases.len() - 1 {
                self.next_line(f)?;
            }
        }
        self.indent_level -= 1;

        if let Some(default) = &match_expr.default_case {
            self.indent_level += 1;
            self.next_line(f)?;
            write!(f, "_ => ")?;
            self.print_expr(default, f)?;
            write!(f, ",")?;
            self.indent_level -= 1;
        }

        self.next_line(f)?;
        write!(f, "}}")
    }

    fn print_comma_separated(
        &mut self,
        exprs: &[Expression],
        f: &mut Formatter,
    ) -> std::fmt::Result {
        for (i, elem) in exprs.iter().enumerate() {
            self.print_expr(elem, f)?;
            if i != exprs.len() - 1 {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }

    fn print_tuple(
        &mut self,
        tuple: &[Expression],
        f: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
        if self.print_function_tuple(tuple, f)? {
            return Ok(());
        }
        write!(f, "(")?;
        self.print_comma_separated(tuple, f)?;
        write!(f, ")")
    }

    /// Check if we have a tuple of (constrained, unconstrained) functions and if we want to print specials as std calls,
    /// then assume that we would rather see `println(foo)` than `println((foo, foo))`, so we can render the AST as Noir
    /// without duplicating into `println(((foo, foo), (foo, foo)))` if we print the AST and re-parse it, for example for comptime tests.
    ///
    /// Returns a flag to indicate if the items were handled.
    fn print_function_tuple(
        &mut self,
        tuple: &[Expression],
        f: &mut Formatter,
    ) -> Result<bool, std::fmt::Error> {
        if !self.show_specials_as_std || tuple.len() != 2 {
            return Ok(false);
        }

        fn maybe_func(expr: &Expression) -> Option<&str> {
            // The AST fuzzer generates Type::Function; the Monomorphizer would be Type::Tuple([Type::Function, Type::Function])
            if let Expression::Ident(Ident { typ: Type::Function(_, _, _, _), name, .. }) = expr {
                Some(name.as_str())
            } else {
                None
            }
        }

        match (maybe_func(&tuple[0]), maybe_func(&tuple[1])) {
            (Some(c), Some(u)) if c == u => {
                // Only print the first element.
                self.print_expr(&tuple[0], f)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn get_called_function(expr: &Expression) -> Option<(bool, &Definition, &String)> {
        let is_unconstrained = |typ: &Type| match typ {
            Type::Function(_, _, _, unconstrained) => *unconstrained,
            Type::Tuple(elements) => match elements.first() {
                Some(Type::Function(_, _, _, unconstrained)) => *unconstrained,
                _ => false,
            },
            _ => false,
        };

        match expr {
            Expression::Ident(Ident { typ, definition, name, .. }) => {
                Some((is_unconstrained(typ), definition, name))
            }
            Expression::Tuple(elements) => match elements.first() {
                Some(Expression::Ident(Ident { typ, definition, name, .. })) => {
                    Some((is_unconstrained(typ), definition, name))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn print_call(
        &mut self,
        call: &super::ast::Call,
        f: &mut Formatter,
    ) -> Result<(), std::fmt::Error> {
        let (print_unsafe, special) = if let Some((unconstrained, definition, name)) =
            Self::get_called_function(&call.func)
        {
            let is_unsafe = unconstrained && !self.in_unconstrained;
            let special = match definition {
                Definition::Oracle(s) if s == "print" => Some(SpecialCall::Print),
                Definition::Builtin(s) if s.starts_with("array") || s.starts_with("list") => {
                    Some(SpecialCall::Object(name.clone()))
                }
                _ => None,
            };
            (is_unsafe, special)
        } else {
            (false, None)
        };

        if let Some(special) = special {
            if self.print_special_call(special, &call.arguments, f)? {
                return Ok(());
            }
        }

        if print_unsafe {
            write!(f, "unsafe {{ ")?;
        }
        self.print_expr(&call.func, f)?;
        write!(f, "(")?;
        self.print_comma_separated(&call.arguments, f)?;
        write!(f, ")")?;
        if print_unsafe {
            write!(f, " }}")?;
        }
        Ok(())
    }

    /// Try to display a special call as Noir.
    fn print_special_call(
        &mut self,
        special: SpecialCall,
        args: &[Expression],
        f: &mut Formatter,
    ) -> Result<bool, std::fmt::Error> {
        if !self.show_specials_as_std {
            return Ok(false);
        }
        match special {
            SpecialCall::Print => self.print_println(args, f),
            SpecialCall::Object(method) => {
                self.print_object_method(&method, args, f)?;
                Ok(true)
            }
        }
    }

    /// Instead of printing a call to the print oracle as a regular function,
    /// print it in a way that makes it look like Noir: without the type
    /// information and bool flags.
    ///
    /// This will only work if the AST bypassed the proxy functions created by
    /// the monomorphizer. The returned flag indicates whether it managed to
    /// do so, or false if the arguments were not as expected.
    fn print_println(
        &mut self,
        args: &[Expression],
        f: &mut Formatter,
    ) -> Result<bool, std::fmt::Error> {
        assert_eq!(args.len(), 4, "print has 4 arguments");
        let Expression::Literal(Literal::Bool(with_newline)) = args[0] else {
            return Ok(false);
        };
        if with_newline {
            write!(f, "println")?;
        } else {
            write!(f, "print")?;
        }
        write!(f, "(")?;
        // The 2nd parameter is the printed value. The 3rd and 4th parameter don't appear in Noir;
        // they are inserted automatically by the monomorphizer in the AST. Here we ignore them.
        self.print_expr(&args[1], f)?;
        write!(f, ")")?;
        Ok(true)
    }

    /// Special method for printing builtin array method calls, turning e.g. `len$array_len(x)` into `x.len()`.
    fn print_object_method(
        &mut self,
        method: &str,
        args: &[Expression],
        f: &mut Formatter,
    ) -> Result<bool, std::fmt::Error> {
        assert!(!args.is_empty(), "methods need at least a self argument");
        let (arr, args) = args.split_at(1);
        self.print_expr(&arr[0], f)?;
        write!(f, ".{method}(")?;
        self.print_comma_separated(args, f)?;
        write!(f, ")")?;
        Ok(true)
    }

    fn print_lvalue(&mut self, lvalue: &LValue, f: &mut Formatter) -> std::fmt::Result {
        match lvalue {
            LValue::Ident(ident) => write!(f, "{}", self.fmt_ident(&ident.name, &ident.definition)),
            LValue::Index { array, index, .. } => {
                self.print_lvalue(array, f)?;
                write!(f, "[")?;
                self.print_expr(index, f)?;
                write!(f, "]")
            }
            LValue::MemberAccess { object, field_index } => {
                self.print_lvalue(object, f)?;
                write!(f, ".{field_index}")
            }
            LValue::Dereference { reference, .. } => {
                write!(f, "*")?;
                self.print_lvalue(reference, f)
            }
            LValue::Clone(lvalue) => {
                self.print_lvalue(lvalue, f)?;
                if self.show_clone_and_drop {
                    write!(f, ".clone()")?;
                }
                Ok(())
            }
        }
    }
}

impl Display for Definition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Definition::Local(id) => write!(f, "l{}", id.0),
            Definition::Global(id) => write!(f, "g{}", id.0),
            Definition::Function(id) => write!(f, "f{id}"),
            Definition::Builtin(name) => write!(f, "{name}"),
            Definition::LowLevel(name) => write!(f, "{name}"),
            Definition::Oracle(name) => write!(f, "{name}"),
        }
    }
}
