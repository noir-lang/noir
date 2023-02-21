use dep::std;

fn main(
    // old_root: Field,
    // old_leaf: Field,
    // old_hash_path: [Field; 3],
    // new_root: pub Field,
    // leaf: Field,
    // index: Field,
    // mimc_input: [Field; 4],

      index: Field, 
    signer: [u8; 64],
    sig: [u8; 64], 
    msg: [u8; 32], 
    hash_path: [u8; 64]
) //-> pub [u8; 32] 
{
    // let old_leaf_exists = std::merkle::check_membership(old_root, old_leaf, index, old_hash_path);
    // constrain old_leaf_exists == 1;
    // constrain old_root == std::merkle::compute_root_from_leaf(old_leaf, index, old_hash_path);
    // let new_leaf_exists = std::merkle::check_membership(new_root, leaf, index, old_hash_path);
    // constrain new_leaf_exists == 1;

    // let h = std::hash::mimc_bn254(mimc_input);
    // constrain h == 18226366069841799622585958305961373004333097209608110160936134895615261821931;
let w = std::to_bits(1,129);
constrain w[127]==0;

     verify_sig(sig, msg, signer);
    let leaf = std::hash::sha256(signer);
    // compute_root_from_leaf(leaf, index, hash_path)
}
    
fn verify_sig(sig: [u8; 64], msg: [u8; 32], signer: [u8; 64]) {
    let mut pub_x: [u8; 32] = [0; 32];
    let mut pub_y: [u8; 32] = [0; 32];
    
    for i in 0..32 {
        pub_x[i] = signer[i];
        pub_y[i] = signer[i + 32];
     };

    constrain(std::ecdsa_secp256k1::verify_signature(
        pub_x,
        pub_y,
        sig,
        msg
    ) == 1);
}

fn compute_root_from_leaf(leaf: [u8; 32], _index: Field, hash_path: [u8]) -> [u8; 32] {
    let n = std::array::len(hash_path) / 32;
    let _index_bits = std::to_bits(_index, n as u32);

    let mut current = [0; 32];
    let mut neighbour = [0; 32];
    for i in 0..32 {
        current[i] = leaf[i];
    }

    for i in 0..n {
        let offset = i * 32;
        for j in 0..32 {
            neighbour[j] = hash_path[j + offset];
        }

        // Dying if _is_right is computed, works if constant. 
        // Fails with "cannot find witness assignment"
        // _is_right = false will compile, _index_bits[i] as bool; will not
        //let _is_right = false; // _index_bits[i] as bool;
        let _is_right = _index_bits[i] as bool;
        let to_hash = concat(current, neighbour, _is_right);
        current = std::hash::sha256(to_hash);
    }
    current
}

fn concat(left: [u8; 32], right: [u8; 32], flip: bool) -> [u8; 64] {
    let mut concat = [0; 64];
    for i in 0..32 {
        let (l, r) = if flip {
            (right[i], left[i])
        } else {
            (left[i], right[i])
        };

        concat[i] = l;
        concat[i + 32] = r;
    }
    concat
}