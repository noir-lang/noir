use crate::types::{
    abis::{
        accumulated_data::private_to_public_accumulated_data::PrivateToPublicAccumulatedData,
        log_hash::LogHash, private_log::PrivateLog, public_call_request::PublicCallRequest,
        side_effect::scoped::Scoped,
    },
    constants::{
        MAX_CONTRACT_CLASS_LOGS_PER_TX, MAX_ENQUEUED_CALLS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX,
        MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_PRIVATE_LOGS_PER_TX,
    },
    messaging::l2_to_l1_message::ScopedL2ToL1Message,
    traits::Empty,
    utils::arrays::array_to_bounded_vec,
};

pub(crate) struct PrivateToPublicAccumulatedDataBuilder {
    pub(crate) note_hashes: BoundedVec<Field, MAX_NOTE_HASHES_PER_TX>,
    pub(crate) nullifiers: BoundedVec<Field, MAX_NULLIFIERS_PER_TX>,
    pub(crate) l2_to_l1_msgs: BoundedVec<ScopedL2ToL1Message, MAX_L2_TO_L1_MSGS_PER_TX>,
    pub(crate) private_logs: BoundedVec<PrivateLog, MAX_PRIVATE_LOGS_PER_TX>,
    pub(crate) contract_class_logs_hashes: BoundedVec<Scoped<LogHash>, MAX_CONTRACT_CLASS_LOGS_PER_TX>,
    pub(crate) public_call_requests: BoundedVec<PublicCallRequest, MAX_ENQUEUED_CALLS_PER_TX>,
}

impl PrivateToPublicAccumulatedDataBuilder {
    pub(crate) unconstrained fn new(data: PrivateToPublicAccumulatedData) -> Self {
        PrivateToPublicAccumulatedDataBuilder {
            note_hashes: array_to_bounded_vec(data.note_hashes),
            nullifiers: array_to_bounded_vec(data.nullifiers),
            l2_to_l1_msgs: array_to_bounded_vec(data.l2_to_l1_msgs),
            private_logs: array_to_bounded_vec(data.private_logs),
            contract_class_logs_hashes: array_to_bounded_vec(data.contract_class_logs_hashes),
            public_call_requests: array_to_bounded_vec(data.public_call_requests),
        }
    }

    pub(crate) fn finish(self) -> PrivateToPublicAccumulatedData {
        PrivateToPublicAccumulatedData {
            note_hashes: self.note_hashes.storage(),
            nullifiers: self.nullifiers.storage(),
            l2_to_l1_msgs: self.l2_to_l1_msgs.storage(),
            private_logs: self.private_logs.storage(),
            contract_class_logs_hashes: self.contract_class_logs_hashes.storage(),
            public_call_requests: self.public_call_requests.storage(),
        }
    }
}

impl Empty for PrivateToPublicAccumulatedDataBuilder {
    fn empty() -> Self {
        PrivateToPublicAccumulatedDataBuilder {
            note_hashes: BoundedVec::new(),
            nullifiers: BoundedVec::new(),
            l2_to_l1_msgs: BoundedVec::new(),
            private_logs: BoundedVec::new(),
            contract_class_logs_hashes: BoundedVec::new(),
            public_call_requests: BoundedVec::new(),
        }
    }
}
