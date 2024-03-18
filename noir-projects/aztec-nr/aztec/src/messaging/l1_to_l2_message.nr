use dep::protocol_types::{
    address::{AztecAddress, EthAddress},
    constants::{L1_TO_L2_MESSAGE_LENGTH, GENERATOR_INDEX__NULLIFIER, GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET},
    hash::{pedersen_hash, sha256_to_field}
};

struct L1ToL2Message {
    sender: EthAddress,
    chain_id: Field,
    recipient: AztecAddress,
    version: Field,
    content: Field,
    secret: Field,
    secret_hash: Field,
    tree_index: Field
}

impl L1ToL2Message {
    pub fn new(
        sender: EthAddress,
        chain_id: Field,
        recipient: AztecAddress,
        version: Field,
        content: Field,
        secret: Field
    ) -> L1ToL2Message {
        let secret_hash = pedersen_hash([secret], GENERATOR_INDEX__L1_TO_L2_MESSAGE_SECRET);
        Self {
            sender,
            chain_id,
            recipient,
            version,
            content,
            secret,
            secret_hash,
            tree_index: 0
        }
    }

    pub fn deserialize(
        fields: [Field; L1_TO_L2_MESSAGE_LENGTH],
        secret: Field,
        tree_index: Field
    ) -> L1ToL2Message {
        L1ToL2Message {
            sender: EthAddress::from_field(fields[0]),
            chain_id: fields[1],
            recipient: AztecAddress::from_field(fields[2]),
            version: fields[3],
            content: fields[4],
            secret,
            secret_hash: fields[5],
            tree_index
        }
    }

    fn hash(self: Self) -> Field {
        let mut hash_bytes = [0 as u8; 192];
        let sender_bytes = self.sender.to_field().to_be_bytes(32);
        let chain_id_bytes = self.chain_id.to_be_bytes(32);
        let recipient_bytes = self.recipient.to_field().to_be_bytes(32);
        let version_bytes = self.version.to_be_bytes(32);
        let content_bytes = self.content.to_be_bytes(32);
        let secret_hash_bytes = self.secret_hash.to_be_bytes(32);

        for i in 0..32 {
            hash_bytes[i] = sender_bytes[i];
            hash_bytes[i + 32] = chain_id_bytes[i];
            hash_bytes[i + 64] = recipient_bytes[i];
            hash_bytes[i + 96] = version_bytes[i];
            hash_bytes[i + 128] = content_bytes[i];
            hash_bytes[i + 160] = secret_hash_bytes[i];
        }

        let message_hash = sha256_to_field(hash_bytes);
        message_hash
    }

    // The nullifier of a l1 to l2 message is the hash of the message salted with the secret and tree index
    // docs:start:l1_to_l2_message_compute_nullifier
    pub fn compute_nullifier(self: Self) -> Field {
        let message_hash = self.hash();
        pedersen_hash(
            [message_hash, self.secret, self.tree_index],
            GENERATOR_INDEX__NULLIFIER
        )
    }
    // docs:end:l1_to_l2_message_compute_nullifier
}
