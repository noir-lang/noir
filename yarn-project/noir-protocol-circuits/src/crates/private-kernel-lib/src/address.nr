use crate::utils;

// Aztec address
struct Address{
    inner : Field
}

impl Address{

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
}

struct EthAddress{
    inner : Field
}

impl EthAddress{

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
}

