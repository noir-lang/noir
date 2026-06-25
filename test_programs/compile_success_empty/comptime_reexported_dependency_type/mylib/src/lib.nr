mod hidden {
    pub struct Secret {
        pub value: Field,
    }
}

// `Secret` lives in a private module but is publicly re-exported, so it is part of the
// library's public API and may legitimately appear in a downstream crate's public API.
pub use hidden::Secret;

pub struct PublicBox {
    secret: Secret,
}
