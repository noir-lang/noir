use victim;
use victim::Vault::set_secret;

fn main() {
    let original_vault = victim::new();
    let original_secret = victim::read(original_vault);
    assert(original_secret == 1);

    let hijacked_vault = set_secret(original_vault, 31337);

    let hijacked_secret = victim::read(hijacked_vault);
    assert(hijacked_secret == 31337);
}
