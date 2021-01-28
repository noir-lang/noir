#[macro_export]
macro_rules! TYPES_LIBRARY {
    () => { r#"

    /**
     * @title PairingsBn254 library used for the fr, g1 and g2 point types
     * @dev Used to manipulate fr, g1, g2 types, perform modular arithmetic on them and call
     * the precompiles add, scalar mul and pairing
     *
     * Notes on optimisations
     * 1) Perform addmod, mulmod etc. in assembly - removes the check that Solidity performs to confirm that
     * the supplied modulus is not 0. This is safe as the modulus's used (r_mod, q_mod) are hard coded
     * inside the contract and not supplied by the user
     */
    library Types {
        uint256 constant STATE_WIDTH = 4;
        uint256 constant NUM_NU_CHALLENGES = 11;
        uint256 constant PRIM_ROOT_SIZE = 28;
        uint256 constant NUM_KATE_OPENING_ELEMENTS = 28; // TODO check this, could be smaller
        uint256 constant PRIM_ROOT = 0x2a3c09f0a58a7e8500e0a7eb8ef62abc402d111e41112ed49bd61b6e725b19f0;
        uint256 constant r_mod = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    
        uint256 constant coset_generator0 = 0x0000000000000000000000000000000000000000000000000000000000000005;
        uint256 constant coset_generator1 = 0x0000000000000000000000000000000000000000000000000000000000000006;
        uint256 constant coset_generator2 = 0x0000000000000000000000000000000000000000000000000000000000000007;
    
        // TODO: add external_coset_generator() method to compute this
        uint256 constant coset_generator7 = 0x000000000000000000000000000000000000000000000000000000000000000c;
    
        struct G1Point {
            uint256 X;
            uint256 Y;
        }
    
        struct Fr {
            uint256 value;
        }
    
        // Encoding of field elements is: X[0] * z + X[1]
        struct G2Point {
            uint256[2] X;
            uint256[2] Y;
        }
    
        struct Proof {
            uint256[] public_input_values;
            G1Point[STATE_WIDTH] wire_commitments;
            G1Point grand_product_commitment;
            G1Point permutation_commitment;
            G1Point[STATE_WIDTH] quotient_poly_commitments;
            Fr[STATE_WIDTH] wire_values_at_z;
            Fr[STATE_WIDTH] wire_values_at_z_omega;
            Fr q_arith_at_z;
            Fr q_ecc_at_z;
            Fr q_c_at_z;
            Fr grand_product_at_z_omega;
            Fr quotient_polynomial_at_z;
            Fr linearization_polynomial_at_z;
            Fr[STATE_WIDTH - 1] permutation_polynomials_at_z;
            Fr wzBar;
            G1Point opening_at_z_proof;
            G1Point opening_at_z_omega_proof;
            G1Point[28] kate_group_elements;
            Fr[NUM_KATE_OPENING_ELEMENTS] kate_field_elements;
            uint256 kate_array_indexer;
            Fr debug_challenge;
            Fr[10] debug_markers;
        }
    
        struct PartialVerifierState {
            Fr alpha;
            Fr beta;
            Fr gamma;
            Fr[NUM_NU_CHALLENGES] v;
            Fr u;
            Fr zeta;
            Fr[] cached_lagrange_evals;
        }
    
        struct ChallengeTranscript {
            bytes32 debug_data;
            Fr init;
            Fr alpha;
            Fr beta;
            Fr gamma;
            Fr zeta;
            Fr[NUM_NU_CHALLENGES] v;
            Fr u;
            Fr alpha_base;
        }
    
        struct VerificationKey {
            uint256 domain_size;
            uint256 circuit_size;
            uint256 num_inputs;
            Fr domain_inverse;
            Fr work_root;
            Fr work_root_inverse;
            Fr omega;
            G1Point Q1;
            G1Point Q2;
            G1Point Q3;
            G1Point Q4;
            G1Point Q5;
            G1Point QM;
            G1Point QC;
            G1Point QARITH;
            G1Point QECC;
            G1Point QRANGE;
            G1Point QLOGIC;
            G1Point[STATE_WIDTH] sigma_commitments;
            Fr[STATE_WIDTH - 1] permutation_non_residues;
            G2Point g2_x;
        }
    
        struct BatchInversions {
            Fr public_input_delta_denominator_inverse;
            Fr zero_poly_inverse;
            Fr lagrange_1_fraction_inverse;
            Fr lagrange_n_fraction_inverse;
        }
    
        struct Fraction {
            Fr numerator;
            Fr denominator;
        }
    
        struct PartialStateFractions {
            Fraction public_input_delta;
            Fraction zero_poly;
            Fraction lagrange_1_fraction;
            Fraction lagrange_n_fraction;
        }
    }
    
    "# };
}

// pub const TYPES_LIBRARY : &str = r#"

// /**
//  * @title PairingsBn254 library used for the fr, g1 and g2 point types
//  * @dev Used to manipulate fr, g1, g2 types, perform modular arithmetic on them and call
//  * the precompiles add, scalar mul and pairing
//  *
//  * Notes on optimisations
//  * 1) Perform addmod, mulmod etc. in assembly - removes the check that Solidity performs to confirm that
//  * the supplied modulus is not 0. This is safe as the modulus's used (r_mod, q_mod) are hard coded
//  * inside the contract and not supplied by the user
//  */
// library Types {
//     uint256 constant STATE_WIDTH = 4;
//     uint256 constant NUM_NU_CHALLENGES = 11;
//     uint256 constant PRIM_ROOT_SIZE = 28;
//     uint256 constant NUM_KATE_OPENING_ELEMENTS = 28; // TODO check this, could be smaller
//     uint256 constant PRIM_ROOT = 0x2a3c09f0a58a7e8500e0a7eb8ef62abc402d111e41112ed49bd61b6e725b19f0;
//     uint256 constant r_mod = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

//     uint256 constant coset_generator0 = 0x0000000000000000000000000000000000000000000000000000000000000005;
//     uint256 constant coset_generator1 = 0x0000000000000000000000000000000000000000000000000000000000000006;
//     uint256 constant coset_generator2 = 0x0000000000000000000000000000000000000000000000000000000000000007;

//     // TODO: add external_coset_generator() method to compute this
//     uint256 constant coset_generator7 = 0x000000000000000000000000000000000000000000000000000000000000000c;

//     struct G1Point {
//         uint256 X;
//         uint256 Y;
//     }

//     struct Fr {
//         uint256 value;
//     }

//     // Encoding of field elements is: X[0] * z + X[1]
//     struct G2Point {
//         uint256[2] X;
//         uint256[2] Y;
//     }

//     struct Proof {
//         uint256[] public_input_values;
//         G1Point[STATE_WIDTH] wire_commitments;
//         G1Point grand_product_commitment;
//         G1Point permutation_commitment;
//         G1Point[STATE_WIDTH] quotient_poly_commitments;
//         Fr[STATE_WIDTH] wire_values_at_z;
//         Fr[STATE_WIDTH] wire_values_at_z_omega;
//         Fr q_arith_at_z;
//         Fr q_ecc_at_z;
//         Fr q_c_at_z;
//         Fr grand_product_at_z_omega;
//         Fr quotient_polynomial_at_z;
//         Fr linearization_polynomial_at_z;
//         Fr[STATE_WIDTH - 1] permutation_polynomials_at_z;
//         Fr wzBar;
//         G1Point opening_at_z_proof;
//         G1Point opening_at_z_omega_proof;
//         G1Point[28] kate_group_elements;
//         Fr[NUM_KATE_OPENING_ELEMENTS] kate_field_elements;
//         uint256 kate_array_indexer;
//         Fr debug_challenge;
//         Fr[10] debug_markers;
//     }

//     struct PartialVerifierState {
//         Fr alpha;
//         Fr beta;
//         Fr gamma;
//         Fr[NUM_NU_CHALLENGES] v;
//         Fr u;
//         Fr zeta;
//         Fr[] cached_lagrange_evals;
//     }

//     struct ChallengeTranscript {
//         bytes32 debug_data;
//         Fr init;
//         Fr alpha;
//         Fr beta;
//         Fr gamma;
//         Fr zeta;
//         Fr[NUM_NU_CHALLENGES] v;
//         Fr u;
//         Fr alpha_base;
//     }

//     struct VerificationKey {
//         uint256 domain_size;
//         uint256 circuit_size;
//         uint256 num_inputs;
//         Fr domain_inverse;
//         Fr work_root;
//         Fr work_root_inverse;
//         Fr omega;
//         G1Point Q1;
//         G1Point Q2;
//         G1Point Q3;
//         G1Point Q4;
//         G1Point Q5;
//         G1Point QM;
//         G1Point QC;
//         G1Point QARITH;
//         G1Point QECC;
//         G1Point QRANGE;
//         G1Point QLOGIC;
//         G1Point[STATE_WIDTH] sigma_commitments;
//         Fr[STATE_WIDTH - 1] permutation_non_residues;
//         G2Point g2_x;
//     }

//     struct BatchInversions {
//         Fr public_input_delta_denominator_inverse;
//         Fr zero_poly_inverse;
//         Fr lagrange_1_fraction_inverse;
//         Fr lagrange_n_fraction_inverse;
//     }

//     struct Fraction {
//         Fr numerator;
//         Fr denominator;
//     }

//     struct PartialStateFractions {
//         Fraction public_input_delta;
//         Fraction zero_poly;
//         Fraction lagrange_1_fraction;
//         Fraction lagrange_n_fraction;
//     }
// }

// "#;