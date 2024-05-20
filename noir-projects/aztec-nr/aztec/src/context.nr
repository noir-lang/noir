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

struct Context {
    private: Option<&mut PrivateContext>,
    public: Option<&mut PublicContext>,
    avm: Option<&mut AvmContext>,
}

impl Context {
    pub fn private(context: &mut PrivateContext) -> Context {
        Context { private: Option::some(context), public: Option::none(), avm: Option::none() }
    }

    pub fn public(context: &mut PublicContext) -> Context {
        Context { public: Option::some(context), private: Option::none(), avm: Option::none() }
    }

    pub fn avm(context: &mut AvmContext) -> Context {
        Context { avm: Option::some(context), public: Option::none(), private: Option::none() }
    }

    pub fn none() -> Context {
        Context { public: Option::none(), private: Option::none(), avm: Option::none() }
    }
}
