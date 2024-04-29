use crate::{
    address::AztecAddress, constants::GENERATOR_INDEX__SIDE_EFFECT,
    traits::{Empty, Hash, Serialize, Deserialize}, constants::SIDE_EFFECT_LENGTH
};
use dep::std::cmp::Eq;

trait Ordered {
    fn counter(self) -> u32;
}

trait OrderedValue<T> where T: Eq {
    fn value(self) -> T;
    fn counter(self) -> u32;
}

trait ContractScopedOrderedValue<T> where T: Eq {
    fn value(self) -> T;
    fn counter(self) -> u32;
    fn contract_address(self) -> AztecAddress;
}

struct SideEffect {
    value: Field,
    counter: u32,
}

impl Ordered for SideEffect {
    fn counter(self) -> u32 {
        self.counter
    }
}

impl OrderedValue<Field> for SideEffect {
    fn value(self) -> Field {
        self.value
    }
    fn counter(self) -> u32 {
        self.counter
    }
}

impl Eq for SideEffect {
    fn eq(self, side_effect: SideEffect) -> bool {
        (self.value == side_effect.value)
        & (self.counter == side_effect.counter)
    }
}

impl Empty for SideEffect {
    fn empty() -> Self {
        SideEffect {
            value: 0,
            counter: 0,
        }
    }
}

impl Hash for SideEffect {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator(
            self.serialize() , GENERATOR_INDEX__SIDE_EFFECT)
    }
}

impl Serialize<SIDE_EFFECT_LENGTH> for SideEffect {
    fn serialize(self) -> [Field; SIDE_EFFECT_LENGTH] {
        [self.value, self.counter as Field]
    }
}

impl Deserialize<SIDE_EFFECT_LENGTH> for SideEffect {
    fn deserialize(values: [Field; SIDE_EFFECT_LENGTH]) -> Self {
        Self {
            value: values[0],
            counter: values[1] as u32,
        }
    }
}
