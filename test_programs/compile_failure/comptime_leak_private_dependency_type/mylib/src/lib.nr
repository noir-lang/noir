mod hidden {
    pub struct Secret {
        pub value: Field,
    }
}

pub struct PublicBox {
    secret: hidden::Secret,
}

pub fn anchor() -> Field {
    0
}
