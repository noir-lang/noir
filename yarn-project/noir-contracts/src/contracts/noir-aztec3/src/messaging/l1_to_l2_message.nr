
use crate::types::point::Point;
use crate::messaging::l1_to_l2_message_getter_data::L1ToL2MessageGetterData;
use crate::messaging::l1_to_l2_message_getter_data::make_l1_to_l2_message_getter_data;


global L1_TO_L2_MESSAGE_LEN = 8; 

global L1_TO_L2_MESSAGE_ORACLE_CALL_LENGTH = 18; // 8 fields + 8 deep hash path + 1 root + 1 index

struct L1ToL2Message {
    sender: Field,
    chainId: Field,
    recipient: Field,
    version: Field,
    content: Field,
    secret: Field,
    secret_hash: Field,
    deadline: u32,
    fee: u64,
    tree_index: Field
}

impl L1ToL2Message {
    fn dummy() -> L1ToL2Message {
        L1ToL2Message {
            sender: 0,
            chainId: 0,
            recipient: 0,
            version: 0,
            content: 0,
            secret: 0,
            secret_hash: 0,
            deadline: 0 as u32,
            fee: 0 as u64,
            tree_index: 0
        }
    }

    fn deserialize(
        fields: [Field; L1_TO_L2_MESSAGE_LEN],
        secret: Field,
        tree_index: Field
    ) -> L1ToL2Message {

        L1ToL2Message {
            sender: fields[0],
            chainId: fields[1],
            recipient: fields[2],
            version: fields[3],
            content: fields[4],
            secret: secret,
            secret_hash: fields[5],
            deadline: fields[6] as u32,
            fee: fields[7] as u64,
            tree_index: tree_index
        }
    }

    fn validate_message_secret(self: Self) {
        let recomputed_hash = dep::std::hash::pedersen([crate::MessageSecret, self.secret])[0];
        assert(self.secret_hash == recomputed_hash);
    }

    fn message_hash(self: Self) -> Field {

        let mut hash_bytes: [u8; 256] = [0; 256];
        let sender_bytes = self.sender.to_be_bytes(32);
        let chainId_bytes = self.chainId.to_be_bytes(32);
        let recipient_bytes = self.recipient.to_be_bytes(32);
        let version_bytes = self.version.to_be_bytes(32);
        let content_bytes = self.content.to_be_bytes(32);
        let secret_hash_bytes = self.secret_hash.to_be_bytes(32);
        let deadline_bytes = (self.deadline as Field).to_be_bytes(32);
        let fee_bytes = (self.fee as Field).to_be_bytes(32);

        // Once loops are performant this will be moved into l1 message
        // Unroll loops because otherwise takes forever to compile
        // for i in range(32):
        //     print('hash_bytes[{0}] = amount_bytes[{1}];'.format(i, i))
        // for i in range(32):
        //     print('hash_bytes[{0} + 32] = recipient_bytes[{1}];'.format(i, i))
        hash_bytes[0] = sender_bytes[0];
        hash_bytes[1] = sender_bytes[1];
        hash_bytes[2] = sender_bytes[2];
        hash_bytes[3] = sender_bytes[3];
        hash_bytes[4] = sender_bytes[4];
        hash_bytes[5] = sender_bytes[5];
        hash_bytes[6] = sender_bytes[6];
        hash_bytes[7] = sender_bytes[7];
        hash_bytes[8] = sender_bytes[8];
        hash_bytes[9] = sender_bytes[9];
        hash_bytes[10] = sender_bytes[10];
        hash_bytes[11] = sender_bytes[11];
        hash_bytes[12] = sender_bytes[12];
        hash_bytes[13] = sender_bytes[13];
        hash_bytes[14] = sender_bytes[14];
        hash_bytes[15] = sender_bytes[15];
        hash_bytes[16] = sender_bytes[16];
        hash_bytes[17] = sender_bytes[17];
        hash_bytes[18] = sender_bytes[18];
        hash_bytes[19] = sender_bytes[19];
        hash_bytes[20] = sender_bytes[20];
        hash_bytes[21] = sender_bytes[21];
        hash_bytes[22] = sender_bytes[22];
        hash_bytes[23] = sender_bytes[23];
        hash_bytes[24] = sender_bytes[24];
        hash_bytes[25] = sender_bytes[25];
        hash_bytes[26] = sender_bytes[26];
        hash_bytes[27] = sender_bytes[27];
        hash_bytes[28] = sender_bytes[28];
        hash_bytes[29] = sender_bytes[29];
        hash_bytes[30] = sender_bytes[30];
        hash_bytes[31] = sender_bytes[31];

        hash_bytes[0 + 32] = chainId_bytes[0];
        hash_bytes[1 + 32] = chainId_bytes[1];
        hash_bytes[2 + 32] = chainId_bytes[2];
        hash_bytes[3 + 32] = chainId_bytes[3];
        hash_bytes[4 + 32] = chainId_bytes[4];
        hash_bytes[5 + 32] = chainId_bytes[5];
        hash_bytes[6 + 32] = chainId_bytes[6];
        hash_bytes[7 + 32] = chainId_bytes[7];
        hash_bytes[8 + 32] = chainId_bytes[8];
        hash_bytes[9 + 32] = chainId_bytes[9];
        hash_bytes[10 + 32] = chainId_bytes[10];
        hash_bytes[11 + 32] = chainId_bytes[11];
        hash_bytes[12 + 32] = chainId_bytes[12];
        hash_bytes[13 + 32] = chainId_bytes[13];
        hash_bytes[14 + 32] = chainId_bytes[14];
        hash_bytes[15 + 32] = chainId_bytes[15];
        hash_bytes[16 + 32] = chainId_bytes[16];
        hash_bytes[17 + 32] = chainId_bytes[17];
        hash_bytes[18 + 32] = chainId_bytes[18];
        hash_bytes[19 + 32] = chainId_bytes[19];
        hash_bytes[20 + 32] = chainId_bytes[20];
        hash_bytes[21 + 32] = chainId_bytes[21];
        hash_bytes[22 + 32] = chainId_bytes[22];
        hash_bytes[23 + 32] = chainId_bytes[23];
        hash_bytes[24 + 32] = chainId_bytes[24];
        hash_bytes[25 + 32] = chainId_bytes[25];
        hash_bytes[26 + 32] = chainId_bytes[26];
        hash_bytes[27 + 32] = chainId_bytes[27];
        hash_bytes[28 + 32] = chainId_bytes[28];
        hash_bytes[29 + 32] = chainId_bytes[29];
        hash_bytes[30 + 32] = chainId_bytes[30];
        hash_bytes[31 + 32] = chainId_bytes[31];

        hash_bytes[0 + 64] = recipient_bytes[0];
        hash_bytes[1 + 64] = recipient_bytes[1];
        hash_bytes[2 + 64] = recipient_bytes[2];
        hash_bytes[3 + 64] = recipient_bytes[3];
        hash_bytes[4 + 64] = recipient_bytes[4];
        hash_bytes[5 + 64] = recipient_bytes[5];
        hash_bytes[6 + 64] = recipient_bytes[6];
        hash_bytes[7 + 64] = recipient_bytes[7];
        hash_bytes[8 + 64] = recipient_bytes[8];
        hash_bytes[9 + 64] = recipient_bytes[9];
        hash_bytes[10 + 64] = recipient_bytes[10];
        hash_bytes[11 + 64] = recipient_bytes[11];
        hash_bytes[12 + 64] = recipient_bytes[12];
        hash_bytes[13 + 64] = recipient_bytes[13];
        hash_bytes[14 + 64] = recipient_bytes[14];
        hash_bytes[15 + 64] = recipient_bytes[15];
        hash_bytes[16 + 64] = recipient_bytes[16];
        hash_bytes[17 + 64] = recipient_bytes[17];
        hash_bytes[18 + 64] = recipient_bytes[18];
        hash_bytes[19 + 64] = recipient_bytes[19];
        hash_bytes[20 + 64] = recipient_bytes[20];
        hash_bytes[21 + 64] = recipient_bytes[21];
        hash_bytes[22 + 64] = recipient_bytes[22];
        hash_bytes[23 + 64] = recipient_bytes[23];
        hash_bytes[24 + 64] = recipient_bytes[24];
        hash_bytes[25 + 64] = recipient_bytes[25];
        hash_bytes[26 + 64] = recipient_bytes[26];
        hash_bytes[27 + 64] = recipient_bytes[27];
        hash_bytes[28 + 64] = recipient_bytes[28];
        hash_bytes[29 + 64] = recipient_bytes[29];
        hash_bytes[30 + 64] = recipient_bytes[30];
        hash_bytes[31 + 64] = recipient_bytes[31];

        hash_bytes[0 + 96] = version_bytes[0];
        hash_bytes[1 + 96] = version_bytes[1];
        hash_bytes[2 + 96] = version_bytes[2];
        hash_bytes[3 + 96] = version_bytes[3];
        hash_bytes[4 + 96] = version_bytes[4];
        hash_bytes[5 + 96] = version_bytes[5];
        hash_bytes[6 + 96] = version_bytes[6];
        hash_bytes[7 + 96] = version_bytes[7];
        hash_bytes[8 + 96] = version_bytes[8];
        hash_bytes[9 + 96] = version_bytes[9];
        hash_bytes[10 + 96] = version_bytes[10];
        hash_bytes[11 + 96] = version_bytes[11];
        hash_bytes[12 + 96] = version_bytes[12];
        hash_bytes[13 + 96] = version_bytes[13];
        hash_bytes[14 + 96] = version_bytes[14];
        hash_bytes[15 + 96] = version_bytes[15];
        hash_bytes[16 + 96] = version_bytes[16];
        hash_bytes[17 + 96] = version_bytes[17];
        hash_bytes[18 + 96] = version_bytes[18];
        hash_bytes[19 + 96] = version_bytes[19];
        hash_bytes[20 + 96] = version_bytes[20];
        hash_bytes[21 + 96] = version_bytes[21];
        hash_bytes[22 + 96] = version_bytes[22];
        hash_bytes[23 + 96] = version_bytes[23];
        hash_bytes[24 + 96] = version_bytes[24];
        hash_bytes[25 + 96] = version_bytes[25];
        hash_bytes[26 + 96] = version_bytes[26];
        hash_bytes[27 + 96] = version_bytes[27];
        hash_bytes[28 + 96] = version_bytes[28];
        hash_bytes[29 + 96] = version_bytes[29];
        hash_bytes[30 + 96] = version_bytes[30];
        hash_bytes[31 + 96] = version_bytes[31];

        hash_bytes[0 + 128] = content_bytes[0];
        hash_bytes[1 + 128] = content_bytes[1];
        hash_bytes[2 + 128] = content_bytes[2];
        hash_bytes[3 + 128] = content_bytes[3];
        hash_bytes[4 + 128] = content_bytes[4];
        hash_bytes[5 + 128] = content_bytes[5];
        hash_bytes[6 + 128] = content_bytes[6];
        hash_bytes[7 + 128] = content_bytes[7];
        hash_bytes[8 + 128] = content_bytes[8];
        hash_bytes[9 + 128] = content_bytes[9];
        hash_bytes[10 + 128] = content_bytes[10];
        hash_bytes[11 + 128] = content_bytes[11];
        hash_bytes[12 + 128] = content_bytes[12];
        hash_bytes[13 + 128] = content_bytes[13];
        hash_bytes[14 + 128] = content_bytes[14];
        hash_bytes[15 + 128] = content_bytes[15];
        hash_bytes[16 + 128] = content_bytes[16];
        hash_bytes[17 + 128] = content_bytes[17];
        hash_bytes[18 + 128] = content_bytes[18];
        hash_bytes[19 + 128] = content_bytes[19];
        hash_bytes[20 + 128] = content_bytes[20];
        hash_bytes[21 + 128] = content_bytes[21];
        hash_bytes[22 + 128] = content_bytes[22];
        hash_bytes[23 + 128] = content_bytes[23];
        hash_bytes[24 + 128] = content_bytes[24];
        hash_bytes[25 + 128] = content_bytes[25];
        hash_bytes[26 + 128] = content_bytes[26];
        hash_bytes[27 + 128] = content_bytes[27];
        hash_bytes[28 + 128] = content_bytes[28];
        hash_bytes[29 + 128] = content_bytes[29];
        hash_bytes[30 + 128] = content_bytes[30];
        hash_bytes[31 + 128] = content_bytes[31];

        hash_bytes[0 + 160] = secret_hash_bytes[0];
        hash_bytes[1 + 160] = secret_hash_bytes[1];
        hash_bytes[2 + 160] = secret_hash_bytes[2];
        hash_bytes[3 + 160] = secret_hash_bytes[3];
        hash_bytes[4 + 160] = secret_hash_bytes[4];
        hash_bytes[5 + 160] = secret_hash_bytes[5];
        hash_bytes[6 + 160] = secret_hash_bytes[6];
        hash_bytes[7 + 160] = secret_hash_bytes[7];
        hash_bytes[8 + 160] = secret_hash_bytes[8];
        hash_bytes[9 + 160] = secret_hash_bytes[9];
        hash_bytes[10 + 160] = secret_hash_bytes[10];
        hash_bytes[11 + 160] = secret_hash_bytes[11];
        hash_bytes[12 + 160] = secret_hash_bytes[12];
        hash_bytes[13 + 160] = secret_hash_bytes[13];
        hash_bytes[14 + 160] = secret_hash_bytes[14];
        hash_bytes[15 + 160] = secret_hash_bytes[15];
        hash_bytes[16 + 160] = secret_hash_bytes[16];
        hash_bytes[17 + 160] = secret_hash_bytes[17];
        hash_bytes[18 + 160] = secret_hash_bytes[18];
        hash_bytes[19 + 160] = secret_hash_bytes[19];
        hash_bytes[20 + 160] = secret_hash_bytes[20];
        hash_bytes[21 + 160] = secret_hash_bytes[21];
        hash_bytes[22 + 160] = secret_hash_bytes[22];
        hash_bytes[23 + 160] = secret_hash_bytes[23];
        hash_bytes[24 + 160] = secret_hash_bytes[24];
        hash_bytes[25 + 160] = secret_hash_bytes[25];
        hash_bytes[26 + 160] = secret_hash_bytes[26];
        hash_bytes[27 + 160] = secret_hash_bytes[27];
        hash_bytes[28 + 160] = secret_hash_bytes[28];
        hash_bytes[29 + 160] = secret_hash_bytes[29];
        hash_bytes[30 + 160] = secret_hash_bytes[30];
        hash_bytes[31 + 160] = secret_hash_bytes[31];

        hash_bytes[0 + 192] = deadline_bytes[0];
        hash_bytes[1 + 192] = deadline_bytes[1];
        hash_bytes[2 + 192] = deadline_bytes[2];
        hash_bytes[3 + 192] = deadline_bytes[3];
        hash_bytes[4 + 192] = deadline_bytes[4];
        hash_bytes[5 + 192] = deadline_bytes[5];
        hash_bytes[6 + 192] = deadline_bytes[6];
        hash_bytes[7 + 192] = deadline_bytes[7];
        hash_bytes[8 + 192] = deadline_bytes[8];
        hash_bytes[9 + 192] = deadline_bytes[9];
        hash_bytes[10 + 192] = deadline_bytes[10];
        hash_bytes[11 + 192] = deadline_bytes[11];
        hash_bytes[12 + 192] = deadline_bytes[12];
        hash_bytes[13 + 192] = deadline_bytes[13];
        hash_bytes[14 + 192] = deadline_bytes[14];
        hash_bytes[15 + 192] = deadline_bytes[15];
        hash_bytes[16 + 192] = deadline_bytes[16];
        hash_bytes[17 + 192] = deadline_bytes[17];
        hash_bytes[18 + 192] = deadline_bytes[18];
        hash_bytes[19 + 192] = deadline_bytes[19];
        hash_bytes[20 + 192] = deadline_bytes[20];
        hash_bytes[21 + 192] = deadline_bytes[21];
        hash_bytes[22 + 192] = deadline_bytes[22];
        hash_bytes[23 + 192] = deadline_bytes[23];
        hash_bytes[24 + 192] = deadline_bytes[24];
        hash_bytes[25 + 192] = deadline_bytes[25];
        hash_bytes[26 + 192] = deadline_bytes[26];
        hash_bytes[27 + 192] = deadline_bytes[27];
        hash_bytes[28 + 192] = deadline_bytes[28];
        hash_bytes[29 + 192] = deadline_bytes[29];
        hash_bytes[30 + 192] = deadline_bytes[30];
        hash_bytes[31 + 192] = deadline_bytes[31];

        hash_bytes[0 + 224] = fee_bytes[0];
        hash_bytes[1 + 224] = fee_bytes[1];
        hash_bytes[2 + 224] = fee_bytes[2];
        hash_bytes[3 + 224] = fee_bytes[3];
        hash_bytes[4 + 224] = fee_bytes[4];
        hash_bytes[5 + 224] = fee_bytes[5];
        hash_bytes[6 + 224] = fee_bytes[6];
        hash_bytes[7 + 224] = fee_bytes[7];
        hash_bytes[8 + 224] = fee_bytes[8];
        hash_bytes[9 + 224] = fee_bytes[9];
        hash_bytes[10 + 224] = fee_bytes[10];
        hash_bytes[11 + 224] = fee_bytes[11];
        hash_bytes[12 + 224] = fee_bytes[12];
        hash_bytes[13 + 224] = fee_bytes[13];
        hash_bytes[14 + 224] = fee_bytes[14];
        hash_bytes[15 + 224] = fee_bytes[15];
        hash_bytes[16 + 224] = fee_bytes[16];
        hash_bytes[17 + 224] = fee_bytes[17];
        hash_bytes[18 + 224] = fee_bytes[18];
        hash_bytes[19 + 224] = fee_bytes[19];
        hash_bytes[20 + 224] = fee_bytes[20];
        hash_bytes[21 + 224] = fee_bytes[21];
        hash_bytes[22 + 224] = fee_bytes[22];
        hash_bytes[23 + 224] = fee_bytes[23];
        hash_bytes[24 + 224] = fee_bytes[24];
        hash_bytes[25 + 224] = fee_bytes[25];
        hash_bytes[26 + 224] = fee_bytes[26];
        hash_bytes[27 + 224] = fee_bytes[27];
        hash_bytes[28 + 224] = fee_bytes[28];
        hash_bytes[29 + 224] = fee_bytes[29];
        hash_bytes[30 + 224] = fee_bytes[30];
        hash_bytes[31 + 224] = fee_bytes[31];

        let message_sha256 = dep::std::hash::sha256(hash_bytes);

        // // Convert the message_sha256 to a field element
        let mut v = 1;
        let mut high = 0 as Field;
        let mut low = 0 as Field;

        // Unroll loops because otherwise takes forever to compile
        // for i in [15 - i for i in range(16)]:
        //     print('high = high + (message_sha256[{0}] as Field) * v;'.format(i))
        //     print('low = low + (message_sha256[16 + {0}] as Field) * v;'.format(i))
        //     print('v = v * 256;');
        high = high + (message_sha256[15] as Field) * v;
        low = low + (message_sha256[16 + 15] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[14] as Field) * v;
        low = low + (message_sha256[16 + 14] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[13] as Field) * v;
        low = low + (message_sha256[16 + 13] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[12] as Field) * v;
        low = low + (message_sha256[16 + 12] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[11] as Field) * v;
        low = low + (message_sha256[16 + 11] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[10] as Field) * v;
        low = low + (message_sha256[16 + 10] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[9] as Field) * v;
        low = low + (message_sha256[16 + 9] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[8] as Field) * v;
        low = low + (message_sha256[16 + 8] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[7] as Field) * v;
        low = low + (message_sha256[16 + 7] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[6] as Field) * v;
        low = low + (message_sha256[16 + 6] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[5] as Field) * v;
        low = low + (message_sha256[16 + 5] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[4] as Field) * v;
        low = low + (message_sha256[16 + 4] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[3] as Field) * v;
        low = low + (message_sha256[16 + 3] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[2] as Field) * v;
        low = low + (message_sha256[16 + 2] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[1] as Field) * v;
        low = low + (message_sha256[16 + 1] as Field) * v;
        v = v * 256;
        high = high + (message_sha256[0] as Field) * v;
        low = low + (message_sha256[16 + 0] as Field) * v;
        v = v * 256;

        let message_hash = low + high * v;
        message_hash        
    }

    // The nullifier of a l1 to l2 message is the hash of the message salted with the secret and tree index
    fn compute_nullifier(self: Self) -> Field {
        let message_hash = self.message_hash();
        dep::std::hash::pedersen([crate::Nullifier, message_hash, self.secret, self.tree_index])[0]
    }
}
