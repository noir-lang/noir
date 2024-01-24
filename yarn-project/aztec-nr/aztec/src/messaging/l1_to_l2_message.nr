use dep::protocol_types::{
    address::{
        AztecAddress,
        EthAddress,
    },
    constants::{
        L1_TO_L2_MESSAGE_LENGTH,
        GENERATOR_INDEX__NULLIFIER,
        GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET,
    },
    hash::{
        pedersen_hash,
        sha256_to_field,
    },
};

struct L1ToL2Message {
    sender: EthAddress,
    chainId: Field,
    recipient: AztecAddress,
    version: Field,
    content: Field,
    secret: Field,
    secret_hash: Field,
    deadline: u32,
    fee: u64,
    tree_index: Field
}

impl L1ToL2Message {
    pub fn deserialize(
        fields: [Field; L1_TO_L2_MESSAGE_LENGTH],
        secret: Field,
        tree_index: Field
    ) -> L1ToL2Message {
        L1ToL2Message {
            sender: EthAddress::from_field(fields[0]),
            chainId: fields[1],
            recipient: AztecAddress::from_field(fields[2]),
            version: fields[3],
            content: fields[4],
            secret: secret,
            secret_hash: fields[5],
            deadline: fields[6] as u32,
            fee: fields[7] as u64,
            tree_index: tree_index
        }
    }

    pub fn validate_message_secret(self: Self) {
        let recomputed_hash = pedersen_hash([self.secret], GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET);
        assert(self.secret_hash == recomputed_hash, "Invalid message secret");
    }

    fn hash(self: Self) -> Field {
        let mut hash_bytes: [u8; 256] = [0; 256];
        let sender_bytes = self.sender.to_field().to_be_bytes(32);
        let chainId_bytes = self.chainId.to_be_bytes(32);
        let recipient_bytes = self.recipient.to_field().to_be_bytes(32);
        let version_bytes = self.version.to_be_bytes(32);
        let content_bytes = self.content.to_be_bytes(32);
        let secret_hash_bytes = self.secret_hash.to_be_bytes(32);
        let deadline_bytes = (self.deadline as Field).to_be_bytes(32);
        let fee_bytes = (self.fee as Field).to_be_bytes(32);

        for i in 0..32 {
            hash_bytes[i] = sender_bytes[i];
            hash_bytes[i + 32] = chainId_bytes[i];
            hash_bytes[i + 64] = recipient_bytes[i];
            hash_bytes[i + 96] = version_bytes[i];
            hash_bytes[i + 128] = content_bytes[i];
            hash_bytes[i + 160] = secret_hash_bytes[i];
            hash_bytes[i + 192] = deadline_bytes[i];
            hash_bytes[i + 224] = fee_bytes[i];
        }

        let message_hash = sha256_to_field(hash_bytes);
        message_hash        
    }

    // The nullifier of a l1 to l2 message is the hash of the message salted with the secret and tree index
    // docs:start:l1_to_l2_message_compute_nullifier
    pub fn compute_nullifier(self: Self) -> Field {
        let message_hash = self.hash();
        pedersen_hash([message_hash, self.secret, self.tree_index], GENERATOR_INDEX__NULLIFIER)
    }
    // docs:end:l1_to_l2_message_compute_nullifier
}
