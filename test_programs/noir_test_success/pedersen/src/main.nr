use std::embedded_curve_ops::EmbeddedCurvePoint;
use std::hash::{pedersen_hash_with_separator, pedersen_commitment_with_separator};

// Some test vectors for Pedersen hash and Pedersen Commitment.
// They have been generated using the same functions so the tests are for now useless
// but they will be useful when we switch to Noir implementation.
#[test]
fn assert_pedersen() {
    assert_eq(
        pedersen_hash_with_separator([1], 1), 0x1b3f4b1a83092a13d8d1a59f7acb62aba15e7002f4440f2275edb99ebbc2305f
    );
    assert_eq(
        pedersen_commitment_with_separator([1], 1), EmbeddedCurvePoint {
        x: 0x054aa86a73cb8a34525e5bbed6e43ba1198e860f5f3950268f71df4591bde402,
        y: 0x209dcfbf2cfb57f9f6046f44d71ac6faf87254afc7407c04eb621a6287cac126,
        is_infinite: false
    }
    );

    assert_eq(
        pedersen_hash_with_separator([1, 2], 2), 0x26691c129448e9ace0c66d11f0a16d9014a9e8498ee78f4d69f0083168188255
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2], 2), EmbeddedCurvePoint {
        x: 0x2e2b3b191e49541fe468ec6877721d445dcaffe41728df0a0eafeb15e87b0753,
        y: 0x2ff4482400ad3a6228be17a2af33e2bcdf41be04795f9782bd96efe7e24f8778,
        is_infinite: false
    }
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3], 3), 0x0bc694b7a1f8d10d2d8987d07433f26bd616a2d351bc79a3c540d85b6206dbe4
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3], 3), EmbeddedCurvePoint {
        x: 0x1fee4e8cf8d2f527caa2684236b07c4b1bad7342c01b0f75e9a877a71827dc85,
        y: 0x2f9fedb9a090697ab69bf04c8bc15f7385b3e4b68c849c1536e5ae15ff138fd1,
        is_infinite: false
    }
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4], 4), 0xdae10fb32a8408521803905981a2b300d6a35e40e798743e9322b223a5eddc
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4], 4), EmbeddedCurvePoint {
        x: 0x07ae3e202811e1fca39c2d81eabe6f79183978e6f12be0d3b8eda095b79bdbc9,
        y: 0x0afc6f892593db6fbba60f2da558517e279e0ae04f95758587760ba193145014,
        is_infinite: false
    }
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5], 5), 0xfc375b062c4f4f0150f7100dfb8d9b72a6d28582dd9512390b0497cdad9c22
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5], 5), EmbeddedCurvePoint {
        x: 0x1754b12bd475a6984a1094b5109eeca9838f4f81ac89c5f0a41dbce53189bb29,
        y: 0x2da030e3cfcdc7ddad80eaf2599df6692cae0717d4e9f7bfbee8d073d5d278f7,
        is_infinite: false
    }
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5, 6], 6), 0x1696ed13dc2730062a98ac9d8f9de0661bb98829c7582f699d0273b18c86a572
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5, 6], 6), EmbeddedCurvePoint {
        x: 0x190f6c0e97ad83e1e28da22a98aae156da083c5a4100e929b77e750d3106a697,
        y: 0x1f4b60f34ef91221a0b49756fa0705da93311a61af73d37a0c458877706616fb,
        is_infinite: false
    }
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5, 6, 7], 7), 0x128c0ff144fc66b6cb60eeac8a38e23da52992fc427b92397a7dffd71c45ede3
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5, 6, 7], 7), EmbeddedCurvePoint {
        x: 0x015441e9d29491b06563fac16fc76abf7a9534c715421d0de85d20dbe2965939,
        y: 0x1d2575b0276f4e9087e6e07c2cb75aa1baafad127af4be5918ef8a2ef2fea8fc,
        is_infinite: false
    }
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5, 6, 7, 8], 8), 0x2f960e117482044dfc99d12fece2ef6862fba9242be4846c7c9a3e854325a55c
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5, 6, 7, 8], 8), EmbeddedCurvePoint {
        x: 0x1657737676968887fceb6dd516382ea13b3a2c557f509811cd86d5d1199bc443,
        y: 0x1f39f0cb569040105fa1e2f156521e8b8e08261e635a2b210bdc94e8d6d65f77,
        is_infinite: false
    }
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5, 6, 7, 8, 9], 9), 0x0c96db0790602dcb166cc4699e2d306c479a76926b81c2cb2aaa92d249ec7be7
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5, 6, 7, 8, 9], 9), EmbeddedCurvePoint {
        x: 0x0a3ceae42d14914a432aa60ec7fded4af7dad7dd4acdbf2908452675ec67e06d,
        y: 0xfc19761eaaf621ad4aec9a8b2e84a4eceffdba78f60f8b9391b0bd9345a2f2,
        is_infinite: false
    }
    );
    assert_eq(
        pedersen_hash_with_separator([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 10), 0x2cd37505871bc460a62ea1e63c7fe51149df5d0801302cf1cbc48beb8dff7e94
    );
    assert_eq(
        pedersen_commitment_with_separator([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 10), EmbeddedCurvePoint {
        x: 0x2fb3f8b3d41ddde007c8c3c62550f9a9380ee546fcc639ffbb3fd30c8d8de30c,
        y: 0x300783be23c446b11a4c0fabf6c91af148937cea15fcf5fb054abf7f752ee245,
        is_infinite: false
    }
    );
}

