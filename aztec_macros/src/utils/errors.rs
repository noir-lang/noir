use noirc_errors::Span;
use noirc_frontend::{macros_api::MacroError, UnresolvedTypeData};

use super::constants::MAX_CONTRACT_PRIVATE_FUNCTIONS;

#[derive(Debug, Clone)]
pub enum AztecMacroError {
    AztecDepNotFound,
    ContractHasTooManyPrivateFunctions { span: Span },
    ContractConstructorMissing { span: Span },
    UnsupportedFunctionArgumentType { span: Span, typ: UnresolvedTypeData },
    UnsupportedStorageType { span: Option<Span>, typ: UnresolvedTypeData },
    CouldNotAssignStorageSlots { secondary_message: Option<String> },
    CouldNotImplementNoteSerialization { span: Option<Span>, typ: UnresolvedTypeData },
    EventError { span: Span, message: String },
    UnsupportedAttributes { span: Span, secondary_message: Option<String> },
}

impl From<AztecMacroError> for MacroError {
    fn from(err: AztecMacroError) -> Self {
        match err {
            AztecMacroError::AztecDepNotFound {} => MacroError {
                primary_message: "Aztec dependency not found. Please add aztec as a dependency in your Cargo.toml. For more information go to https://docs.aztec.network/developers/debugging/aztecnr-errors#aztec-dependency-not-found-please-add-aztec-as-a-dependency-in-your-nargotoml".to_owned(),
                secondary_message: None,
                span: None,
            },
            AztecMacroError::ContractHasTooManyPrivateFunctions { span } => MacroError {
                primary_message: format!("Contract can only have a maximum of {} private functions", MAX_CONTRACT_PRIVATE_FUNCTIONS),
                secondary_message: None,
                span: Some(span),
            },
            AztecMacroError::ContractConstructorMissing { span } => MacroError {
                primary_message: "Contract must have a constructor function".to_owned(),
                secondary_message: None,
                span: Some(span),
            },
            AztecMacroError::UnsupportedFunctionArgumentType { span, typ } => MacroError {
                primary_message: format!("Provided parameter type `{typ:?}` is not supported in Aztec contract interface"),
                secondary_message: None,
                span: Some(span),
            },
            AztecMacroError::UnsupportedStorageType { span, typ } => MacroError {
                primary_message: format!("Provided storage type `{typ:?}` is not directly supported in Aztec. Please provide a custom storage implementation"),
                secondary_message: None,
                span,
            },
            AztecMacroError::CouldNotAssignStorageSlots { secondary_message } => MacroError {
                primary_message: "Could not assign storage slots, please provide a custom storage implementation".to_string(),
                secondary_message,
                span: None,
            },
            AztecMacroError::CouldNotImplementNoteSerialization { span, typ } => MacroError {
                primary_message: format!("Could not implement serialization methods for note `{typ:?}`, please provide a serialize_content and deserialize_content methods"),
                secondary_message: None,
                span,
            },
            AztecMacroError::EventError { span, message } => MacroError {
                primary_message: message,
                secondary_message: None,
                span: Some(span),
            },
AztecMacroError::UnsupportedAttributes { span, secondary_message } => MacroError {
                primary_message: "Unsupported attributes in contract function".to_string(),
                secondary_message,
                span: Some(span),
            },
        }
    }
}
