global BLACKLIST_FLAG: U128 = U128::from_integer(1);
global MINTER_FLAG: U128 = U128::from_integer(2);
global ADMIN_FLAG: U128 = U128::from_integer(4);

struct UserFlags {
    is_admin: bool,
    is_minter: bool,
    is_blacklisted: bool,
}

impl UserFlags {

    pub fn new(value: U128) -> Self {
        let is_admin = value & ADMIN_FLAG == ADMIN_FLAG;
        let is_minter = value & MINTER_FLAG == MINTER_FLAG;
        let is_blacklisted = value & BLACKLIST_FLAG == BLACKLIST_FLAG;

        Self { is_admin, is_minter, is_blacklisted }
    }

    pub fn get_value(self) -> U128 {
        let mut value: U128 = U128::from_integer(0);

        if self.is_admin {
            value = value | ADMIN_FLAG;
        }

        if self.is_minter {
            value = value | MINTER_FLAG;
        }

        if self.is_blacklisted {
            value = value | BLACKLIST_FLAG;
        }

        value
    }
}
