use dep::aztec::types::type_serialization::TypeSerializationInterface;

// docs:start:state_vars-CustomStruct
struct Queen {
    account: Field,
    points: u8,
}
// docs:end:state_vars-CustomStruct

// docs:start:state_vars-PublicStateCustomStruct
global QUEEN_SERIALIZED_LEN: Field = 2;

fn deserialize(fields: [Field; QUEEN_SERIALIZED_LEN]) -> Queen {
    Queen {
        account: fields[0],
        points: fields[1] as u8,
    }
}

fn serialize(queen: Queen) -> [Field; QUEEN_SERIALIZED_LEN] {
    [queen.account, queen.points as Field]
}

global QueenSerializationMethods = TypeSerializationInterface {
    deserialize,
    serialize,
};
// docs:end:state_vars-PublicStateCustomStruct