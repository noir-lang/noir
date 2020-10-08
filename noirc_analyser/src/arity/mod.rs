// Check argument arities for call expressions
// And arrays

// Checks if each function has the right argument call size
// fn argument_size_check(ast: &Program, symbol_table : &SymbolTable) {
//     let main_func = match &ast.main {
//         Some(func) => func,
//         None => panic!("No main function found"),
//     };

//     for statement in main_func.literal.body.0.iter() {
//         match statement {
//             Statement::Expression(expr) => expression_check(ast, symbol_table,&expr.0),
//             Statement::Private(private_stmt) => {
//                 expression_check(ast, symbol_table, &private_stmt.expression);
//             }
//             Statement::Constrain(constrain_stmt) => {
//                 expression_check(ast, symbol_table, &constrain_stmt.0.lhs);
//                 expression_check(ast, symbol_table,&constrain_stmt.0.rhs);
//             }
//             k => {
//                 dbg!(k);
//             }
//         }
//     }
// }

// fn find_function<'a>(ast: &'a Program, func_name: &str) -> Option<&'a FunctionDefinition> {
//     for function in ast.functions.iter() {
//         if &function.name.0 == func_name {
//             return Some(function);
//         }
//     }

//     return None;
// }

// fn check_correct_number_of_arguments(func_def: &FunctionDefinition, num_called: usize) -> bool {
//     let num_parameters = func_def.literal.parameters.len();
//     match num_parameters == num_called {
//         true => return true,
//         false => panic!(
//             "The function {} takes {} parameters, but {} were supplied",
//             func_def.name.0, num_parameters, num_called
//         ),
//     }
// }