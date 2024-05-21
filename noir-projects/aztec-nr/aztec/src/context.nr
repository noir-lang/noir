mod globals;
mod inputs;

mod private_context;
mod public_context;
mod avm_context;
mod interface;
mod call_interfaces;
mod gas;

use interface::ContextInterface;
use call_interfaces::{
    PrivateCallInterface, PrivateStaticCallInterface, PublicCallInterface, PublicStaticCallInterface,
    PrivateVoidCallInterface, PrivateStaticVoidCallInterface, PublicVoidCallInterface,
    PublicStaticVoidCallInterface, AvmCallInterface, AvmStaticCallInterface, AvmVoidCallInterface,
    AvmStaticVoidCallInterface
};
use private_context::PrivateContext;
use private_context::PackedReturns;
use public_context::PublicContext;
use public_context::FunctionReturns;
use avm_context::AvmContext;
