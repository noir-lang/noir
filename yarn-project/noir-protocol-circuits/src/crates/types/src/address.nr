use crate::utils;

// Aztec address
struct AztecAddress {
    inner : Field
}

impl AztecAddress {
    pub fn zero() -> Self {
        Self {
            inner: 0
        }
    }

    pub fn default() -> Self {
        Self {
            inner : 0
        }
    }

    pub fn from_field(field : Field) -> Self {
        Self {
            inner : field
        }
    }

    pub fn to_field(self) -> Field {
        self.inner
    }

    pub fn is_zero(self) -> bool {
        self.inner == 0
    }
    
    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }

    pub fn conditional_assign(predicate: bool, lhs : Self, rhs : Self) -> Self{
        let result = utils::conditional_assign(predicate, rhs.to_field(), lhs.to_field());
        Self {
            inner : result
        }
    }

    pub fn eq(self, other : Self) -> bool {
        self.to_field() == other.to_field()
    }

    pub fn serialize(self: Self) -> [Field; 1] {
        [self.inner]
    }

    pub fn deserialize(fields: [Field; 1]) -> Self {
        Self {
            inner: fields[0]
        }
    }
}

struct EthAddress{
    inner : Field
}

impl EthAddress{
    pub fn zero() -> Self {
        Self {
            inner: 0
        }
    }

    pub fn default() -> Self {
        Self {
            inner : 0
        }
    }

    pub fn from_field(field : Field) -> Self {
        Self {
            inner : field
        }
    }

    pub fn to_field(self) -> Field{
        self.inner
    }

    pub fn is_zero(self) -> bool {
        self.inner == 0
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }

    pub fn conditional_assign(predicate: bool, lhs : Self, rhs : Self) -> Self{
        let result = utils::conditional_assign(predicate, rhs.to_field(), lhs.to_field());
        Self {
            inner : result
        }
    }

    pub fn eq(self, other : Self) -> bool {
        self.to_field() == other.to_field()
    }

    pub fn serialize(self: Self) -> [Field; 1] {
        [self.inner]
    }

    pub fn deserialize(fields: [Field; 1]) -> Self {
        Self {
            inner: fields[0]
        }
    }
}

