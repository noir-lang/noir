use acvm::{
    acir::brillig::{ForeignCallParam, ForeignCallResult, Value},
    pwg::ForeignCallWaitInfo,
};
use nargo::{
    artifacts::debug::{DebugArtifact, DebugVars},
    ops::{DefaultForeignCallExecutor, ForeignCallExecutor},
};
use noirc_printable_type::{ForeignCallError, PrintableType, PrintableValue};

pub(crate) enum DebugForeignCall {
    VarAssign,
    VarDrop,
    MemberAssign(u32),
    DerefAssign,
}

impl std::fmt::Display for DebugForeignCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl DebugForeignCall {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            DebugForeignCall::VarAssign => "__debug_var_assign",
            DebugForeignCall::VarDrop => "__debug_var_drop",
            DebugForeignCall::MemberAssign(1) => "__debug_member_assign_1",
            DebugForeignCall::MemberAssign(2) => "__debug_member_assign_2",
            DebugForeignCall::MemberAssign(3) => "__debug_member_assign_3",
            DebugForeignCall::MemberAssign(4) => "__debug_member_assign_4",
            DebugForeignCall::MemberAssign(5) => "__debug_member_assign_5",
            DebugForeignCall::MemberAssign(6) => "__debug_member_assign_6",
            DebugForeignCall::MemberAssign(7) => "__debug_member_assign_7",
            DebugForeignCall::MemberAssign(8) => "__debug_member_assign_8",
            DebugForeignCall::MemberAssign(_) => panic!("unsupported member assignment arity"),
            DebugForeignCall::DerefAssign => "__debug_deref_assign",
        }
    }

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

impl ForeignCallExecutor for DefaultDebugForeignCallExecutor {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo,
    ) -> Result<ForeignCallResult, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        match DebugForeignCall::lookup(foreign_call_name) {
            Some(DebugForeignCall::VarAssign) => {
                let fcp_var_id = &foreign_call.inputs[0];
                if let ForeignCallParam::Single(var_id_value) = fcp_var_id {
                    let var_id = var_id_value.to_u128() as u32;
                    let values: Vec<Value> =
                        foreign_call.inputs[1..].iter().flat_map(|x| x.values()).collect();
                    self.debug_vars.assign(var_id, &values);
                }
                Ok(ForeignCallResult { values: vec![] })
            }
            Some(DebugForeignCall::VarDrop) => {
                let fcp_var_id = &foreign_call.inputs[0];
                if let ForeignCallParam::Single(var_id_value) = fcp_var_id {
                    let var_id = var_id_value.to_u128() as u32;
                    self.debug_vars.drop(var_id);
                }
                Ok(ForeignCallResult { values: vec![] })
            }
            Some(DebugForeignCall::MemberAssign(arity)) => {
                if let Some(ForeignCallParam::Single(var_id_value)) = foreign_call.inputs.get(0) {
                    let arity = arity as usize;
                    let var_id = var_id_value.to_u128() as u32;
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
                Ok(ForeignCallResult { values: vec![] })
            }
            Some(DebugForeignCall::DerefAssign) => {
                let fcp_var_id = &foreign_call.inputs[0];
                let fcp_value = &foreign_call.inputs[1];
                if let ForeignCallParam::Single(var_id_value) = fcp_var_id {
                    let var_id = var_id_value.to_u128() as u32;
                    self.debug_vars.assign_deref(var_id, &fcp_value.values());
                }
                Ok(ForeignCallResult { values: vec![] })
            }
            None => self.executor.execute(foreign_call),
        }
    }
}
