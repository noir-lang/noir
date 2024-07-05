use dep::protocol_types::{
    abis::{function_selector::FunctionSelector, private_circuit_public_inputs::PrivateCircuitPublicInputs},
    address::AztecAddress, traits::Deserialize
};

use crate::context::{
    private_context::PrivateContext, public_context::PublicContext, gas::GasOpts,
    public_context::FunctionReturns, inputs::{PrivateContextInputs, PublicContextInputs}
};

use crate::oracle::arguments::pack_arguments;
use crate::hash::hash_args;

trait CallInterface<N, T, P, Env> {
    fn get_original(self) -> fn[Env](T) -> P;

    fn get_args(self) -> [Field] {
        self.args
    }

    fn get_selector(self) -> FunctionSelector {
        self.selector
    }

    fn get_name(self) -> str<N> {
        self.name
    }

    fn get_contract_address(self) -> AztecAddress {
        self.target_contract
    }

    fn get_is_static(self) -> bool {
        self.is_static
    }
}

impl<N, T, P, Env> CallInterface<N, PrivateContextInputs, PrivateCircuitPublicInputs, Env> for PrivateCallInterface<N, T, Env> {
    fn get_original(self) -> fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs {
        self.original
    }
}

struct PrivateCallInterface<N, T, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args_hash: Field,
    args: [Field],
    original: fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs,
    is_static: bool
}

impl<N, T, Env> PrivateCallInterface<N, T, Env> {
    pub fn call<M>(self, context: &mut PrivateContext) -> T where T: Deserialize<M> {
        assert(self.args_hash == pack_arguments(self.args));
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

    pub fn view<M>(self, context: &mut PrivateContext) -> T where T: Deserialize<M> {
        assert(self.args_hash == pack_arguments(self.args));
        let returns = context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false);
        returns.unpack_into()
    }

    pub fn delegate_call<M>(self, context: &mut PrivateContext) -> T where T: Deserialize<M> {
        assert(self.args_hash == pack_arguments(self.args));
        let returns = context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, false, true);
        returns.unpack_into()
    }
}

impl<N, T, P, Env> CallInterface<N, PrivateContextInputs, PrivateCircuitPublicInputs, Env> for PrivateVoidCallInterface<N, Env> {
    fn get_original(self) -> fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs {
        self.original
    }
}

struct PrivateVoidCallInterface<N, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args_hash: Field,
    args: [Field],
    original: fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs,
    is_static: bool
}

impl<N, Env> PrivateVoidCallInterface<N, Env> {
    pub fn call(self, context: &mut PrivateContext) {
        assert(self.args_hash == pack_arguments(self.args));
        context.call_private_function_with_packed_args(
            self.target_contract,
            self.selector,
            self.args_hash,
            false,
            false
        ).assert_empty();
    }

    pub fn view(self, context: &mut PrivateContext) {
        assert(self.args_hash == pack_arguments(self.args));
        context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false).assert_empty();
    }

    pub fn delegate_call(self, context: &mut PrivateContext) {
        assert(self.args_hash == pack_arguments(self.args));
        context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, false, true).assert_empty();
    }
}

impl<N, T, P, Env> CallInterface<N, PrivateContextInputs, PrivateCircuitPublicInputs, Env> for PrivateStaticCallInterface<N, T, Env> {
    fn get_original(self) -> fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs {
        self.original
    }
}

struct PrivateStaticCallInterface<N, T, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args_hash: Field,
    args: [Field],
    original: fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs,
    is_static: bool
}

impl<N, T, Env> PrivateStaticCallInterface<N, T, Env> {
    pub fn view<M>(self, context: &mut PrivateContext) -> T where T: Deserialize<M> {
        assert(self.args_hash == pack_arguments(self.args));
        let returns = context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false);
        returns.unpack_into()
    }
}

impl<N, T, P, Env> CallInterface<N, PrivateContextInputs, PrivateCircuitPublicInputs, Env> for PrivateStaticVoidCallInterface<N, Env> {
    fn get_original(self) -> fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs {
        self.original
    }
}

struct PrivateStaticVoidCallInterface<N, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args_hash: Field,
    args: [Field],
    original: fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs,
    is_static: bool
}

impl<N, Env> PrivateStaticVoidCallInterface<N, Env> {
    pub fn view(self, context: &mut PrivateContext) {
        assert(self.args_hash == pack_arguments(self.args));
        context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false).assert_empty();
    }
}

impl<N, T, P, Env> CallInterface<N, PublicContextInputs, T, Env> for PublicCallInterface<N, T, Env> {
    fn get_original(self) -> fn[Env](PublicContextInputs) -> T {
        self.original
    }
}

struct PublicCallInterface<N, T, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args: [Field],
    gas_opts: GasOpts,
    original: fn[Env](PublicContextInputs) -> T,
    is_static: bool
}

impl<N, T, Env> PublicCallInterface<N, T, Env> {
    pub fn with_gas(self: &mut Self, gas_opts: GasOpts) -> &mut Self {
        self.gas_opts = gas_opts;
        self
    }

    pub fn call<M>(self, context: &mut PublicContext) -> T where T: Deserialize<M> {
        let returns = context.call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.deserialize_into()
    }

    pub fn view<M>(self, context: &mut PublicContext) -> T where T: Deserialize<M> {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.deserialize_into()
    }

    pub fn delegate_call<M>(self, context: &mut PublicContext) -> T where T: Deserialize<M> {
        let returns = context.delegate_call_public_function(self.target_contract, self.selector, self.args);
        returns.deserialize_into()
    }

    pub fn enqueue(self, context: &mut PrivateContext) {
        let args_hash = hash_args(self.args);
        assert(args_hash == pack_arguments(self.args));
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ false,
            /*delegate=*/ false
        )
    }

    pub fn enqueue_view(self, context: &mut PrivateContext) {
        let args_hash = hash_args(self.args);
        assert(args_hash == pack_arguments(self.args));
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ true,
            /*delegate=*/ false
        )
    }

    pub fn delegate_enqueue(self, context: &mut PrivateContext) {
        let args_hash = hash_args(self.args);
        assert(args_hash == pack_arguments(self.args));
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ false,
            /*delegate=*/ true
        )
    }
}

impl<N, T, P, Env> CallInterface<N, PublicContextInputs, (), Env> for PublicVoidCallInterface<N, Env> {
    fn get_original(self) -> fn[Env](PublicContextInputs) -> () {
        self.original
    }
}

struct PublicVoidCallInterface<N, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args: [Field],
    original: fn[Env](PublicContextInputs) -> (),
    is_static: bool,
    gas_opts: GasOpts
}

impl<N, Env> PublicVoidCallInterface<N, Env> {
    pub fn with_gas(self: &mut Self, gas_opts: GasOpts) -> &mut Self {
        self.gas_opts = gas_opts;
        self
    }

    pub fn call(self, context: &mut PublicContext) {
        let returns = context.call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.assert_empty()
    }

    pub fn view(self, context: &mut PublicContext) {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.assert_empty()
    }

    pub fn delegate_call(self, context: &mut PublicContext) {
        let returns = context.delegate_call_public_function(self.target_contract, self.selector, self.args);
        returns.assert_empty()
    }

    pub fn enqueue(self, context: &mut PrivateContext) {
        let args_hash = hash_args(self.args);
        assert(args_hash == pack_arguments(self.args));
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ false,
            /*delegate=*/ false
        )
    }

    pub fn enqueue_view(self, context: &mut PrivateContext) {
        let args_hash = hash_args(self.args);
        assert(args_hash == pack_arguments(self.args));
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ true,
            /*delegate=*/ false
        )
    }

    pub fn delegate_enqueue(self, context: &mut PrivateContext) {
        let args_hash = hash_args(self.args);
        assert(args_hash == pack_arguments(self.args));
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ false,
            /*delegate=*/ true
        )
    }
}

impl<N, T, P, Env> CallInterface<N, PublicContextInputs, T, Env> for PublicStaticCallInterface<N, T, Env> {
    fn get_original(self) -> fn[Env](PublicContextInputs) -> T {
        self.original
    }
}

struct PublicStaticCallInterface<N, T, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args: [Field],
    original: fn[Env](PublicContextInputs) -> T,
    is_static: bool,
    gas_opts: GasOpts
}

impl<N, T, Env> PublicStaticCallInterface<N, T, Env> {
    pub fn with_gas(self: &mut Self, gas_opts: GasOpts) -> &mut Self {
        self.gas_opts = gas_opts;
        self
    }

    pub fn view<M>(self, context: &mut PublicContext) -> T where T: Deserialize<M> {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        let unpacked: T = returns.deserialize_into();
        unpacked
    }

    pub fn enqueue_view(self, context: &mut PrivateContext) {
        let args_hash = hash_args(self.args);
        assert(args_hash == pack_arguments(self.args));
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ true,
            /*delegate=*/ false
        )
    }
}

impl<N, T, P, Env> CallInterface<N, PublicContextInputs, (), Env> for PublicStaticVoidCallInterface<N, Env> {
    fn get_original(self) -> fn[Env](PublicContextInputs) -> () {
        self.original
    }
}

struct PublicStaticVoidCallInterface<N, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args: [Field],
    original: fn[Env](PublicContextInputs) -> (),
    is_static: bool,
    gas_opts: GasOpts
}

impl<N, Env> PublicStaticVoidCallInterface<N, Env> {
    pub fn with_gas(self: &mut Self, gas_opts: GasOpts) -> &mut Self {
        self.gas_opts = gas_opts;
        self
    }

    pub fn view(self, context: &mut PublicContext) {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.assert_empty()
    }

    pub fn enqueue_view(self, context: &mut PrivateContext) {
        let args_hash = hash_args(self.args);
        assert(args_hash == pack_arguments(self.args));
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            args_hash,
            /*static=*/ true,
            /*delegate=*/ false
        )
    }
}
