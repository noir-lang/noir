use std::{cmp::Ordering, collections::HashMap};

use crate::ssa_refactor::{
    ir::{
        function::{Function, FunctionId},
        instruction::{Instruction, TerminatorInstruction},
        map::AtomicCounter,
        post_order::PostOrder,
        types::Type,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

struct FunctionalParameter {
    index: usize,
    function_id: FunctionId,
}

struct FunctionalReturn {
    index: usize,
    function_id: FunctionId,
}

struct ResolutionContext {
    functions: Vec<Function>,
    function_id_generator: AtomicCounter<Function>,
    source_fn_id_to_fn_id: HashMap<FunctionId, FunctionId>,
}

impl ResolutionContext {
    pub(crate) fn resolve_functions(original_ssa: Ssa) -> Ssa {
        let ctx = ResolutionContext {
            functions: vec![],
            function_id_generator: AtomicCounter::default(),
            source_fn_id_to_fn_id: HashMap::new(),
        };
        ctx.resolve_all(original_ssa)
    }

    fn resolve_all(mut self, original_ssa: Ssa) -> Ssa {
        let main = original_ssa.main();
        let main_id = self.resolve_function(main.id(), &[], &original_ssa).0;
        self.functions.sort_by(
            |a, _b| {
                if a.id() == main_id {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            },
        );
        println!("Functions: {:?}", self.functions);
        Ssa::new(self.functions)
    }

    fn resolve_function(
        &mut self,
        source_function_id: FunctionId,
        functional_parameters: &[FunctionalParameter],
        original_ssa: &Ssa,
    ) -> (FunctionId, Vec<FunctionalReturn>) {
        let source_function = &original_ssa.functions[&source_function_id];

        let mut new_function = self.clone_function(source_function);

        let mut value_to_function = HashMap::new();

        let new_main_block = &mut new_function.dfg[source_function.entry_block()];
        let mut new_parameters = Vec::with_capacity(new_main_block.parameters().len());
        for (i, parameter) in new_main_block.parameters().iter().enumerate() {
            let found_function = functional_parameters.iter().find(|fp| fp.index == i);
            if let Some(FunctionalParameter { function_id: parameter_fn_id, .. }) = found_function {
                value_to_function.insert(*parameter, *parameter_fn_id);
            } else {
                new_parameters.push(*parameter);
            }
        }
        new_main_block.set_parameters(new_parameters);

        let find_function_parameter =
            |value_id: &ValueId, value_to_function: &HashMap<ValueId, FunctionId>| {
                match &source_function.dfg[*value_id] {
                    Value::Function(func_id) => *func_id,
                    Value::Param { .. } | Value::Instruction { .. } => value_to_function
                        .get(value_id)
                        .expect("ICE: Cannot resolve function parameter")
                        .clone(),
                    _ => unreachable!(
                        "ICE: unsupported function value {:?}",
                        &source_function.dfg[*value_id]
                    ),
                }
            };

        for block_id in PostOrder::with_function(source_function).as_slice() {
            let block = &source_function.dfg[*block_id];
            for instruction_id in block.instructions() {
                let instruction = &source_function.dfg[*instruction_id];
                if let Instruction::Call { func, arguments } = instruction {
                    let value = &source_function.dfg[*func];

                    if !matches!(value, Value::ForeignFunction(_) | Value::Intrinsic(_)) {
                        let function_id = find_function_parameter(func, &value_to_function);

                        let mut function_parameters = Vec::with_capacity(arguments.len());
                        for (position, argument) in arguments.iter().enumerate() {
                            let typ = source_function.dfg.type_of_value(*argument);
                            if let Type::Function = typ {
                                let function_id =
                                    find_function_parameter(argument, &value_to_function);
                                function_parameters.push(FunctionalParameter {
                                    index: position,
                                    function_id: function_id,
                                });
                            }
                        }
                        let (new_target, functional_returns) =
                            self.resolve_function(function_id, &function_parameters, original_ssa);

                        let result_ids = source_function.dfg.instruction_results(*instruction_id);
                        let mut non_functional_returns = Vec::with_capacity(result_ids.len());
                        for (position, result_id) in result_ids.iter().enumerate() {
                            if let Some(FunctionalReturn { function_id, .. }) =
                                functional_returns.iter().find(|fr| fr.index == position)
                            {
                                value_to_function.insert(*result_id, *function_id);
                            } else {
                                non_functional_returns.push(*result_id);
                            }
                        }

                        let new_target_value = new_function.dfg.import_function(new_target);
                        let new_arguments: Vec<ValueId> = arguments
                            .iter()
                            .filter(|arg| {
                                !matches!(source_function.dfg.type_of_value(**arg), Type::Function)
                            })
                            .map(|arg| *arg)
                            .collect();
                        new_function.dfg[*instruction_id] =
                            Instruction::Call { func: new_target_value, arguments: new_arguments };

                        new_function.dfg.set_results(*instruction_id, non_functional_returns);
                    }
                }
            }
        }

        let mut functional_returns = vec![];

        for block in new_function.reachable_blocks() {
            let terminator = new_function.dfg[block].terminator();
            if let Some(TerminatorInstruction::Return { return_values }) = terminator {
                let mut non_functional_returns = Vec::with_capacity(return_values.len());

                for return_value in return_values {
                    if let Type::Function = new_function.dfg.type_of_value(*return_value) {
                        let function_id = find_function_parameter(return_value, &value_to_function);
                        functional_returns.push(FunctionalReturn {
                            index: functional_returns.len(),
                            function_id,
                        });
                    } else {
                        non_functional_returns.push(*return_value);
                    }
                }

                new_function.dfg[block].set_terminator(TerminatorInstruction::Return {
                    return_values: non_functional_returns,
                });
            }
        }

        let new_id = new_function.id();

        println!("Pushing function: {:?}", new_function);

        self.functions.push(new_function);

        (new_id, functional_returns)
    }

    fn clone_function(&mut self, source_function: &Function) -> Function {
        let id = self.function_id_generator.next();
        let mut function = Function::new(source_function.name().to_string(), id);
        function.dfg = source_function.dfg.clone();
        function.entry_block = source_function.entry_block;
        function.set_runtime(source_function.runtime());
        println!("Cloned function: {} with runtime {:?}", function.name(), function.runtime());
        function
    }
}

impl Ssa {
    pub(crate) fn resolve_functions(self) -> Self {
        ResolutionContext::resolve_functions(self)
    }
}
