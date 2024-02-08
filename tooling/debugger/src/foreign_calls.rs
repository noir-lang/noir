use acvm::{
    acir::brillig::{ForeignCallParam, ForeignCallResult, Value},
    pwg::ForeignCallWaitInfo,
};
use nargo::{
    artifacts::debug::{DebugArtifact, DebugVars},
    ops::{DefaultForeignCallExecutor, ForeignCallExecutor, NargoForeignCallResult},
};
use noirc_errors::debug_info::DebugVarId;
use noirc_printable_type::{ForeignCallError, PrintableType, PrintableValue};

pub(crate) enum DebugForeignCall {
    VarAssign,
    VarDrop,
    MemberAssign(u32),
    DerefAssign,
}

impl DebugForeignCall {
    pub(crate) fn lookup(op_name: &str) -> Option<DebugForeignCall> {
        let member_pre = "__debug_member_assign_";
        if let Some(op_suffix) = op_name.strip_prefix(member_pre) {
            let arity =
                op_suffix.parse::<u32>().expect("failed to parse debug_member_assign arity");
            return Some(DebugForeignCall::MemberAssign(arity));
        }
        match op_name {
            "__debug_var_assign" => Some(DebugForeignCall::VarAssign),
            "__debug_var_drop" => Some(DebugForeignCall::VarDrop),
            "__debug_deref_assign" => Some(DebugForeignCall::DerefAssign),
            _ => None,
        }
    }
}

pub trait DebugForeignCallExecutor: ForeignCallExecutor {
    fn get_variables(&self) -> Vec<(&str, &PrintableValue, &PrintableType)>;
}

pub struct DefaultDebugForeignCallExecutor {
    executor: DefaultForeignCallExecutor,
    pub debug_vars: DebugVars,
}

impl DefaultDebugForeignCallExecutor {
    pub fn new(show_output: bool) -> Self {
        Self {
            executor: DefaultForeignCallExecutor::new(show_output, None),
            debug_vars: DebugVars::default(),
        }
    }

    pub fn from_artifact(show_output: bool, artifact: &DebugArtifact) -> Self {
        let mut ex = Self::new(show_output);
        ex.load_artifact(artifact);
        ex
    }

    pub fn load_artifact(&mut self, artifact: &DebugArtifact) {
        artifact.debug_symbols.iter().for_each(|info| {
            self.debug_vars.insert_variables(&info.variables);
            self.debug_vars.insert_types(&info.types);
        });
    }
}

impl DebugForeignCallExecutor for DefaultDebugForeignCallExecutor {
    fn get_variables(&self) -> Vec<(&str, &PrintableValue, &PrintableType)> {
        self.debug_vars.get_variables()
    }
}

fn debug_var_id(value: &Value) -> DebugVarId {
    DebugVarId(value.to_u128() as u32)
}

impl ForeignCallExecutor for DefaultDebugForeignCallExecutor {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo,
    ) -> Result<NargoForeignCallResult, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        match DebugForeignCall::lookup(foreign_call_name) {
            Some(DebugForeignCall::VarAssign) => {
                let fcp_var_id = &foreign_call.inputs[0];
                if let ForeignCallParam::Single(var_id_value) = fcp_var_id {
                    let var_id = debug_var_id(var_id_value);
                    let values: Vec<Value> =
                        foreign_call.inputs[1..].iter().flat_map(|x| x.values()).collect();
                    self.debug_vars.assign_var(var_id, &values);
                }
                Ok(ForeignCallResult::default().into())
            }
            Some(DebugForeignCall::VarDrop) => {
                let fcp_var_id = &foreign_call.inputs[0];
                if let ForeignCallParam::Single(var_id_value) = fcp_var_id {
                    let var_id = debug_var_id(var_id_value);
                    self.debug_vars.drop_var(var_id);
                }
                Ok(ForeignCallResult::default().into())
            }
            Some(DebugForeignCall::MemberAssign(arity)) => {
                if let Some(ForeignCallParam::Single(var_id_value)) = foreign_call.inputs.get(0) {
                    let arity = arity as usize;
                    let var_id = debug_var_id(var_id_value);
                    let n = foreign_call.inputs.len();
                    let indexes: Vec<u32> = foreign_call.inputs[(n - arity)..n]
                        .iter()
                        .map(|fcp_v| {
                            if let ForeignCallParam::Single(v) = fcp_v {
                                v.to_u128() as u32
                            } else {
                                panic!("expected ForeignCallParam::Single(v)");
                            }
                        })
                        .collect();
                    let values: Vec<Value> = (0..n - 1 - arity)
                        .flat_map(|i| {
                            foreign_call.inputs.get(1 + i).map(|fci| fci.values()).unwrap_or(vec![])
                        })
                        .collect();
                    self.debug_vars.assign_field(var_id, indexes, &values);
                }
                Ok(ForeignCallResult::default().into())
            }
            Some(DebugForeignCall::DerefAssign) => {
                let fcp_var_id = &foreign_call.inputs[0];
                let fcp_value = &foreign_call.inputs[1];
                if let ForeignCallParam::Single(var_id_value) = fcp_var_id {
                    let var_id = debug_var_id(var_id_value);
                    self.debug_vars.assign_deref(var_id, &fcp_value.values());
                }
                Ok(ForeignCallResult::default().into())
            }
            None => self.executor.execute(foreign_call),
        }
    }
}
