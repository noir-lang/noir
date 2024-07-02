use dep::aztec::{
    prelude::{AztecAddress, NoteHeader, NoteInterface, PrivateContext},
    protocol_types::{constants::GENERATOR_INDEX__NOTE_NULLIFIER, grumpkin_point::GrumpkinPoint, hash::poseidon2_hash},
    note::utils::compute_note_hash_for_consumption, oracle::unsafe_rand::unsafe_rand,
    keys::getters::get_nsk_app, note::note_getter_options::PropertySelector
};
use dep::std::field::bn254::decompose;
use dep::std::embedded_curve_ops::{EmbeddedCurvePoint, EmbeddedCurveScalar, multi_scalar_mul, fixed_base_scalar_mul};

trait OwnedNote {
    fn new(amount: U128, owner_npk_m_hash: Field) -> Self;
    fn get_amount(self) -> U128;
    fn get_owner_npk_m_hash(self) -> Field;
    fn get_owner_selector() -> PropertySelector;
}

trait PrivatelyRefundable {
    fn generate_refund_points(
        fee_payer_npk_m_hash: Field,
        sponsored_user_npk_m_hash: Field,
        funded_amount: Field,
        refund_nonce: Field
    ) -> [EmbeddedCurvePoint; 2];

    fn complete_refund(
        fee_payer_point: EmbeddedCurvePoint,
        sponsored_user_point: EmbeddedCurvePoint,
        transaction_fee: Field
    ) -> [Field; 2];
}

global TOKEN_NOTE_LEN: Field = 3; // 3 plus a header.
global TOKEN_NOTE_BYTES_LEN: Field = 3 * 32 + 64;
// Grumpkin generator point. 
global G1 = EmbeddedCurvePoint { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };

#[aztec(note)]
struct TokenNote {
    // The amount of tokens in the note
    amount: U128,
    // The nullifying public key hash is used with the nsk_app to ensure that the note can be privately spent.
    npk_m_hash: Field,
    // Randomness of the note to hide its contents
    randomness: Field,
}

impl NoteInterface<TOKEN_NOTE_LEN, TOKEN_NOTE_BYTES_LEN> for TokenNote {
    // docs:start:nullifier
    fn compute_note_hash_and_nullifier(self, context: &mut PrivateContext) -> ( Field, Field ) {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = context.request_nsk_app(self.npk_m_hash);
        let nullifier = poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ]);
        (note_hash_for_nullify, nullifier)
    }
    // docs:end:nullifier

    fn compute_note_hash_and_nullifier_without_context(self) -> ( Field, Field ) {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = get_nsk_app(self.npk_m_hash);
        let nullifier = poseidon2_hash([
            note_hash_for_nullify,
            secret,
            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
        ]);
        (note_hash_for_nullify, nullifier)
    }



    fn compute_note_content_hash(self) -> Field {
        let (npk_lo, npk_hi) = decompose(self.npk_m_hash);
        let (random_lo, random_hi) = decompose(self.randomness);
        multi_scalar_mul(
            [G1, G1, G1],
            [EmbeddedCurveScalar {
                lo: self.amount.to_integer(),
                hi: 0
            },
            EmbeddedCurveScalar {
                lo: npk_lo,
                hi: npk_hi
            },
            EmbeddedCurveScalar {
                lo: random_lo,
                hi: random_hi,
            }]
        )[0]
    }
}

impl OwnedNote for TokenNote {
    fn new(amount: U128, owner_npk_m_hash: Field) -> Self {
        Self {
            amount,
            npk_m_hash: owner_npk_m_hash,
            randomness: unsafe_rand(),
            header: NoteHeader::empty(),
        }
    }

    fn get_amount(self) -> U128 {
        self.amount
    }

    fn get_owner_npk_m_hash(self) -> Field {
        self.npk_m_hash
    }

    fn get_owner_selector() -> PropertySelector {
        PropertySelector { index: 1, offset: 0, length: 32 }
    }
}

impl PrivatelyRefundable for TokenNote {
    fn generate_refund_points(fee_payer_npk_m_hash: Field, sponsored_user_npk_m_hash: Field, funded_amount: Field, refund_nonce: Field) -> [EmbeddedCurvePoint; 2] {
        let (refund_nonce_lo, refund_nonce_hi) = decompose(refund_nonce);
        let (fee_payer_lo, fee_payer_hi) = decompose(fee_payer_npk_m_hash);

        let fee_payer_point = multi_scalar_mul(
            [G1, G1],
            [EmbeddedCurveScalar {
                lo: fee_payer_lo,
                hi: fee_payer_hi
            },
            EmbeddedCurveScalar {
                lo: refund_nonce_lo,
                hi: refund_nonce_hi
            }]
        );

        let (sponsored_user_lo, sponsored_user_hi) = decompose(sponsored_user_npk_m_hash);
        let (funded_amount_lo, funded_amount_hi) = decompose(funded_amount);
        let sponsored_user_point = multi_scalar_mul(
            [G1, G1, G1],
            [EmbeddedCurveScalar {
                lo: sponsored_user_lo,
                hi: sponsored_user_hi
            },
            EmbeddedCurveScalar {
                lo: funded_amount_lo,
                hi: funded_amount_hi
            },
            EmbeddedCurveScalar {
                lo: refund_nonce_lo,
                hi: refund_nonce_hi
            }]
        );

        [EmbeddedCurvePoint {
            x: fee_payer_point[0],
            y: fee_payer_point[1],
            is_infinite: fee_payer_point[2] == 1
        },EmbeddedCurvePoint {
            x: sponsored_user_point[0],
            y: sponsored_user_point[1],
            is_infinite: sponsored_user_point[2] == 1
        } ]
    }

    fn complete_refund(fee_payer_point: EmbeddedCurvePoint, sponsored_user_point: EmbeddedCurvePoint, transaction_fee: Field) -> [Field; 2] {

        let (transaction_fee_lo, transaction_fee_hi) = decompose(transaction_fee);
        let fee_point_raw = multi_scalar_mul(
            [G1],
            [EmbeddedCurveScalar {
                lo: transaction_fee_lo,
                hi: transaction_fee_hi,
            }]
        );
        let fee_point = EmbeddedCurvePoint {
            x: fee_point_raw[0],
            y: fee_point_raw[1],
            is_infinite: fee_point_raw[2] ==1
        };

        /**
        What is happening here?

        Back up in generate_refund_points, we created two points on the grumpkin curve;
        these are going to be eventually turned into notes:
        one for the user, and one for the FPC.

        So you can think of these (x,y) points as "partial notes": they encode part of the internals of the notes.

        This is because the compute_note_content_hash function above defines the the content hash to be
        the x-coordinate of a point defined as:

        amount * G + npk * G + randomness * G
          = (amount + npk + randomness) * G
        
        where G is a generator point. Interesting point here is that we actually need to convert
        - amount
        - npk
        - randomness
        from grumpkin Field elements
        (which have a modulus of 21888242871839275222246405745257275088548364400416034343698204186575808495617)
        into a grumpkin scalar
        (which have a modulus of 21888242871839275222246405745257275088696311157297823662689037894645226208583)

        The intuition for this is that the Field elements define the domain of the x,y coordinates for points on the curves,
        but the number of points on the curve is actually greater than the size of that domain.

        (Consider, e.g. if the curve were defined over a field of 10 elements, and each x coord had two corresponding y for +/-)

        For a bit more info, see
        https://hackmd.io/@aztec-network/ByzgNxBfd#2-Grumpkin---A-curve-on-top-of-BN-254-for-SNARK-efficient-group-operations


        Anyway, if we have a secret scalar n := amount + npk + randomness, and then we reveal a point n * G, there is no efficient way to
        deduce what n is. This is the discrete log problem.

        However we can still perform addition/subtraction on points! That is why we generate those two points, which are:
        fee_payer_point := (fee_payer_npk + nonce) * G
        sponsored_user_point := (sponsored_user_npk + funded_amount + nonce) * G

        where `funded_amount` is the total amount in tokens that the sponsored user initially supplied, from which the transaction fee will be subtracted.

        So we pass those points into the teardown function (here) and compute a third point corresponding to the transaction fee as just

        fee_point := transaction_fee * G

        Then we arrive at the final points via addition/subtraction of that transaction fee point:

        completed_fpc_point := fee_payer_point + fee_point
                             = (fee_payer_npk + nonce) * G + transaction_fee * G
                             = (fee_payer_npk + nonce + transaction_fee) * G

        completed_user_point := sponsored_user_point - fee_point
                              = (sponsored_user_npk + funded_amount + nonce) * G - transaction_fee * G
                              = (sponsored_user_npk + nonce + (funded_amount - transaction_fee)) * G
        
        When we return the x-coordinate of those points, it identically matches the note_content_hash of (and therefore *is*) notes like:
        {
            amount: (funded_amount - transaction_fee),
            npk_m_hash: sponsored_user_npk,
            randomness: nonce
        }
        */

        let completed_fpc_point = fee_payer_point + fee_point;

        let completed_user_point = sponsored_user_point - fee_point;
        assert_eq(completed_user_point.is_infinite, false);

        [completed_fpc_point.x, completed_user_point.x]
    }
}
