use dep::aztec::protocol_types::traits::{FromField, ToField, Serialize, Deserialize};

global ADMIN_FLAG: u64 = 1;
global MINTER_FLAG: u64 = 2;
global BLACKLIST_FLAG: u64 = 4;

struct UserFlags {
    is_admin: bool,
    is_minter: bool,
    is_blacklisted: bool,
}

impl FromField for UserFlags {
    fn from_field(value: Field) -> UserFlags {
        let value: u64 = value as u64;
        let is_admin = value & ADMIN_FLAG == ADMIN_FLAG;
        let is_minter = value & MINTER_FLAG == MINTER_FLAG;
        let is_blacklisted = value & BLACKLIST_FLAG == BLACKLIST_FLAG;

        Self { is_admin, is_minter, is_blacklisted }
    }
}

impl ToField for UserFlags {
    fn to_field(self) -> Field {
        let mut value: u64 = 0;

        if self.is_admin {
            value = value | ADMIN_FLAG;
        }

        if self.is_minter {
            value = value | MINTER_FLAG;
        }

        if self.is_blacklisted {
            value = value | BLACKLIST_FLAG;
        }

        value.to_field()
    }
}

// We implement this as it is used when serializing the state variable into return values
// This is very inefficient if used to store the state variable.
// We are currently "abusing" that the `to_field` is called in the `scheduled_value_change`
// where we are using this value.
impl Serialize<3> for UserFlags {
    fn serialize(self) -> [Field; 3] {
        [self.is_admin.to_field(), self.is_minter.to_field(), self.is_blacklisted.to_field()]
        // [self.to_field()]
    }
}

// We implement this as it is required for other contracts to be able to "rebuild" the variable
// after having received the serialized value as a return-value
impl Deserialize<3> for UserFlags {
    fn deserialize(fields: [Field; 3]) -> Self {
        Self {
            is_admin: fields[0] as bool,
            is_minter: fields[1] as bool,
            is_blacklisted: fields[2] as bool,
        }
    }
}

mod test {
    use crate::types::roles::UserFlags;

    fn assert_to_from_field(is_minter: bool, is_admin: bool, is_blacklisted: bool) {
        let flags = UserFlags { is_minter, is_admin, is_blacklisted };
        let converted = UserFlags::from_field(flags.to_field());

        assert_eq(converted.is_minter, is_minter);
        assert_eq(converted.is_admin, is_admin);
        assert_eq(converted.is_blacklisted, is_blacklisted);
    }

    #[test]
    fn test_to_from_field() {
        assert_to_from_field(false, false, false);
        assert_to_from_field(false, false, true);

        assert_to_from_field(false, true, false);
        assert_to_from_field(false, true, true);

        assert_to_from_field(true, false, false);
        assert_to_from_field(true, false, true);

        assert_to_from_field(true, true, false);
        assert_to_from_field(true, true, true);
    }
}
