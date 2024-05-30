use acvm::{
    acir::brillig::{ForeignCallParam, ForeignCallResult},
    pwg::ForeignCallWaitInfo,
    AcirField, FieldElement,
};
use nargo::{
    artifacts::debug::{DebugArtifact, DebugVars, StackFrame},
    ops::{DefaultForeignCallExecutor, ForeignCallExecutor},
};
use noirc_errors::debug_info::{DebugFnId, DebugVarId};
use noirc_printable_type::ForeignCallError;

pub(crate) enum DebugForeignCall {
    VarAssign,
    VarDrop,
    MemberAssign(u32),
    DerefAssign,
    FnEnter,
    FnExit,
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
            "__debug_fn_enter" => Some(DebugForeignCall::FnEnter),
            "__debug_fn_exit" => Some(DebugForeignCall::FnExit),
            _ => None,
        }
    }
}

pub trait DebugForeignCallExecutor: ForeignCallExecutor {
    fn get_variables(&self) -> Vec<StackFrame>;
    fn current_stack_frame(&self) -> Option<StackFrame>;
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
        // TODO: handle loading from the correct DebugInfo when we support
        // debugging contracts
        let Some(info) = artifact.debug_symbols.first() else {
            return;
        };
        self.debug_vars.insert_debug_info(info);
    }
}

impl DebugForeignCallExecutor for DefaultDebugForeignCallExecutor {
    fn get_variables(&self) -> Vec<StackFrame> {
        self.debug_vars.get_variables()
    }

    fn current_stack_frame(&self) -> Option<StackFrame> {
        self.debug_vars.current_stack_frame()
    }
}

fn debug_var_id(value: &FieldElement) -> DebugVarId {
    DebugVarId(value.to_u128() as u32)
}

fn debug_fn_id(value: &FieldElement) -> DebugFnId {
    DebugFnId(value.to_u128() as u32)
}

impl ForeignCallExecutor for DefaultDebugForeignCallExecutor {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<FieldElement>,
    ) -> Result<ForeignCallResult<FieldElement>, ForeignCallError> {
        let foreign_call_name = foreign_call.function.as_str();
        match DebugForeignCall::lookup(foreign_call_name) {
            Some(DebugForeignCall::VarAssign) => {
                let fcp_var_id = &foreign_call.inputs[0];
                if let ForeignCallParam::Single(var_id_value) = fcp_var_id {
                    let var_id = debug_var_id(var_id_value);
                    let values: Vec<FieldElement> =
                        foreign_call.inputs[1..].iter().flat_map(|x| x.fields()).collect();
                    self.debug_vars.assign_var(var_id, &values);
                }
                Ok(ForeignCallResult::default())
            }
            Some(DebugForeignCall::VarDrop) => {
                let fcp_var_id = &foreign_call.inputs[0];
                if let ForeignCallParam::Single(var_id_value) = fcp_var_id {
                    let var_id = debug_var_id(var_id_value);
                    self.debug_vars.drop_var(var_id);
                }
                Ok(ForeignCallResult::default())
            }
            Some(DebugForeignCall::MemberAssign(arity)) => {
                if let Some(ForeignCallParam::Single(var_id_value)) = foreign_call.inputs.first() {
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
                    let values: Vec<FieldElement> = (0..n - 1 - arity)
                        .flat_map(|i| {
                            foreign_call
                                .inputs
                                .get(1 + i)
                                .map(|fci| fci.fields())
                                .unwrap_or_default()
                        })
                        .collect();
                    self.debug_vars.assign_field(var_id, indexes, &values);
                }
                Ok(ForeignCallResult::default())
            }
            Some(DebugForeignCall::DerefAssign) => {
                let fcp_var_id = &foreign_call.inputs[0];
                let fcp_value = &foreign_call.inputs[1];
                if let ForeignCallParam::Single(var_id_value) = fcp_var_id {
                    let var_id = debug_var_id(var_id_value);
                    self.debug_vars.assign_deref(var_id, &fcp_value.fields());
                }
                Ok(ForeignCallResult::default())
            }
            Some(DebugForeignCall::FnEnter) => {
                let fcp_fn_id = &foreign_call.inputs[0];
                let ForeignCallParam::Single(fn_id_value) = fcp_fn_id else {
                    panic!("unexpected foreign call parameter in fn enter: {fcp_fn_id:?}")
                };
                let fn_id = debug_fn_id(fn_id_value);
                self.debug_vars.push_fn(fn_id);
                Ok(ForeignCallResult::default())
            }
            Some(DebugForeignCall::FnExit) => {
                self.debug_vars.pop_fn();
                Ok(ForeignCallResult::default())
            }
            None => self.executor.execute(foreign_call),
        }
    }
}
