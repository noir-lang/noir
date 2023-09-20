// docs:start:token_util_all
use dep::std::hash::{pedersen_with_separator};
use dep::aztec::constants_gen::GENERATOR_INDEX__SIGNATURE_PAYLOAD;

fn compute_message_hash<N>(args: [Field; N]) -> Field {
  // @todo @lherskind We should probably use a separate generator for this, 
  //                  to avoid any potential collisions with payloads.
  pedersen_with_separator(args, GENERATOR_INDEX__SIGNATURE_PAYLOAD)[0]
}
// docs:end:token_util_all