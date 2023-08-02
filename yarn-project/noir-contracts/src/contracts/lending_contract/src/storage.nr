use dep::aztec::state_vars::map::Map;
use dep::aztec::state_vars::public_state::PublicState;
use dep::aztec::state_vars::type_serialisation::TypeSerialisationInterface;
use dep::aztec::state_vars::type_serialisation::field_serialisation::FieldSerialisationMethods;
use dep::aztec::state_vars::type_serialisation::field_serialisation::FIELD_SERIALISED_LEN;
use dep::std::hash::pedersen;

// Utility struct used to easily get a "id" for a private user that sits in the same
// "space" as the public users.
// It help us to have a single mapping for collateral that have both public and private users.
struct Account {
    address: Field,
    secret: Field,
}

impl Account {
    fn new(
        address: Field,
        secret: Field
    ) -> Self {
        Self {
            address,
            secret
        }
    }

    fn key(self: Self) -> Field {
        pedersen([self.address, self.secret])[0]
    }
}



// Struct to be used to represent "totals". Generally, there should be one per asset.
// It stores the global values that are shared among all users, such as an accumulator
// and last time it was updated.
// In practice, it should also point to an oracle and have more fields related to 
// loan to value ratios and other things, but we did not have enough reads/writes for this.
struct Tot {
    interest_accumulator: u120,
    last_updated_ts: u120,
}

global TOT_SERIALISED_LEN: Field = 2;

fn deserialiseTot(fields: [Field; TOT_SERIALISED_LEN]) -> Tot {
    Tot {
        interest_accumulator: fields[0] as u120,
        last_updated_ts: fields[1] as u120,
    }
}

fn serialiseTot(tot: Tot) -> [Field; TOT_SERIALISED_LEN] {
    [tot.interest_accumulator as Field, tot.last_updated_ts as Field]
}

global TotSerialisationMethods = TypeSerialisationInterface {
    deserialise: deserialiseTot,
    serialise: serialiseTot,
};


// // Struct to be used to represent positions when we have more reads.
// global POS_SERIALISED_LEN: Field = 2;

// struct Pos {
//     owner: Field,
//     value: Field,
// }

// fn deserialisePos(fields: [Field; POS_SERIALISED_LEN]) -> Pos {
//     Pos {
//         owner: fields[0],
//         value: fields[1],
//     }
// }

// fn serialisePos(pos: Pos) -> [Field; POS_SERIALISED_LEN] {
//     [pos.owner, pos.value]
// }

// global PosSerialisationMethods = TypeSerialisationInterface {
//     deserialise: deserialisePos,
//     serialise: serialisePos,
// };


// Storage structure, containing all storage, and specifying what slots they use.
struct Storage {
    assets: Map<PublicState<Tot, TOT_SERIALISED_LEN>>,
    collateral: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
    static_debt: Map<PublicState<Field, FIELD_SERIALISED_LEN>>, // abusing keys very heavily
}

impl Storage {
    fn init() -> Self {
        Storage {
            assets: Map::new(1, |slot| PublicState::new(slot, TotSerialisationMethods)),
            collateral: Map::new(2, |slot| PublicState::new(slot, FieldSerialisationMethods)),
            static_debt: Map::new(3, |slot| PublicState::new(slot, FieldSerialisationMethods)),
        }
    }
}