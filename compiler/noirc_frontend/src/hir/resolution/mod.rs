//! This set of modules implements the second half of the name resolution pass.
//! After all definitions are collected by def_collector, resolver::Resolvers are
//! created to resolve all names within a definition. In this context 'resolving'
//! a name means validating that it has a valid definition, and that it was not
//! redefined multiple times in the same scope. Once this is validated, it is linked
//! to that definition via a matching DefinitionId. All references to the same definition
//! will have the same DefinitionId.
pub mod errors;
pub mod import;
pub mod visibility;
