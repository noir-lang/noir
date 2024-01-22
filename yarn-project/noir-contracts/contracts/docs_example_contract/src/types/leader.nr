use dep::aztec::protocol_types::address::AztecAddress;
use dep::aztec::types::type_serialization::TypeSerializationInterface;

// Shows how to create a custom struct in Public
struct Leader {
    account: AztecAddress,
    points: u8,
}

global LEADER_SERIALIZED_LEN: Field = 2;

fn deserialize(fields: [Field; LEADER_SERIALIZED_LEN]) -> Leader {
    Leader { account: AztecAddress::from_field(fields[0]), points: fields[1] as u8 }
}

fn serialize(leader: Leader) -> [Field; LEADER_SERIALIZED_LEN] {
    [leader.account.to_field(), leader.points as Field]
}

global LeaderSerializationMethods = TypeSerializationInterface {
    deserialize,
    serialize,
};
