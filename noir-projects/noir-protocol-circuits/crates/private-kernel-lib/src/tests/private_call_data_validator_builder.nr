use crate::private_call_data_validator::PrivateCallDataValidator;
use dep::types::{
    abis::call_request::CallRequest, tests::private_call_data_builder::PrivateCallDataBuilder,
    transaction::tx_request::TxRequest
};

struct PrivateCallDataValidatorBuilder {
    private_call: PrivateCallDataBuilder,
    first_revertible_private_call_request_index: u64,
    first_revertible_public_call_request_index: u64
}

impl PrivateCallDataValidatorBuilder {
    pub fn new() -> Self {
        let default_counter_start = 23;
        PrivateCallDataValidatorBuilder::new_with_counter(default_counter_start)
    }

    pub fn new_with_counter(counter: u32) -> Self {
        let private_call = PrivateCallDataBuilder::new_with_counter(counter);
        PrivateCallDataValidatorBuilder {
            private_call,
            first_revertible_private_call_request_index: 0,
            first_revertible_public_call_request_index: 0
        }
    }

    pub fn is_delegate_call(&mut self) -> Self {
        let _ = self.private_call.is_delegate_call();
        *self
    }

    pub fn is_static_call(&mut self) -> Self {
        let _ = self.private_call.is_static_call();
        *self
    }

    pub fn validate(self) {
        let private_call = self.private_call.finish();
        PrivateCallDataValidator::new(private_call).validate();
    }

    pub fn validate_as_first_call(self) {
        let private_call = self.private_call.finish();
        PrivateCallDataValidator::new(private_call).validate_as_first_call(
            self.first_revertible_private_call_request_index,
            self.first_revertible_public_call_request_index
        );
    }

    pub fn validate_against_tx_request(self, request: TxRequest) {
        let private_call = self.private_call.finish();
        PrivateCallDataValidator::new(private_call).validate_against_tx_request(request);
    }

    pub fn validate_against_call_request(self, request: CallRequest) {
        let private_call = self.private_call.finish();
        PrivateCallDataValidator::new(private_call).validate_against_call_request(request);
    }
}
