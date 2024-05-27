use dep::aztec::prelude::AztecAddress;
use dep::aztec::protocol_types::traits::{Deserialize, Serialize};

struct Position {
    collateral: Field,
    static_debt: Field,
    debt: Field,
}

global POSITION_SERIALIZED_LEN: Field = 3;

impl Serialize<POSITION_SERIALIZED_LEN> for Position {
    fn serialize(position: Position) -> [Field; POSITION_SERIALIZED_LEN] {
        [
            position.collateral.to_field(),
            position.static_debt.to_field(),
            position.debt.to_field(),
        ]
    }
}

impl Deserialize<POSITION_SERIALIZED_LEN> for Position {
    fn deserialize(fields: [Field; POSITION_SERIALIZED_LEN]) -> Position {
        Position {
            collateral: fields[0],
            static_debt: fields[1],
            debt: fields[2],
        }
    }
}
