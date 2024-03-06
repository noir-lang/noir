mod globals;
mod inputs;

mod private_context;
mod public_context;
mod interface;
mod avm;

use private_context::PrivateContext;
use interface::ContextInterface;
use public_context::PublicContext;
use avm::AVMContext;

struct Context {
    private: Option<&mut PrivateContext>,
    public: Option<&mut PublicContext>,
    public_vm: Option<&mut AVMContext>,
}

impl Context {
    pub fn private(context: &mut PrivateContext) -> Context {
        Context { private: Option::some(context), public: Option::none(), public_vm: Option::none() }
    }

    pub fn public(context: &mut PublicContext) -> Context {
        Context { public: Option::some(context), private: Option::none(), public_vm: Option::none() }
    }

    pub fn public_vm(context: &mut AVMContext) -> Context {
        Context { public_vm: Option::some(context), public: Option::none(), private: Option::none() }
    }

    pub fn none() -> Context {
        Context { public: Option::none(), private: Option::none(), public_vm: Option::none() }
    }
}
