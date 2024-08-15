use dep::aztec::protocol_types::{address::AztecAddress, traits::{Serialize, Deserialize}};

global SETTINGS_LENGTH = 2;

struct Settings {
    other_asset: AztecAddress,
    admin: AztecAddress,
}

impl Serialize<SETTINGS_LENGTH> for Settings {
    fn serialize(self: Self) -> [Field; SETTINGS_LENGTH] {
        [self.other_asset.to_field(), self.admin.to_field()]
    }
}

impl Deserialize<SETTINGS_LENGTH> for Settings {
    fn deserialize(fields: [Field; SETTINGS_LENGTH]) -> Self {
        Settings {
            other_asset: AztecAddress::from_field(fields[0]),
            admin: AztecAddress::from_field(fields[1]),
        }
    }
}
