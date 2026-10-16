pub mod hash;
pub mod aes128;
pub mod array;
pub mod vector;
pub mod ecdsa_secp256k1;
pub mod ecdsa_secp256r1;
pub mod embedded_curve_ops;
pub mod field;
pub mod collections;
pub mod compat;
pub mod convert;
pub mod option;
pub mod string;
pub mod test;
pub mod cmp;
pub mod ops;
pub mod default;
pub mod prelude;
pub mod runtime;
pub mod meta;
pub mod append;
pub mod mem;
pub mod panic;
pub mod hint;

mod integer;
mod primitive_docs;
mod internal;

// Oracle calls are required to be wrapped in an unconstrained function
// Thus, the only argument to the `println` oracle is expected to always be an ident
#[oracle(print)]
unconstrained fn print_oracle<T>(with_newline: bool, input: T) {}

unconstrained fn print_unconstrained<T>(with_newline: bool, input: T) {
    print_oracle(with_newline, input);
}

/// Print the given input to stdout followed by a newline
///
/// It works in constrained, unconstrained and comptime code. It is a debugging aid: it adds no
/// constraints, and nothing about it ends up in a proof. Format strings interpolate values.
///
/// Example:
///
/// ```noir
/// let x = 42;
/// println(x);
/// println(f"x is {x}");
/// ```
pub fn println<T>(input: T) {
    // Safety: a print statement cannot be constrained
    unsafe {
        print_unconstrained(true, input);
    }
}

/// Print the given input to stdout
///
/// Like [`println`], but without a trailing newline.
///
/// Example:
///
/// ```noir
/// print("x = ");
/// println(42);
/// ```
pub fn print<T>(input: T) {
    // Safety: a print statement cannot be constrained
    unsafe {
        print_unconstrained(false, input);
    }
}

/// Asserts the validity of the provided proof and public inputs against the provided verification key and hash.
///
/// The ACVM cannot determine whether the provided proof is valid during execution as this requires knowledge of
/// the backend against which the program is being proven. However if an invalid proof if submitted, the program may
/// fail to prove or the backend may generate a proof which will subsequently fail to verify.
///
/// # Important Note
///
/// If you are not developing your own proving backend, you probably shouldn't need to interact with
/// this function directly. It's easier and safer to use a verification
/// library which is published by the developers of the backend which will document or enforce any safety requirements.
///
/// If you use this directly, you're liable to introduce underconstrainedness bugs and *your circuit will be insecure*.
///
/// # Arguments
/// - verification_key: The verification key of the circuit to be verified.
/// - proof: The proof to be verified.
/// - public_inputs: The public inputs associated with `proof`
/// - key_hash: The hash of `verification_key` of the form expected by the backend.
/// - proof_type: An identifier for the proving scheme used to generate the proof to be verified. This allows
///               for a single backend to support verifying multiple proving schemes.
///
/// # Constraining `key_hash`
///
/// The Noir compiler does not by itself constrain that `key_hash` is a valid hash of `verification_key`.
/// This is because different backends may differ in how they hash their verification keys.
/// It is then the responsibility of either the noir developer (by explicitly hashing the verification key
/// in the correct manner) or by the proving system itself internally asserting the correctness of `key_hash`.
///
/// `proof_type` must be a compile-time constant in constrained code.
///
/// Example:
///
/// The sizes of the key, proof and public inputs, and the meaning of `proof_type`, depend on the
/// backend, so this example doesn't run.
///
/// ```noir ignore
/// fn main(
///     verification_key: [Field; 115],
///     proof: [Field; 93],
///     public_inputs: pub [Field; 1],
///     key_hash: Field,
/// ) {
///     std::verify_proof_with_type(verification_key, proof, public_inputs, key_hash, 1);
/// }
/// ```
pub fn verify_proof_with_type<let N: u32, let M: u32, let K: u32>(
    verification_key: [Field; N],
    proof: [Field; M],
    public_inputs: [Field; K],
    key_hash: Field,
    proof_type: u32,
) {
    if !crate::runtime::is_unconstrained() {
        crate::assert_constant(proof_type);
    }
    verify_proof_internal(verification_key, proof, public_inputs, key_hash, proof_type);
}

#[foreign(recursive_aggregation)]
fn verify_proof_internal<let N: u32, let M: u32, let K: u32>(
    verification_key: [Field; N],
    proof: [Field; M],
    public_inputs: [Field; K],
    key_hash: Field,
    proof_type: u32,
) {}

/// Asserts that the given value is known at compile-time.
/// Useful for debugging for-loop bounds.
///
/// In constrained code, compilation fails if `x` isn't a compile-time constant after the compiler's
/// optimizations. It has no effect in unconstrained code.
///
/// Example:
///
/// ```noir
/// let n = 2 + 3;
/// std::assert_constant(n);
/// ```
#[builtin(assert_constant)]
pub fn assert_constant<T>(x: T) {}

/// Asserts that the given value is both true and known at compile-time.
/// The message can be a string, a format string, or any value, as long as it is known at compile-time
///
/// Compilation fails with `message` if `predicate` is false or isn't known at compile time. It is
/// typically used to check numeric generics.
///
/// Example:
///
/// ```noir
/// fn first<let N: u32>(array: [u32; N]) -> u32 {
///     std::static_assert(N > 0, "the array must not be empty");
///     array[0]
/// }
///
/// fn main() {
///     assert_eq(first([7, 8]), 7);
/// }
/// ```
#[builtin(static_assert)]
pub fn static_assert<T>(predicate: bool, message: T) {}

/// Force a field value to be a witness instead of a constant in the compiled output.
/// This is often only useful for debugging compiler optimizations.
///
/// This has no effect in unconstrained or comptime code.
///
/// Example:
///
/// ```noir
/// let x: Field = 5;
/// std::as_witness(x);
/// assert_eq(x, 5);
/// ```
#[builtin(as_witness)]
pub fn as_witness(x: Field) {}
