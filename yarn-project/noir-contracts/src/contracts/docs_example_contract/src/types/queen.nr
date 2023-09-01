use dep::aztec::types::type_serialisation::TypeSerialisationInterface;

// docs:start:state_vars-CustomStruct
struct Queen {
    account: Field,
    points: u8,
}
// docs:end:state_vars-CustomStruct

// docs:start:state_vars-PublicStateCustomStruct
global QUEEN_SERIALISED_LEN: Field = 2;

fn deserialise(fields: [Field; QUEEN_SERIALISED_LEN]) -> Queen {
    Queen {
        account: fields[0],
        points: fields[1] as u8,
    }
}

fn serialise(queen: Queen) -> [Field; QUEEN_SERIALISED_LEN] {
    [queen.account, queen.points as Field]
}

global QueenSerialisationMethods = TypeSerialisationInterface {
    deserialise,
    serialise,
};
// docs:end:state_vars-PublicStateCustomStruct