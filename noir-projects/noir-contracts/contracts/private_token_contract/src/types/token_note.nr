use dep::aztec::{
    prelude::{AztecAddress, NoteHeader, NoteInterface, PrivateContext},
    protocol_types::{constants::GENERATOR_INDEX__NOTE_NULLIFIER, point::Point, hash::poseidon2_hash},
    note::utils::compute_note_hash_for_consumption, oracle::unsafe_rand::unsafe_rand,
    keys::getters::get_nsk_app, note::note_getter_options::PropertySelector
};
use dep::std::field::bn254::decompose;
use dep::std::embedded_curve_ops::{EmbeddedCurveScalar, multi_scalar_mul, fixed_base_scalar_mul};

trait OwnedNote {
    fn new(amount: U128, owner_npk_m_hash: Field) -> Self;
    fn get_amount(self) -> U128;
    fn get_owner_npk_m_hash(self) -> Field;
    fn get_owner_selector() -> PropertySelector;
}

trait PrivatelyRefundable {
    fn generate_refund_points(
        fee_payer_npk_m_hash: Field,
        user_npk_m_hash: Field,
        funded_amount: Field,
        user_randomness: Field,
        fee_payer_randomness: Field
    ) -> (Point, Point);

    fn complete_refund(
        incomplete_fee_payer_point: Point,
        incomplete_user_point: Point,
        transaction_fee: Field
    ) -> (Field, Field);
}

global TOKEN_NOTE_LEN: Field = 3; // 3 plus a header.
global TOKEN_NOTE_BYTES_LEN: Field = 3 * 32 + 64;
// Grumpkin generator point. 
global G1 = Point { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };

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
        // We compute the note content hash as an x-coordinate of `G ^ (amount + npk_m_hash + randomness)` instead
        // of using pedersen or poseidon2 because it allows us to privately add and subtract from amount in public
        // by leveraging homomorphism.
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

/**
 * What is happening below?
 *
 * First in generate_refund_points, we create two points on the grumpkin curve;
 * these are going to be eventually turned into notes:
 * one for the user, and one for the fee payer.
 *
 * So you can think of these (x,y) points as "partial notes": they encode part of the internals of the notes.
 *
 * This is because the compute_note_content_hash function above defines the content hash to be
 * the x-coordinate of a point defined as:
 *
 * amount * G + npk * G + randomness * G
 *   = (amount + npk + randomness) * G
 * 
 * where G is a generator point. Interesting point here is that we actually need to convert
 * - amount
 * - npk
 * - randomness
 * from grumpkin Field elements
 * (which have a modulus of 21888242871839275222246405745257275088548364400416034343698204186575808495617)
 * into a grumpkin scalar
 * (which have a modulus of 21888242871839275222246405745257275088696311157297823662689037894645226208583)
 *
 * The intuition for this is that the Field elements define the domain of the x,y coordinates for points on the curves,
 * but the number of points on the curve is actually greater than the size of that domain.
 *
 * (Consider, e.g. if the curve were defined over a field of 10 elements, and each x coord had two corresponding y for +/-)
 *
 * For a bit more info, see
 * https://hackmd.io/@aztec-network/ByzgNxBfd#2-Grumpkin---A-curve-on-top-of-BN-254-for-SNARK-efficient-group-operations
 *
 *
 * Anyway, if we have a secret scalar n := amount + npk + randomness, and then we reveal a point n * G, there is no efficient way to
 * deduce what n is. This is the discrete log problem.
 *
 * However we can still perform addition/subtraction on points! That is why we generate those two points, which are:
 * incomplete_fee_payer_point := (fee_payer_npk + fee_payer_randomness) * G
 * incomplete_user_point := (user_npk + funded_amount + user_randomness) * G
 *
 * where `funded_amount` is the total amount in tokens that the sponsored user initially supplied, from which the transaction fee will be subtracted.
 *
 * So we pass those points into the teardown function (here) and compute a third point corresponding to the transaction fee as just
 *
 * fee_point := transaction_fee * G
 *
 * Then we arrive at the final points via addition/subtraction of that transaction fee point:
 *
 * fee_payer_point := incomplete_fee_payer_point + fee_point
 *                      = (fee_payer_npk + fee_payer_randomness) * G + transaction_fee * G
 *                      = (fee_payer_npk + fee_payer_randomness + transaction_fee) * G
 *
 * user_point := incomplete_user_point - fee_point
 *                       = (user_npk + funded_amount + user_randomness) * G - transaction_fee * G
 *                       = (user_npk + user_randomness + (funded_amount - transaction_fee)) * G
 * 
 * When we return the x-coordinate of those points, it identically matches the note_content_hash of (and therefore *is*) notes like:
 * {
 *     amount: (funded_amount - transaction_fee),
 *     npk_m_hash: user_npk,
 *     randomness: user_randomness
 * }
 *
 * Why do we need different randomness for the user and the fee payer notes?
 * --> This is because if the randomness values were the same we could fingerprint the user by doing the following:
 *      1) randomness_influence = incomplete_fee_payer_point - G * fee_payer_npk =
 *                              = (fee_payer_npk + randomness) * G - G * fee_payer_npk = randomness * G
 *      2) user_fingerprint = incomplete_user_point - G * funded_amount - randomness_influence =
 *                          = (user_npk + funded_amount + randomness) * G - funded_amount * G - randomness * G =
 *                          = user_npk * G
 *      3) Then the second time the user would use this fee paying contract we would recover the same fingerprint and
 *         link that the 2 transactions were made by the same user. Given that it's expected that only a limited set
 *         of fee paying contracts will be used and they will be known searching for fingerprints by trying different
 *         fee payer npk values of these known contracts is a feasible attack.
 */
impl PrivatelyRefundable for TokenNote {
    fn generate_refund_points(fee_payer_npk_m_hash: Field, user_npk_m_hash: Field, funded_amount: Field, user_randomness: Field, fee_payer_randomness: Field) -> (Point, Point) {
        // 1. To be able to multiply generators with randomness and npk_m_hash using barretneberg's (BB) blackbox function we
        // first need to convert the fields to high and low limbs.
        let (fee_payer_randomness_lo, fee_payer_randomness_hi) = decompose(fee_payer_randomness);
        let (fee_payer_npk_m_hash_lo, fee_payer_npk_m_hash_hi) = decompose(fee_payer_npk_m_hash);

        // 2. Now that we have correct representationsn of fee payer and randomness we can compute `G ^ (fee_payer_npk + randomness)`
        let incomplete_fee_payer_point = multi_scalar_mul(
            [G1, G1],
            [EmbeddedCurveScalar {
                lo: fee_payer_npk_m_hash_lo,
                hi: fee_payer_npk_m_hash_hi
            },
            EmbeddedCurveScalar {
                lo: fee_payer_randomness_lo,
                hi: fee_payer_randomness_hi
            }]
        );

        // 3. We do the necessary conversion for values relevant for the sponsored user point.
        let (user_randomness_lo, user_randomness_hi) = decompose(user_randomness);
        // TODO(#7324): representing user with their npk_m_hash here does not work with key rotation
        let (user_lo, user_hi) = decompose(user_npk_m_hash);
        let (funded_amount_lo, funded_amount_hi) = decompose(funded_amount);

        // 4. We compute `G ^ (user_npk_m_hash + funded_amount + randomness)`
        let incomplete_user_point = multi_scalar_mul(
            [G1, G1, G1],
            [EmbeddedCurveScalar {
                lo: user_lo,
                hi: user_hi
            },
            EmbeddedCurveScalar {
                lo: funded_amount_lo,
                hi: funded_amount_hi
            },
            EmbeddedCurveScalar {
                lo: user_randomness_lo,
                hi: user_randomness_hi
            }]
        );

        // 5. At last we represent the points as Points and return them.
        (Point {
            x: incomplete_fee_payer_point[0],
            y: incomplete_fee_payer_point[1],
            is_infinite: incomplete_fee_payer_point[2] == 1
        }, Point {
            x: incomplete_user_point[0],
            y: incomplete_user_point[1],
            is_infinite: incomplete_user_point[2] == 1
        })
    }

    fn complete_refund(incomplete_fee_payer_point: Point, incomplete_user_point: Point, transaction_fee: Field) -> (Field, Field) {
        // 1. We convert the transaction fee to high and low limbs to be able to use BB API.
        let (transaction_fee_lo, transaction_fee_hi) = decompose(transaction_fee);

        // 2. We compute the fee point as `G ^ transaction_fee`
        let fee_point_raw = multi_scalar_mul(
            [G1],
            [EmbeddedCurveScalar {
                lo: transaction_fee_lo,
                hi: transaction_fee_hi,
            }]
        );
        let fee_point = Point {
            x: fee_point_raw[0],
            y: fee_point_raw[1],
            is_infinite: fee_point_raw[2] == 1
        };

        // 3. Now we leverage homomorphism to privately add the fee to fee payer point and subtract it from
        // the sponsored user point in public.
        let fee_payer_point = incomplete_fee_payer_point + fee_point;
        let user_point = incomplete_user_point - fee_point;

        assert_eq(user_point.is_infinite, false);

        // Finally we return the x-coordinates of the points which are the note content hashes.
        (fee_payer_point.x, user_point.x)
    }
}
