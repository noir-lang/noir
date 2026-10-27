use acir::BlackBoxFunc;
use ecdsa::{EcdsaCurve, Signature, SignatureSize};
use k256::elliptic_curve::{
    AffinePoint, CurveArithmetic, FieldBytes, FieldBytesSize, ProjectivePoint, PublicKey, Scalar,
    array::ArraySize,
    bigint::U256,
    ff::Field,
    group::{Curve, Group},
    ops::Reduce,
    scalar::IsHigh,
    sec1::{FromSec1Point, ModulusSize, Sec1Point, ToSec1Point},
};

use crate::BlackBoxResolutionError;

mod secp256k1;
mod secp256r1;

pub fn ecdsa_secp256k1_verify(
    hashed_msg: &[u8; 32],
    public_key_x: &[u8; 32],
    public_key_y: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError> {
    verify_signature::<k256::Secp256k1>(
        BlackBoxFunc::EcdsaSecp256k1,
        hashed_msg,
        public_key_x,
        public_key_y,
        signature,
    )
}

pub fn ecdsa_secp256r1_verify(
    hashed_msg: &[u8; 32],
    public_key_x: &[u8; 32],
    public_key_y: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError> {
    verify_signature::<p256::NistP256>(
        BlackBoxFunc::EcdsaSecp256r1,
        hashed_msg,
        public_key_x,
        public_key_y,
        signature,
    )
}

/// Verifies an ECDSA signature over the elliptic curve `C`.
///
/// # Parameters:
///
/// * `func` - The black box function being solved, used in error reports
/// * `hashed_msg` - The 32-byte hash of the message that was signed
/// * `public_key_x_bytes` - The x-coordinate of the public key (32 bytes, big-endian)
/// * `public_key_y_bytes` - The y-coordinate of the public key (32 bytes, big-endian)
/// * `signature` - The 64-byte signature in (r, s) format, where r and s are 32 bytes each
///
/// Returns `true` if the signature is valid, `false` otherwise.
///
/// The function does not validate a signature if any of the following are true:
/// - The signature is not "low S" normalized per BIP 0062 to prevent malleability
/// - The signature components `r` and `s` is zero
/// - The public key point is not on the curve
///
/// If `hashed_msg >= C::ORDER`, the message hash is reduced modulo the curve
/// order per ECDSA specification (SEC 1, section 4.1.4). The x-coordinate of the
/// recovered point is likewise reduced modulo the curve order before comparison with `r`.
fn verify_signature<C>(
    func: BlackBoxFunc,
    hashed_msg: &[u8; 32],
    public_key_x_bytes: &[u8; 32],
    public_key_y_bytes: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError>
where
    C: EcdsaCurve + CurveArithmetic,
    SignatureSize<C>: ArraySize,
    FieldBytesSize<C>: ModulusSize,
    AffinePoint<C>: FromSec1Point<C> + ToSec1Point<C>,
    FieldBytes<C>: From<[u8; 32]>,
    Scalar<C>: Reduce<U256> + IsHigh,
{
    #[allow(non_snake_case)]
    let Some((R, r)) =
        nonce_point::<C>(hashed_msg, public_key_x_bytes, public_key_y_bytes, signature)
    else {
        return Ok(false);
    };

    // Compare R.x with signature's r component.
    match R.to_sec1_point(false).coordinates() {
        k256::elliptic_curve::sec1::Coordinates::Uncompressed { x, y: _ } => {
            // Per ECDSA specification (SEC 1, section 4.1.4), R.x is reduced modulo the curve
            // order before comparing it against `r`, as R.x may lie in `[C::ORDER, p)`.
            let v = <Scalar<C> as Reduce<U256>>::reduce(&U256::from_be_slice(x));
            Ok(v == r)
        }
        k256::elliptic_curve::sec1::Coordinates::Identity => Ok(false),
        _ => {
            Err(BlackBoxResolutionError::Failed(func, "Unexpected coordinate encoding".to_string()))
        }
    }
}

/// Computes the point `R = z/s * G + r/s * Q` whose x-coordinate an ECDSA verifier compares
/// against the signature's `r` component, returning it alongside `r`.
///
/// Returns `None` if the inputs fail any of the checks listed on [`verify_signature`].
fn nonce_point<C>(
    hashed_msg: &[u8; 32],
    public_key_x_bytes: &[u8; 32],
    public_key_y_bytes: &[u8; 32],
    signature: &[u8; 64],
) -> Option<(AffinePoint<C>, Scalar<C>)>
where
    C: EcdsaCurve + CurveArithmetic,
    SignatureSize<C>: ArraySize,
    FieldBytesSize<C>: ModulusSize,
    AffinePoint<C>: FromSec1Point<C> + ToSec1Point<C>,
    FieldBytes<C>: From<[u8; 32]>,
    Scalar<C>: Reduce<U256> + IsHigh,
{
    // Convert the inputs into the curve's data structures
    let Ok(signature) = Signature::<C>::try_from(signature.as_slice()) else {
        // Signature `r` and `s` are forbidden from being zero.
        log::warn!("Signature provided for ECDSA verification is zero");
        return None;
    };

    let point = Sec1Point::<C>::from_affine_coordinates(
        &FieldBytes::<C>::from(*public_key_x_bytes),
        &FieldBytes::<C>::from(*public_key_y_bytes),
        false,
    );

    let pubkey = PublicKey::<C>::from_sec1_point(&point);
    if pubkey.is_none().into() {
        // Public key must sit on the curve.
        log::warn!("Invalid public key provided for ECDSA verification");
        return None;
    }
    let pubkey = pubkey.unwrap();

    // Convert the hashed message to a scalar.
    // Per ECDSA specification (SEC 1, section 4.1.4), if `hashed_msg >= C::ORDER`,
    // the message hash should be reduced modulo the curve order.
    let z = <Scalar<C> as Reduce<U256>>::reduce(&U256::from_be_slice(hashed_msg));

    // Finished converting bytes into data structures

    let r = signature.r();
    let s = signature.s();

    // Ensure signature is "low S" normalized ala BIP 0062
    if s.is_high().into() {
        log::warn!(
            "Signature provided for ECDSA verification is not properly normalized (high S value)"
        );
        return None;
    }

    let s_inv = s.as_ref().invert().unwrap();
    let u1 = z * s_inv;
    let u2 = *r * s_inv;

    #[allow(non_snake_case)]
    let R: AffinePoint<C> = ((ProjectivePoint::<C>::generator() * u1)
        + (ProjectivePoint::<C>::from(*pubkey.as_affine()) * u2))
        .to_affine();

    Some((R, *r))
}

/// The x-coordinate of [`nonce_point`] as an integer, without reducing it modulo the curve order.
#[cfg(test)]
fn nonce_point_x<C>(
    hashed_msg: &[u8; 32],
    public_key_x_bytes: &[u8; 32],
    public_key_y_bytes: &[u8; 32],
    signature: &[u8; 64],
) -> U256
where
    C: EcdsaCurve + CurveArithmetic,
    SignatureSize<C>: ArraySize,
    FieldBytesSize<C>: ModulusSize,
    AffinePoint<C>: FromSec1Point<C> + ToSec1Point<C>,
    FieldBytes<C>: From<[u8; 32]>,
    Scalar<C>: Reduce<U256> + IsHigh,
{
    let (point, _) =
        nonce_point::<C>(hashed_msg, public_key_x_bytes, public_key_y_bytes, signature).unwrap();
    U256::from_be_slice(point.to_sec1_point(false).x().unwrap())
}
