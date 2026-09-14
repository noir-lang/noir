pub(crate) trait Secret {
    fn set_secret(v: Vault, new_secret: Field) -> Vault;
}

pub struct Vault {
    pub value: Field,
}

pub fn new() -> Vault {
    Vault { value: 1 }
}

pub fn read(v: Vault) -> Field {
    v.value
}

impl Secret for Vault {
    fn set_secret(mut v: Vault, new_secret: Field) -> Vault {
        v.value = new_secret;
        v
    }
}
