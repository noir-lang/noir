use crate::utils::field_from_bytes;

global SELECTOR_SIZE = 4;

fn compute_selector<N>(signature: str<N>) -> Field {
    let bytes = signature.as_bytes();
    let hash = dep::std::hash::keccak256(bytes, bytes.len() as u32);
    
    let mut selector_be_bytes = [0; SELECTOR_SIZE];
    for i in 0..SELECTOR_SIZE {
        selector_be_bytes[i] = hash[i];
    }

    field_from_bytes(selector_be_bytes, true)
}