use std::{collections::HashMap, sync::Arc};

use acvm::acir::circuit::ErrorSelector;

use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::{
        basic_block::BasicBlockId,
        call_stack::CallStackId,
        dfg::GlobalsGraph,
        function::{Function, FunctionId},
        instruction::{ConstrainError, Instruction},
        value::ValueId,
    },
};

use super::{
    ast::AssertMessage, Identifier, ParsedBlock, ParsedFunction, ParsedGlobal, ParsedGlobalValue,
    ParsedInstruction, ParsedSsa, ParsedTerminator, ParsedValue, RuntimeType, Ssa, SsaError, Type,
};

impl ParsedSsa {
    pub(crate) fn into_ssa(self, simplify: bool) -> Result<Ssa, SsaError> {
        Translator::translate(self, simplify)
    }
}

struct Translator {
    builder: FunctionBuilder,

    /// Maps internal function names (e.g. "f1") to their IDs
    functions: HashMap<String, FunctionId>,

    /// Maps block names to their IDs
    blocks: HashMap<FunctionId, HashMap<String, BasicBlockId>>,

    /// Maps variable names to their IDs.
    ///
    /// This is necessary because the SSA we parse might have undergone some
    /// passes already which replaced some of the original IDs. The translator
    /// will recreate the SSA step by step, which can result in a new ID layout.
    variables: HashMap<FunctionId, HashMap<String, ValueId>>,

    /// The function that will hold the actual SSA globals.
    globals_function: Function,

    /// The types of globals in the parsed SSA, in the order they were defined.
    global_types: Vec<Type>,

    /// Maps names (e.g. "g0") in the parsed SSA to global IDs.
    global_values: HashMap<String, ValueId>,

    globals_graph: Arc<GlobalsGraph>,

    error_selector_counter: u64,
}

impl Translator {
    fn translate(mut parsed_ssa: ParsedSsa, simplify: bool) -> Result<Ssa, SsaError> {
        let mut translator = Self::new(&mut parsed_ssa, simplify)?;

        // Note that the `new` call above removed the main function,
        // so all we are left with are non-main functions.
        for function in parsed_ssa.functions {
            translator.translate_non_main_function(function)?;
        }

        Ok(translator.finish())
    }

    fn new(parsed_ssa: &mut ParsedSsa, simplify: bool) -> Result<Self, SsaError> {
        // A FunctionBuilder must be created with a main Function, so here wer remove it
        // from the parsed SSA to avoid adding it twice later on.
        let main_function = parsed_ssa.functions.remove(0);
        let main_id = FunctionId::test_new(0);
        let mut builder = FunctionBuilder::new(main_function.external_name.clone(), main_id);
        builder.set_runtime(main_function.runtime_type);
        builder.simplify = simplify;

        // Map function names to their IDs so calls can be resolved
        let mut function_id_counter = 1;
        let mut functions = HashMap::new();
        for function in &parsed_ssa.functions {
            let function_id = FunctionId::test_new(function_id_counter);
            function_id_counter += 1;

            functions.insert(function.internal_name.clone(), function_id);
        }

        // Does not matter what ID we use here.
        let globals = Function::new("globals".to_owned(), main_id);

        let mut translator = Self {
            builder,
            functions,
            variables: HashMap::new(),
            blocks: HashMap::new(),
            globals_function: globals,
            global_types: Vec::new(),
            global_values: HashMap::new(),
            globals_graph: Arc::new(GlobalsGraph::default()),
            error_selector_counter: 0,
        };

        translator.translate_globals(std::mem::take(&mut parsed_ssa.globals))?;

        translator.globals_graph =
            Arc::new(GlobalsGraph::from_dfg(translator.globals_function.dfg.clone()));

        translator.translate_function_body(main_function)?;

        Ok(translator)
    }

    fn translate_non_main_function(&mut self, function: ParsedFunction) -> Result<(), SsaError> {
        let function_id = self.functions[&function.internal_name];
        let external_name = function.external_name.clone();

        match function.runtime_type {
            RuntimeType::Acir(inline_type) => {
                self.builder.new_function(external_name, function_id, inline_type);
            }
            RuntimeType::Brillig(inline_type) => {
                self.builder.new_brillig_function(external_name, function_id, inline_type);
            }
        }

        self.translate_function_body(function)
    }

    fn translate_function_body(&mut self, function: ParsedFunction) -> Result<(), SsaError> {
        self.builder.set_globals(self.globals_graph.clone());

        // First define all blocks so that they are known (a block might jump to a block that comes next)
        for (index, block) in function.blocks.iter().enumerate() {
            // The first block is the entry block and it was automatically created by the builder
            let block_id = if index == 0 {
                self.builder.current_function.entry_block()
            } else {
                self.builder.insert_block()
            };
            let entry = self.blocks.entry(self.current_function_id()).or_default();
            entry.insert(block.name.clone(), block_id);
        }

        for block in function.blocks {
            self.translate_block(block)?;
        }

        Ok(())
    }

    fn translate_block(&mut self, block: ParsedBlock) -> Result<(), SsaError> {
        let block_id = self.blocks[&self.current_function_id()][&block.name];
        self.builder.switch_to_block(block_id);

        for parameter in block.parameters {
            let parameter_value_id = self.builder.add_block_parameter(block_id, parameter.typ);
            self.define_variable(parameter.identifier, parameter_value_id)?;
        }

        for instruction in block.instructions {
            self.translate_instruction(instruction)?;
        }

        match block.terminator {
            ParsedTerminator::Jmp { destination, arguments } => {
                let block_id = self.lookup_block(&destination)?;
                let arguments = self.translate_values(arguments)?;
                self.builder.terminate_with_jmp(block_id, arguments);
            }
            ParsedTerminator::Jmpif { condition, then_block, else_block } => {
                let condition = self.translate_value(condition)?;
                let then_destination = self.lookup_block(&then_block)?;
                let else_destination = self.lookup_block(&else_block)?;
                self.builder.terminate_with_jmpif(condition, then_destination, else_destination);
            }
            ParsedTerminator::Return(values) => {
                let return_values = self.translate_values(values)?;
                self.builder.terminate_with_return(return_values);
            }
        }

        Ok(())
    }

    fn translate_instruction(&mut self, instruction: ParsedInstruction) -> Result<(), SsaError> {
        match instruction {
            ParsedInstruction::Allocate { target, typ } => {
                let value_id = self.builder.insert_allocate(typ);
                self.define_variable(target, value_id)?;
            }
            ParsedInstruction::ArrayGet { target, element_type, array, index } => {
                let array = self.translate_value(array)?;
                let index = self.translate_value(index)?;
                let value_id = self.builder.insert_array_get(array, index, element_type);
                self.define_variable(target, value_id)?;
            }
            ParsedInstruction::ArraySet { target, array, index, value, mutable } => {
                let array = self.translate_value(array)?;
                let index = self.translate_value(index)?;
                let value = self.translate_value(value)?;
                let value_id = if mutable {
                    self.builder.insert_mutable_array_set(array, index, value)
                } else {
                    self.builder.insert_array_set(array, index, value)
                };
                self.define_variable(target, value_id)?;
            }
            ParsedInstruction::BinaryOp { target, lhs, op, rhs } => {
                let lhs = self.translate_value(lhs)?;
                let rhs = self.translate_value(rhs)?;
                let value_id = self.builder.insert_binary(lhs, op, rhs);
                self.define_variable(target, value_id)?;
            }
            ParsedInstruction::Call { targets, function, arguments, types } => {
                let function_id = if let Some(id) = self.builder.import_intrinsic(&function.name) {
                    id
                } else {
                    let maybe_func =
                        self.lookup_function(&function).map(|f| self.builder.import_function(f));

                    maybe_func.or_else(|e| {
                        // e.g. `v2 = call v0(v1) -> u32`, a lambda passed as a parameter
                        self.lookup_variable(&function).map_err(|_| e)
                    })?
                };

                let arguments = self.translate_values(arguments)?;

                let value_ids = self.builder.insert_call(function_id, arguments, types).to_vec();

                if value_ids.len() != targets.len() {
                    return Err(SsaError::MismatchedReturnValues {
                        returns: targets,
                        expected: value_ids.len(),
                    });
                }

                for (target, value_id) in targets.into_iter().zip(value_ids.into_iter()) {
                    self.define_variable(target, value_id)?;
                }
            }
            ParsedInstruction::Cast { target, lhs, typ } => {
                let lhs = self.translate_value(lhs)?;
                let value_id = self.builder.insert_cast(lhs, typ.unwrap_numeric());
                self.define_variable(target, value_id)?;
            }
            ParsedInstruction::Constrain { lhs, rhs, assert_message } => {
                let lhs = self.translate_value(lhs)?;
                let rhs = self.translate_value(rhs)?;
                let assert_message = match assert_message {
                    Some(AssertMessage::Static(string)) => {
                        Some(ConstrainError::StaticString(string))
                    }
                    Some(AssertMessage::Dynamic(values)) => {
                        let error_selector = ErrorSelector::new(self.error_selector_counter);
                        self.error_selector_counter += 1;

                        let is_string_type = false;
                        let values = self.translate_values(values)?;

                        Some(ConstrainError::Dynamic(error_selector, is_string_type, values))
                    }
                    None => None,
                };
                self.builder.insert_constrain(lhs, rhs, assert_message);
            }
            ParsedInstruction::DecrementRc { value } => {
                let value = self.translate_value(value)?;
                self.builder.decrement_array_reference_count(value);
            }
            ParsedInstruction::EnableSideEffectsIf { condition } => {
                let condition = self.translate_value(condition)?;
                self.builder.insert_enable_side_effects_if(condition);
            }
            ParsedInstruction::IncrementRc { value } => {
                let value = self.translate_value(value)?;
                self.builder.increment_array_reference_count(value);
            }
            ParsedInstruction::MakeArray { target, elements, typ } => {
                let elements = elements
                    .into_iter()
                    .map(|element| self.translate_value(element))
                    .collect::<Result<_, _>>()?;
                let value_id = self.builder.insert_make_array(elements, typ);
                self.define_variable(target, value_id)?;
            }
            ParsedInstruction::Load { target, value, typ } => {
                let value = self.translate_value(value)?;
                let value_id = self.builder.insert_load(value, typ);
                self.define_variable(target, value_id)?;
            }
            ParsedInstruction::Not { target, value } => {
                let value = self.translate_value(value)?;
                let value_id = self.builder.insert_not(value);
                self.define_variable(target, value_id)?;
            }
            ParsedInstruction::RangeCheck { value, max_bit_size } => {
                let value = self.translate_value(value)?;
                self.builder.insert_range_check(value, max_bit_size, None);
            }
            ParsedInstruction::Store { value, address } => {
                let value = self.translate_value(value)?;
                let address = self.translate_value(address)?;
                self.builder.insert_store(address, value);
            }
            ParsedInstruction::Truncate { target, value, bit_size, max_bit_size } => {
                let value = self.translate_value(value)?;
                let value_id = self.builder.insert_truncate(value, bit_size, max_bit_size);
                self.define_variable(target, value_id)?;
            }
        }

        Ok(())
    }

    fn translate_values(&mut self, values: Vec<ParsedValue>) -> Result<Vec<ValueId>, SsaError> {
        let mut translated_values = Vec::with_capacity(values.len());
        for value in values {
            translated_values.push(self.translate_value(value)?);
        }
        Ok(translated_values)
    }

    fn translate_value(&mut self, value: ParsedValue) -> Result<ValueId, SsaError> {
        match value {
            ParsedValue::NumericConstant(constant) => {
                Ok(self.builder.numeric_constant(constant.value, constant.typ.unwrap_numeric()))
            }
            ParsedValue::Variable(identifier) => self.lookup_variable(&identifier).or_else(|e| {
                self.lookup_function(&identifier)
                    .map(|f| {
                        // e.g. `v3 = call f1(f2, v0) -> u32`
                        self.builder.import_function(f)
                    })
                    .map_err(|_| e)
            }),
        }
    }

    fn translate_globals(&mut self, globals: Vec<ParsedGlobal>) -> Result<(), SsaError> {
        for global in globals {
            self.translate_global(global)?;
        }
        Ok(())
    }

    fn translate_global(&mut self, global: ParsedGlobal) -> Result<(), SsaError> {
        let value_id = match global.value {
            ParsedGlobalValue::NumericConstant(constant) => self
                .globals_function
                .dfg
                .make_constant(constant.value, constant.typ.unwrap_numeric()),
            ParsedGlobalValue::MakeArray(make_array) => {
                let mut elements = im::Vector::new();
                for element in make_array.elements {
                    let element_id = match element {
                        ParsedValue::NumericConstant(constant) => self
                            .globals_function
                            .dfg
                            .make_constant(constant.value, constant.typ.unwrap_numeric()),
                        ParsedValue::Variable(identifier) => self.lookup_global(identifier)?,
                    };
                    elements.push_back(element_id);
                }

                let instruction = Instruction::MakeArray { elements, typ: make_array.typ.clone() };
                let block = self.globals_function.entry_block();
                let call_stack = CallStackId::root();
                self.globals_function
                    .dfg
                    .insert_instruction_and_results(instruction, block, None, call_stack)
                    .first()
            }
        };

        self.define_global(global.name, value_id)
    }

    fn define_variable(
        &mut self,
        identifier: Identifier,
        value_id: ValueId,
    ) -> Result<(), SsaError> {
        if let Some(vars) = self.variables.get(&self.current_function_id()) {
            if vars.contains_key(&identifier.name) {
                return Err(SsaError::VariableAlreadyDefined(identifier));
            }
        }

        let entry = self.variables.entry(self.current_function_id()).or_default();
        entry.insert(identifier.name, value_id);

        Ok(())
    }

    fn lookup_variable(&mut self, identifier: &Identifier) -> Result<ValueId, SsaError> {
        if let Some(value_id) = self
            .variables
            .get(&self.current_function_id())
            .and_then(|hash| hash.get(&identifier.name))
        {
            Ok(*value_id)
        } else if let Some(value_id) = self.global_values.get(&identifier.name) {
            Ok(*value_id)
        } else {
            Err(SsaError::UnknownVariable(identifier.clone()))
        }
    }

    fn define_global(&mut self, identifier: Identifier, value_id: ValueId) -> Result<(), SsaError> {
        if self.global_values.contains_key(&identifier.name) {
            return Err(SsaError::GlobalAlreadyDefined(identifier));
        }

        self.global_values.insert(identifier.name, value_id);

        let typ = self.globals_function.dfg.type_of_value(value_id);
        self.global_types.push(typ);

        Ok(())
    }

    fn lookup_global(&mut self, identifier: Identifier) -> Result<ValueId, SsaError> {
        if let Some(value_id) = self.global_values.get(&identifier.name) {
            Ok(*value_id)
        } else {
            Err(SsaError::UnknownGlobal(identifier))
        }
    }

    fn lookup_block(&mut self, identifier: &Identifier) -> Result<BasicBlockId, SsaError> {
        if let Some(block_id) = self.blocks[&self.current_function_id()].get(&identifier.name) {
            Ok(*block_id)
        } else {
            Err(SsaError::UnknownBlock(identifier.clone()))
        }
    }

    fn lookup_function(&mut self, identifier: &Identifier) -> Result<FunctionId, SsaError> {
        if let Some(function_id) = self.functions.get(&identifier.name) {
            Ok(*function_id)
        } else {
            Err(SsaError::UnknownFunction(identifier.clone()))
        }
    }

    fn finish(self) -> Ssa {
        let mut ssa = self.builder.finish();

        // Normalize the IDs so we have a better chance of matching the SSA we parsed
        // after the step-by-step reconstruction done during translation. This assumes
        // that the SSA we parsed was printed by the `SsaBuilder`, which normalizes
        // before each print.
        ssa.normalize_ids();

        ssa
    }

    fn current_function_id(&self) -> FunctionId {
        self.builder.current_function.id()
    }
}
