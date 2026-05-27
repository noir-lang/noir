// `FunctionDefinition::as_typed_expr` must not let a caller mint a callable
// reference to a private function declared in another module.
// `as_typed_expr` builds the `HirIdent` directly from the `FuncId`
// and so must perform the visibility check itself.
mod victim {
    pub struct Vault {
        secret: Field,
    }

    pub fn new() -> Vault {
        Vault { secret: 1 }
    }

    pub fn read(v: Vault) -> Field {
        v.secret
    }

    fn set_secret(mut v: Vault, new_secret: Field) -> Vault {
        v.secret = new_secret;
        v
    }
}

fn main() {
    let original_vault = victim::new();

    let hijacked_vault = comptime {
        let victim_module = quote { victim }.as_module().unwrap();
        let mut found = Option::none();
        for f in victim_module.functions() {
            if f.name() == quote { set_secret } {
                found = Option::some(f);
            }
        }
        let set_secret = found.unwrap();
        let set_secret_expr = set_secret.as_typed_expr();
        quote { $set_secret_expr(original_vault, 31337) }
    };

    assert(victim::read(hijacked_vault) == 31337);
}
