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

trait CallInterface<let N: u32, T, P, Env> {
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

impl<let N: u32, T, P, Env> CallInterface<N, PrivateContextInputs, PrivateCircuitPublicInputs, Env> for PrivateCallInterface<N, T, Env> {
    fn get_original(self) -> fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs {
        self.original
    }
}

struct PrivateCallInterface<let N: u32, T, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args_hash: Field,
    args: [Field],
    original: fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs,
    is_static: bool
}

impl<let N: u32, T, Env> PrivateCallInterface<N, T, Env> {
    pub fn call<let M: u32>(self, context: &mut PrivateContext) -> T where T: Deserialize<M> {
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

    pub fn view<let M: u32>(self, context: &mut PrivateContext) -> T where T: Deserialize<M> {
        assert(self.args_hash == pack_arguments(self.args));
        let returns = context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false);
        returns.unpack_into()
    }

    pub fn delegate_call<let M: u32>(self, context: &mut PrivateContext) -> T where T: Deserialize<M> {
        assert(self.args_hash == pack_arguments(self.args));
        let returns = context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, false, true);
        returns.unpack_into()
    }
}

impl<let N: u32, T, P, Env> CallInterface<N, PrivateContextInputs, PrivateCircuitPublicInputs, Env> for PrivateVoidCallInterface<N, Env> {
    fn get_original(self) -> fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs {
        self.original
    }
}

struct PrivateVoidCallInterface<let N: u32, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args_hash: Field,
    args: [Field],
    original: fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs,
    is_static: bool
}

impl<let N: u32, Env> PrivateVoidCallInterface<N, Env> {
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

impl<let N: u32, T, P, Env> CallInterface<N, PrivateContextInputs, PrivateCircuitPublicInputs, Env> for PrivateStaticCallInterface<N, T, Env> {
    fn get_original(self) -> fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs {
        self.original
    }
}

struct PrivateStaticCallInterface<let N: u32, T, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args_hash: Field,
    args: [Field],
    original: fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs,
    is_static: bool
}

impl<let N: u32, T, Env> PrivateStaticCallInterface<N, T, Env> {
    pub fn view<let M: u32>(self, context: &mut PrivateContext) -> T where T: Deserialize<M> {
        assert(self.args_hash == pack_arguments(self.args));
        let returns = context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false);
        returns.unpack_into()
    }
}

impl<let N: u32, T, P, Env> CallInterface<N, PrivateContextInputs, PrivateCircuitPublicInputs, Env> for PrivateStaticVoidCallInterface<N, Env> {
    fn get_original(self) -> fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs {
        self.original
    }
}

struct PrivateStaticVoidCallInterface<let N: u32, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args_hash: Field,
    args: [Field],
    original: fn[Env](PrivateContextInputs) -> PrivateCircuitPublicInputs,
    is_static: bool
}

impl<let N: u32, Env> PrivateStaticVoidCallInterface<N, Env> {
    pub fn view(self, context: &mut PrivateContext) {
        assert(self.args_hash == pack_arguments(self.args));
        context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false).assert_empty();
    }
}

impl<let N: u32, T, P, Env> CallInterface<N, PublicContextInputs, T, Env> for PublicCallInterface<N, T, Env> {
    fn get_original(self) -> fn[Env](PublicContextInputs) -> T {
        self.original
    }
}

struct PublicCallInterface<let N: u32, T, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args: [Field],
    gas_opts: GasOpts,
    original: fn[Env](PublicContextInputs) -> T,
    is_static: bool
}

impl<let N: u32, T, Env> PublicCallInterface<N, T, Env> {
    pub fn with_gas(self: &mut Self, gas_opts: GasOpts) -> &mut Self {
        self.gas_opts = gas_opts;
        self
    }

    pub fn call<let M: u32>(self, context: &mut PublicContext) -> T where T: Deserialize<M> {
        let returns = context.call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.deserialize_into()
    }

    pub fn view<let M: u32>(self, context: &mut PublicContext) -> T where T: Deserialize<M> {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, self.gas_opts);
        returns.deserialize_into()
    }

    pub fn delegate_call<let M: u32>(self, context: &mut PublicContext) -> T where T: Deserialize<M> {
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

impl<let N: u32, T, P, Env> CallInterface<N, PublicContextInputs, (), Env> for PublicVoidCallInterface<N, Env> {
    fn get_original(self) -> fn[Env](PublicContextInputs) -> () {
        self.original
    }
}

struct PublicVoidCallInterface<let N: u32, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args: [Field],
    original: fn[Env](PublicContextInputs) -> (),
    is_static: bool,
    gas_opts: GasOpts
}

impl<let N: u32, Env> PublicVoidCallInterface<N, Env> {
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

impl<let N: u32, T, P, Env> CallInterface<N, PublicContextInputs, T, Env> for PublicStaticCallInterface<N, T, Env> {
    fn get_original(self) -> fn[Env](PublicContextInputs) -> T {
        self.original
    }
}

struct PublicStaticCallInterface<let N: u32, T, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args: [Field],
    original: fn[Env](PublicContextInputs) -> T,
    is_static: bool,
    gas_opts: GasOpts
}

impl<let N: u32, T, Env> PublicStaticCallInterface<N, T, Env> {
    pub fn with_gas(self: &mut Self, gas_opts: GasOpts) -> &mut Self {
        self.gas_opts = gas_opts;
        self
    }

    pub fn view<let M: u32>(self, context: &mut PublicContext) -> T where T: Deserialize<M> {
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

impl<let N: u32, T, P, Env> CallInterface<N, PublicContextInputs, (), Env> for PublicStaticVoidCallInterface<N, Env> {
    fn get_original(self) -> fn[Env](PublicContextInputs) -> () {
        self.original
    }
}

struct PublicStaticVoidCallInterface<let N: u32, Env> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    name: str<N>,
    args: [Field],
    original: fn[Env](PublicContextInputs) -> (),
    is_static: bool,
    gas_opts: GasOpts
}

impl<let N: u32, Env> PublicStaticVoidCallInterface<N, Env> {
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
