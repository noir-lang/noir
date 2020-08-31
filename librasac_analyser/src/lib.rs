/// This module is for now just a placeholder
/// We want the analyser to do quite a few things such as:
/// - Be able to check for unused variables
/// - Check if function parameters and arguments are lined up
/// - Check if all variables are in scope
/// 
/// Note: Given private x = arith
/// Throw an error, if private x is never used.
/// This allows us to constrain x straight away
use librasac_ast::{Expression, Statement};
use librasac_parser::{Program};

pub fn check(ast : &Program) {
    argument_size_check(ast);
}

// Checks if each function has the right argument call size
fn argument_size_check(ast : &Program) {

    let main_func = match &ast.main {
        Some(func) => func,
        None => panic!("No main function found")
    };

    
    for statement in main_func.func.body.0.iter() {
        match statement {
            Statement::Expression(expr) => {
                expression_check(ast, &expr.0)
            },
            Statement::Private(private_stmt) => {
                expression_check(ast, &private_stmt.expression);
            },
            Statement::Constrain(constrain_stmt) => {
                
                expression_check(ast, &constrain_stmt.0.lhs);
                expression_check(ast, &constrain_stmt.0.rhs);
            },
            k=>{dbg!(k);}
        }
    }
}


fn expression_check(ast : &Program, expr : &Expression) {
    match expr {
        Expression::Call(call_expr) => {
            // First check that the function we are calling, has been declared before
            check_call_expression(ast, call_expr);            
        },
        Expression::Infix(infix_expr) => {
            expression_check(ast, &infix_expr.lhs);
            expression_check(ast, &infix_expr.rhs);
        }
        _=> {}
    }
}

use librasac_ast::{FunctionDefinition,CallExpression};
fn check_call_expression(ast : &Program, call_expr : &CallExpression) {
    
    let func_name = &call_expr.func_name.0;
    
    // Check if the function exists
    let func = match find_function(ast, func_name) {
        Some(func) => {
            func
        }, 
        None => {
            panic!("Cannot find function named {}", func_name)
        }
    };

    // Check if we have the correct number of arguments
    check_correct_number_of_arguments(func, call_expr.arguments.len());
}

fn find_function<'a>(ast : &'a Program, func_name : &str) -> Option<&'a FunctionDefinition> {
    for function in ast.functions.iter() {
        if &function.name.0 == func_name {
            return Some(function)
        }
    }
    return None
}

fn check_correct_number_of_arguments(func_def : &FunctionDefinition, num_called : usize) -> bool {
    
    let num_parameters = func_def.func.parameters.len();
    match num_parameters == num_called {
        true => {
            return true
        },
        false => {
            panic!("The function {} takes {} parameters, but {} were supplied", func_def.name.0, num_parameters, num_called)
        }
    }
}