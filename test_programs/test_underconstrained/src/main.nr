// Vulnerable Merkle proof verification
// Classic bug: missing direction bit constraint
// Attacker can use non-binary values for direction!

// Simple "hash" for demonstration (not cryptographic)
fn hash_pair(left: Field, right: Field) -> Field {
    left * 31 + right * 17 + 42
}

// VULNERABLE Merkle proof verification
// Bug: direction bits are not constrained to be 0 or 1
// Attacker can use any field value for direction!
fn verify_merkle_proof(
    leaf: Field,
    root: Field,
    siblings: [Field; 3],
    directions: [Field; 3]  // Should be 0 or 1, but not enforced!
) -> bool {
    let mut current = leaf;
    
    for i in 0..3 {
        let sibling = siblings[i];
        let dir = directions[i];
        
        // BUG: dir can be ANY field value, not just 0 or 1
        // This allows non-determinism in proof verification
        // When dir=0: left=current, right=sibling
        // When dir=1: left=sibling, right=current
        // When dir=0.5: ???  <- undefined behavior!
        let left = current * (1 - dir) + sibling * dir;
        let right = current * dir + sibling * (1 - dir);
        
        current = hash_pair(left, right);
    }
    
    current == root
}

fn main(
    leaf: pub Field,
    root: pub Field, 
    siblings: [Field; 3],
    directions: [Field; 3]
) {
    assert(verify_merkle_proof(leaf, root, siblings, directions));
}
