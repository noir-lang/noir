use noirc_errors::Span;
use noirc_frontend::ast;
use noirc_frontend::macros_api::MacroError;

use super::constants::MAX_CONTRACT_PRIVATE_FUNCTIONS;

#[derive(Debug, Clone)]
pub enum AztecMacroError {
    AztecDepNotFound,
    ContractHasTooManyPrivateFunctions { span: Span },
    UnsupportedFunctionArgumentType { span: Span, typ: ast::UnresolvedTypeData },
    UnsupportedFunctionReturnType { span: Span, typ: ast::UnresolvedTypeData },
    UnsupportedStorageType { span: Option<Span>, typ: ast::UnresolvedTypeData },
    CouldNotAssignStorageSlots { secondary_message: Option<String> },
    CouldNotImplementComputeNoteHashAndNullifier { secondary_message: Option<String> },
    CouldNotImplementNoteInterface { span: Option<Span>, secondary_message: Option<String> },
    MultipleStorageDefinitions { span: Option<Span> },
    CouldNotExportStorageLayout { span: Option<Span>, secondary_message: Option<String> },
    CouldNotExportFunctionAbi { span: Option<Span>, secondary_message: Option<String> },
    CouldNotGenerateContractInterface { secondary_message: Option<String> },
    EventError { span: Span, message: String },
    UnsupportedAttributes { span: Span, secondary_message: Option<String> },
    PublicArgsDisallowed { span: Span },
}

impl From<AztecMacroError> for MacroError {
    fn from(err: AztecMacroError) -> Self {
        match err {
            AztecMacroError::AztecDepNotFound {} => MacroError {
                primary_message: "Aztec dependency not found. Please add aztec as a dependency in your Nargo.toml. For more information go to https://docs.aztec.network/developers/debugging/aztecnr-errors#aztec-dependency-not-found-please-add-aztec-as-a-dependency-in-your-nargotoml".to_owned(),
                secondary_message: None,
                span: None,
            },
            AztecMacroError::ContractHasTooManyPrivateFunctions { span } => MacroError {
                primary_message: format!("Contract can only have a maximum of {} private functions", MAX_CONTRACT_PRIVATE_FUNCTIONS),
                secondary_message: None,
                span: Some(span),
            },
            AztecMacroError::UnsupportedFunctionArgumentType { span, typ } => MacroError {
                primary_message: format!("Provided parameter type `{typ:?}` is not supported in Aztec contract interface"),
                secondary_message: None,
                span: Some(span),
            },
            AztecMacroError::UnsupportedFunctionReturnType { span, typ } => MacroError {
                primary_message: format!("Provided return type `{typ:?}` is not supported in Aztec contract interface"),
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
            AztecMacroError::CouldNotImplementComputeNoteHashAndNullifier { secondary_message } => MacroError {
                primary_message: "Could not implement compute_note_hash_and_nullifier automatically, please provide an implementation".to_string(),
                secondary_message,
                span: None,
            },
            AztecMacroError::CouldNotImplementNoteInterface { span, secondary_message } => MacroError {
                primary_message: "Could not implement automatic methods for note, please provide an implementation of the NoteInterface trait".to_string(),
                secondary_message,
                span
            },
            AztecMacroError::MultipleStorageDefinitions { span } => MacroError {
                primary_message: "Only one struct can be tagged as #[aztec(storage)]".to_string(),
                secondary_message: None,
                span,
            },
            AztecMacroError::CouldNotExportStorageLayout { secondary_message, span } => MacroError {
                primary_message: "Could not generate and export storage layout".to_string(),
                secondary_message,
                span,
            },
            AztecMacroError::CouldNotExportFunctionAbi { secondary_message, span } => MacroError {
                primary_message: "Could not generate and export function abi".to_string(),
                secondary_message,
                span,
            },
            AztecMacroError::CouldNotGenerateContractInterface { secondary_message } => MacroError {
                primary_message: "Could not generate contract interface".to_string(),
                secondary_message,
                span: None
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
            AztecMacroError::PublicArgsDisallowed { span } => MacroError {
                primary_message: "Aztec functions can't have public arguments".to_string(),
                secondary_message: None,
                span: Some(span),
            },
        }
    }
}
