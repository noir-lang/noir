use dep::aztec::state_vars::map::Map;
use dep::aztec::state_vars::public_state::PublicState;
use dep::aztec::state_vars::public_state_value::PublicStateValueInterface;
use dep::aztec::state_vars::public_state_value::field_value::FieldValueInterface;
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


global TOT_SIZE: Field = 2;

// Struct to be used to represent "totals". Generally, there should be one per asset.
// It stores the global values that are shared among all users, such as an accumulator
// and last time it was updated.
// In practice, it should also point to an oracle and have more fields related to 
// loan to value ratios and other things, but we did not have enough reads/writes for this.
struct Tot {
    interest_accumulator: u120,
    last_updated_ts: u120,
}

fn deserialiseTot(fields: [Field; TOT_SIZE]) -> Tot {
    Tot {
        interest_accumulator: fields[0] as u120,
        last_updated_ts: fields[1] as u120,
    }
}

fn serialiseTot(tot: Tot) -> [Field; TOT_SIZE] {
    [tot.interest_accumulator as Field, tot.last_updated_ts as Field]
}

global TotInterface = PublicStateValueInterface {
    deserialise: deserialiseTot,
    serialise: serialiseTot,
};


// Struct to be used to represent positions when we have more reads.
global POS_SIZE: Field = 2;

struct Pos {
    owner: Field,
    value: Field,
}

fn deserialisePos(fields: [Field; POS_SIZE]) -> Pos {
    Pos {
        owner: fields[0],
        value: fields[1],
    }
}

fn serialisePos(pos: Pos) -> [Field; POS_SIZE] {
    [pos.owner, pos.value]
}

global PosInterface = PublicStateValueInterface {
    deserialise: deserialisePos,
    serialise: serialisePos,
};


// Storage structure, containing all storage, and specifying what slots they use.
struct Storage {
    assets: Map<PublicState<Tot, TOT_SIZE>>,
    collateral: Map<PublicState<Field, 1>>,
    static_debt: Map<PublicState<Field, 1>>, // abusing keys very heavily
}

impl Storage {
    fn init() -> Self {
        Storage {
            assets: Map::new(1, |s| PublicState::new(s, TotInterface)), // uses 2 storage slots.
            collateral: Map::new(2, |s| PublicState::new(s, FieldValueInterface)), // uses 1 storage slots.
            static_debt: Map::new(3, |s| PublicState::new(s, FieldValueInterface)), // uses 1 storage slots. 
        }
    }
}