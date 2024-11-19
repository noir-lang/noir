use crate::ssa::ir::instruction::Instruction;



impl Ssa {
    /// Map arrays with the last instruction that uses it
    /// For this we simply process all the instructions in execution order
    /// and update the map whenever there is a match
    pub(crate) fn map_to_call_data_array(&self) -> HashMap<ValueId, InstructionId> {
        let mut array_use = HashMap::default();
        for func in self.functions.values() {
            let mut reverse_post_order = PostOrder::with_function(func).into_vec();
            reverse_post_order.reverse();
            for block in reverse_post_order {
                last_use(block, &func.dfg, &mut array_use);
            }
        }
        array_use
    }
//la question c comment on remplace  ?
//on pourrait simplement garder une liste instruction_id -> new instruction
// et on les remplace dans la liste des instructions instructions_mut()
    pub(crate) fn array_me(block_id: BasicBlockId, dfg: &mut DataFlowGraph) {
        let block = &dfg[block_id];
        for instruction_id in block.instructions() {
            match &dfg[*instruction_id] {
                Instruction::ArrayGet { array, index } => {
                                 // Get operations to call-data parameters are replaced by a get to the call-data-bus array
            if let Some(call_data) = dfg.data_bus.call_data {
                let array_id = dfg.resolve(*array);
                if dfg.data_bus.call_data_map.contains_key(&array_id) {
                    // on doit remplacer ce get par un autre, mais on doit d'abord fiare un calcul
                    //
                    dfg.make_constant(FieldElement::from(
                        self.data_bus.call_data_map[&array_id] as i128,
                    ), type_of_value(index));
                    let new_index = self.acir_context.add_var(index, bus_index)?; //TODO add instruction
                    Instruction::ArrayGet { array: call_data, index: new_index };; //on veut remplacer par ca
                }
                }
            }
             
        }
    }
}
}
