mod victim {
    pub struct Vault {
        secret: Field,
    }

    pub fn new() -> Vault {
        Vault { secret: 1 }
    }

    fn set_secret(mut v: Vault, new_secret: Field) -> Vault {
        v.secret = new_secret;
        v
    }
}

// Resolving a private function name in a foreign module's scope should fail.
fn main() {
    let _hijacked = comptime {
        let victim_module = quote { victim }.as_module().unwrap();
        let secret_expr = quote { set_secret }.as_expr().unwrap().resolve(Option::some(victim_module.functions()[0]));
        quote { $secret_expr(victim::new(), 31337) }
    };
}
