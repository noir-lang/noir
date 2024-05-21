use dep::protocol_types::{abis::function_selector::FunctionSelector, address::AztecAddress, traits::Deserialize};

use crate::context::private_context::PrivateContext;
use crate::context::public_context::PublicContext;
use crate::context::gas::GasOpts;
use crate::context::public_context::FunctionReturns;

use crate::oracle::arguments;

struct PrivateCallInterface<T> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args_hash: Field,
}

impl<T> PrivateCallInterface<T> {
    pub fn call<N>(self, context: &mut PrivateContext) -> T where T: Deserialize<N> {
        let returns = context.call_private_function_with_packed_args(
            self.target_contract,
            self.selector,
            self.args_hash,
            false,
            false
        );
        let unpacked: T = returns.unpack_into();
        unpacked
    }

    pub fn view<N>(self, context: &mut PrivateContext) -> T where T: Deserialize<N> {
        let returns = context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false);
        returns.unpack_into()
    }

    pub fn delegate_call<N>(self, context: &mut PrivateContext) -> T where T: Deserialize<N> {
        let returns = context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, false, true);
        returns.unpack_into()
    }
}

struct PrivateVoidCallInterface {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args_hash: Field,
}

impl PrivateVoidCallInterface {
    pub fn call(self, context: &mut PrivateContext) {
        context.call_private_function_with_packed_args(
            self.target_contract,
            self.selector,
            self.args_hash,
            false,
            false
        ).assert_empty();
    }

    pub fn view(self, context: &mut PrivateContext) {
        context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false).assert_empty();
    }

    pub fn delegate_call(self, context: &mut PrivateContext) {
        context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, false, true).assert_empty();
    }
}

struct PrivateStaticCallInterface<T> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args_hash: Field,
}

impl<T> PrivateStaticCallInterface<T> {
    pub fn view<N>(self, context: &mut PrivateContext) -> T where T: Deserialize<N> {
        let returns = context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false);
        returns.unpack_into()
    }
}

struct PrivateStaticVoidCallInterface {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args_hash: Field,
}

impl PrivateStaticVoidCallInterface {
    pub fn view(self, context: &mut PrivateContext) {
        context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false).assert_empty();
    }
}

struct PublicCallInterface<T> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args: [Field],
    gas_opts: GasOpts,
}

impl<T> PublicCallInterface<T> {
    pub fn with_gas(self: &mut Self, gas_opts: GasOpts) -> &mut Self {
        self.gas_opts = gas_opts;
        self
    }

    pub fn call<N>(self, context: &mut PublicContext) -> T where T: Deserialize<N> {
        let returns = context.call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.deserialize_into()
    }

    pub fn view<N>(self, context: &mut PublicContext) -> T where T: Deserialize<N> {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.deserialize_into()
    }

    pub fn delegate_call<N>(self, context: &mut PublicContext) -> T where T: Deserialize<N> {
        let returns = context.delegate_call_public_function(self.target_contract, self.selector, self.args);
        returns.deserialize_into()
    }

    pub fn enqueue(self, context: &mut PrivateContext) {
        // This packing is only here because PrivateContext's call_public* functions do not accept a slice for the args.
        let args_hash = arguments::pack_arguments(self.args);
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ false,
            /*delegate=*/ false
        )
    }

    pub fn enqueue_view(self, context: &mut PrivateContext) {
        // This packing is only here because PrivateContext's call_public* functions do not accept a slice for the args.
        let args_hash = arguments::pack_arguments(self.args);
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ true,
            /*delegate=*/ false
        )
    }

    pub fn delegate_enqueue(self, context: &mut PrivateContext) {
        // This packing is only here because PrivateContext's call_public* functions do not accept a slice for the args.
        let args_hash = arguments::pack_arguments(self.args);
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ false,
            /*delegate=*/ true
        )
    }
}

struct PublicVoidCallInterface {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args: [Field],
    gas_opts: GasOpts,
}

impl PublicVoidCallInterface {
    pub fn with_gas(self: &mut Self, gas_opts: GasOpts) -> &mut Self {
        self.gas_opts = gas_opts;
        self
    }

    pub fn call<N>(self, context: &mut PublicContext) {
        let returns = context.call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.assert_empty()
    }

    pub fn view<N>(self, context: &mut PublicContext) {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.assert_empty()
    }

    pub fn delegate_call<N>(self, context: &mut PublicContext) {
        let returns = context.delegate_call_public_function(self.target_contract, self.selector, self.args);
        returns.assert_empty()
    }

    pub fn enqueue(self, context: &mut PrivateContext) {
        // This packing is only here because PrivateContext's call_public* functions do not accept a slice for the args.
        let args_hash = arguments::pack_arguments(self.args);
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ false,
            /*delegate=*/ false
        )
    }

    pub fn enqueue_view(self, context: &mut PrivateContext) {
        // This packing is only here because PrivateContext's call_public* functions do not accept a slice for the args.
        let args_hash = arguments::pack_arguments(self.args);
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ true,
            /*delegate=*/ false
        )
    }

    pub fn delegate_enqueue(self, context: &mut PrivateContext) {
        // This packing is only here because PrivateContext's call_public* functions do not accept a slice for the args.
        let args_hash = arguments::pack_arguments(self.args);
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ false,
            /*delegate=*/ true
        )
    }
}

struct PublicStaticCallInterface<T> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args: [Field],
    gas_opts: GasOpts,
}

impl<T> PublicStaticCallInterface<T> {
    pub fn with_gas(self: &mut Self, gas_opts: GasOpts) -> &mut Self {
        self.gas_opts = gas_opts;
        self
    }

    pub fn view<N>(self, context: &mut PublicContext) -> T where T: Deserialize<N> {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.deserialize_into()
    }

    pub fn enqueue_view(self, context: &mut PrivateContext) {
        // This packing is only here because PrivateContext's call_public* functions do not accept a slice for the args.
        let args_hash = arguments::pack_arguments(self.args);
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ true,
            /*delegate=*/ false
        )
    }
}

struct PublicStaticVoidCallInterface {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args: [Field],
    gas_opts: GasOpts,
}

impl PublicStaticVoidCallInterface {
    pub fn with_gas(self: &mut Self, gas_opts: GasOpts) -> &mut Self {
        self.gas_opts = gas_opts;
        self
    }

    pub fn view<N>(self, context: &mut PublicContext) {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.assert_empty()
    }

    pub fn enqueue_view(self, context: &mut PrivateContext) {
        // This packing is only here because PrivateContext's call_public* functions do not accept a slice for the args.
        let args_hash = arguments::pack_arguments(self.args);
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ true,
            /*delegate=*/ false
        )
    }
}
