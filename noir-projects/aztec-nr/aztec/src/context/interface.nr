use dep::protocol_types::{
    abis::function_selector::FunctionSelector, address::{AztecAddress, EthAddress}, header::Header,
    traits::Deserialize
};

use crate::context::private_context::PrivateContext;
use crate::context::public_context::PublicContext;
use crate::context::avm_context::AvmContext;
use crate::context::gas::GasOpts;
use crate::context::public_context::FunctionReturns;

trait ContextInterface {
    fn push_new_note_hash(&mut self, note_hash: Field);
    fn push_new_nullifier(&mut self, nullifier: Field, nullified_commitment: Field);
    fn msg_sender(self) -> AztecAddress;
    fn this_address(self) -> AztecAddress;
    fn chain_id(self) -> Field;
    fn version(self) -> Field;
    fn selector(self) -> FunctionSelector;
    fn get_args_hash(self) -> Field;
}

// TEMPORARY: This trait is to promote sharing of the current public context
// and the upcoming AvmContext. This will be removed once the AvmContext is the default.
// If you are adding something here, then you should also implement it in the AvmContext.
trait PublicContextInterface {
    fn block_number(self) -> Field;
    fn timestamp(self) -> u64;
    fn coinbase(self) -> EthAddress;
    fn fee_recipient(self) -> AztecAddress;
    fn fee_per_da_gas(self) -> Field;
    fn fee_per_l1_gas(self) -> Field;
    fn fee_per_l2_gas(self) -> Field;
    fn message_portal(&mut self, recipient: EthAddress, content: Field);
    fn consume_l1_to_l2_message(&mut self, content: Field, secret: Field, sender: EthAddress, leaf_index: Field);
    fn emit_unencrypted_log<T>(&mut self, log: T);
    fn call_public_function<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field],
        gas_opts: GasOpts
    ) -> FunctionReturns<RETURNS_COUNT>;
    fn static_call_public_function<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field],
        gas_opts: GasOpts
    ) -> FunctionReturns<RETURNS_COUNT>;
    fn delegate_call_public_function<RETURNS_COUNT>(
        self: &mut Self,
        contract_address: AztecAddress,
        function_selector: FunctionSelector,
        args: [Field]
    ) -> FunctionReturns<RETURNS_COUNT>;
    fn nullifier_exists(self, unsiloed_nullifier: Field, address: AztecAddress) -> bool;
}

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

    pub fn static_call<N>(self, context: &mut PrivateContext) -> T where T: Deserialize<N> {
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

    pub fn static_call(self, context: &mut PrivateContext) {
        context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false).assert_empty();
    }

    pub fn delegate_call(self, context: &mut PrivateContext) {
        context.call_private_function_with_packed_args(self.target_contract, self.selector, self.args_hash, false, true).assert_empty();
    }
}

struct PublicCallInterface<T> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args_hash: Field,
}

impl<T> PublicCallInterface<T> {

    pub fn call<N>(self, context: &mut PublicContext) -> T where T: Deserialize<N> {
        let returns = context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            self.args_hash,
            false,
            false
        );
        returns.deserialize_into()
    }

    pub fn static_call<N>(self, context: &mut PublicContext) -> T where T: Deserialize<N> {
        let returns = context.call_public_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false);
        returns.deserialize_into()
    }

    pub fn delegate_call<N>(self, context: &mut PublicContext) -> T where T: Deserialize<N> {
        let returns = context.call_public_function_with_packed_args(self.target_contract, self.selector, self.args_hash, false, true);
        returns.deserialize_into()
    }

    pub fn enqueue(self, context: &mut PrivateContext) {
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            self.args_hash,
            false,
            false
        )
    }

    pub fn static_enqueue(self, context: &mut PrivateContext) {
        context.call_public_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false)
    }

    pub fn delegate_enqueue(self, context: &mut PrivateContext) {
        context.call_public_function_with_packed_args(self.target_contract, self.selector, self.args_hash, false, true)
    }
}

struct PublicVoidCallInterface {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args_hash: Field
}

impl PublicVoidCallInterface {
    pub fn call(self, context: &mut PublicContext) {
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            self.args_hash,
            false,
            false
        ).assert_empty()
    }

    pub fn static_call(self, context: &mut PublicContext) {
        context.call_public_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false).assert_empty();
    }

    pub fn delegate_call(self, context: &mut PublicContext) {
        context.call_public_function_with_packed_args(self.target_contract, self.selector, self.args_hash, false, true).assert_empty();
    }

    pub fn enqueue(self, context: &mut PrivateContext) {
        context.call_public_function_with_packed_args(
            self.target_contract,
            self.selector,
            self.args_hash,
            false,
            false
        )
    }

    pub fn static_enqueue(self, context: &mut PrivateContext) {
        context.call_public_function_with_packed_args(self.target_contract, self.selector, self.args_hash, true, false)
    }

    pub fn delegate_enqueue(self, context: &mut PrivateContext) {
        context.call_public_function_with_packed_args(self.target_contract, self.selector, self.args_hash, false, true)
    }
}

struct AvmCallInterface<T> {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args: [Field],
}

impl<T> AvmCallInterface<T> {
    pub fn call<N>(self, context: &mut AvmContext, gas_opts: GasOpts) -> T where T: Deserialize<N> {
        let returns = context.call_public_function(self.target_contract, self.selector, self.args, gas_opts);
        returns.deserialize_into()
    }

    pub fn static_call<N>(
        self,
        context: &mut AvmContext,
        gas_opts: GasOpts
    ) -> T where T: Deserialize<N> {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, gas_opts);
        returns.deserialize_into()
    }

    pub fn delegate_call<N>(self, context: &mut AvmContext) -> T where T: Deserialize<N> {
        let returns = context.delegate_call_public_function(self.target_contract, self.selector, self.args);
        returns.deserialize_into()
    }
}

struct AvmVoidCallInterface {
    target_contract: AztecAddress,
    selector: FunctionSelector,
    args: [Field],
}

impl AvmVoidCallInterface {
    pub fn call<N>(self, context: &mut AvmContext, gas_opts: GasOpts) {
        let returns = context.call_public_function(self.target_contract, self.selector, self.args, gas_opts);
        returns.assert_empty()
    }

    pub fn static_call<N>(self, context: &mut AvmContext, gas_opts: GasOpts) {
        let returns = context.static_call_public_function(self.target_contract, self.selector, self.args, gas_opts);
        returns.assert_empty()
    }

    pub fn delegate_call<N>(self, context: &mut AvmContext) {
        let returns = context.delegate_call_public_function(self.target_contract, self.selector, self.args);
        returns.assert_empty()
    }
}
