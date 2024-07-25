use std::collections::BTreeMap;
use std::rc::Rc;

use crate::ssa::ir::{types::Type, value::ValueId};
use acvm::FieldElement;
use fxhash::FxHashMap as HashMap;
use noirc_frontend::ast;
use noirc_frontend::hir_def::function::FunctionSignature;

use super::FunctionBuilder;

#[derive(Clone)]
pub(crate) enum DatabusVisibility {
    None,
    CallData(u32),
    ReturnData,
}
/// Used to create a data bus, which is an array of private inputs
/// replacing public inputs
pub(crate) struct DataBusBuilder {
    pub(crate) values: im::Vector<ValueId>,
    index: usize,
    pub(crate) map: HashMap<ValueId, usize>,
    pub(crate) databus: Option<ValueId>,
}

impl DataBusBuilder {
    pub(crate) fn new() -> DataBusBuilder {
        DataBusBuilder {
            index: 0,
            map: HashMap::default(),
            databus: None,
            values: im::Vector::new(),
        }
    }

    /// Generates a vector telling which (ssa) parameters from the given function signature
    /// are tagged with databus visibility
    pub(crate) fn is_databus(main_signature: &FunctionSignature) -> Vec<DatabusVisibility> {
        let mut params_is_databus = Vec::new();

        for param in &main_signature.0 {
            let is_databus = match param.2 {
                ast::Visibility::Public | ast::Visibility::Private => DatabusVisibility::None,
                ast::Visibility::CallData(id) => DatabusVisibility::CallData(id),
                ast::Visibility::ReturnData => DatabusVisibility::ReturnData,
            };
            let len = param.1.field_count() as usize;
            params_is_databus.extend(vec![is_databus; len]);
        }
        params_is_databus
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CallData {
    pub(crate) array_id: ValueId,
    pub(crate) index_map: HashMap<ValueId, usize>,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct DataBus {
    pub(crate) call_data: Vec<CallData>,
    pub(crate) return_data: Option<ValueId>,
}

impl DataBus {
    /// Updates the databus values with the provided function
    pub(crate) fn map_values(&self, mut f: impl FnMut(ValueId) -> ValueId) -> DataBus {
        let call_data = self
            .call_data
            .iter()
            .map(|cd| {
                let mut call_data_map = HashMap::default();
                for (k, v) in cd.index_map.iter() {
                    call_data_map.insert(f(*k), *v);
                }
                CallData { array_id: f(cd.array_id), index_map: call_data_map }
            })
            .collect();
        DataBus { call_data, return_data: self.return_data.map(&mut f) }
    }

    pub(crate) fn call_data_array(&self) -> Vec<ValueId> {
        self.call_data.iter().map(|cd| cd.array_id).collect()
    }
    /// Construct a databus from call_data and return_data data bus builders
    pub(crate) fn get_data_bus(
        call_data: Vec<DataBusBuilder>,
        return_data: DataBusBuilder,
    ) -> DataBus {
        let mut call_data_args = Vec::new();
        for call_data_item in call_data {
            if let Some(array_id) = call_data_item.databus {
                call_data_args.push(CallData { array_id, index_map: call_data_item.map });
            }
        }

        DataBus { call_data: call_data_args, return_data: return_data.databus }
    }
}

impl FunctionBuilder {
    /// Insert a value into a data bus builder
    fn add_to_data_bus(&mut self, value: ValueId, databus: &mut DataBusBuilder) {
        assert!(databus.databus.is_none(), "initializing finalized call data");
        let typ = self.current_function.dfg[value].get_type().clone();
        match typ {
            Type::Numeric(_) => {
                databus.values.push_back(value);
                databus.index += 1;
            }
            Type::Array(typ, len) => {
                assert!(typ.len() == 1, "unsupported composite type");
                databus.map.insert(value, databus.index);
                for i in 0..len {
                    // load each element of the array
                    let index = self
                        .current_function
                        .dfg
                        .make_constant(FieldElement::from(i as i128), Type::length_type());
                    let element = self.insert_array_get(value, index, typ[0].clone());
                    self.add_to_data_bus(element, databus);
                }
            }
            Type::Reference(_) => {
                unreachable!("Attempted to add invalid type (reference) to databus")
            }
            Type::Slice(_) => unreachable!("Attempted to add invalid type (slice) to databus"),
            Type::Function => unreachable!("Attempted to add invalid type (function) to databus"),
        }
    }

    /// Create a data bus builder from a list of values
    pub(crate) fn initialize_data_bus(
        &mut self,
        values: &[ValueId],
        mut databus: DataBusBuilder,
    ) -> DataBusBuilder {
        for value in values {
            self.add_to_data_bus(*value, &mut databus);
        }
        let len = databus.values.len();

        let array = if len > 0 {
            let array =
                self.array_constant(databus.values, Type::Array(Rc::new(vec![Type::field()]), len));
            Some(array)
        } else {
            None
        };

        DataBusBuilder { index: 0, map: databus.map, databus: array, values: im::Vector::new() }
    }

    /// Generate the data bus for call-data, based on the parameters of the entry block
    /// and a vector telling which ones are call-data
    pub(crate) fn call_data_bus(
        &mut self,
        is_params_databus: Vec<DatabusVisibility>,
    ) -> Vec<DataBusBuilder> {
        //filter parameters of the first block that have call-data visibility
        let first_block = self.current_function.entry_block();
        let params = self.current_function.dfg[first_block].parameters();
        let mut databus_param: BTreeMap<u32, Vec<ValueId>> = BTreeMap::new();
        for (param, databus_attribute) in params.iter().zip(is_params_databus) {
            match databus_attribute {
                DatabusVisibility::None | DatabusVisibility::ReturnData => continue,
                DatabusVisibility::CallData(call_data_id) => {
                    if let std::collections::btree_map::Entry::Vacant(e) =
                        databus_param.entry(call_data_id)
                    {
                        e.insert(vec![param.to_owned()]);
                    } else {
                        databus_param.get_mut(&call_data_id).unwrap().push(param.to_owned());
                    }
                }
            }
        }
        // create the call-data-bus from the filtered lists
        let mut result = Vec::new();
        for id in databus_param.keys() {
            let builder = DataBusBuilder::new();
            let call_databus = self.initialize_data_bus(&databus_param[id], builder);
            result.push(call_databus);
        }
        result
    }
}
