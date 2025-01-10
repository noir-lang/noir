use acir::BlackBoxFunc;

use crate::BlackBoxResolutionError;

/// This component will generate outputs for Blackbox function calls where the underlying [`acir::BlackBoxFunc`]
/// doesn't have a canonical Rust implementation.
///
/// Returns an [`BlackBoxResolutionError`] if the backend does not support the given [`acir::BlackBoxFunc`].
pub trait BlackBoxFunctionSolver<F> {
    fn pedantic_solving(&self) -> bool;
    fn multi_scalar_mul(
        &self,
        points: &[F],
        scalars_lo: &[F],
        scalars_hi: &[F],
    ) -> Result<(F, F, F), BlackBoxResolutionError>;
    fn ec_add(
        &self,
        input1_x: &F,
        input1_y: &F,
        input1_infinite: &F,
        input2_x: &F,
        input2_y: &F,
        input2_infinite: &F,
    ) -> Result<(F, F, F), BlackBoxResolutionError>;
    fn poseidon2_permutation(
        &self,
        _inputs: &[F],
        _len: u32,
    ) -> Result<Vec<F>, BlackBoxResolutionError>;
}

// pedantic_solving: bool
pub struct StubbedBlackBoxSolver(pub bool);

// pedantic_solving enabled by default
impl Default for StubbedBlackBoxSolver {
    fn default() -> StubbedBlackBoxSolver {
        let pedantic_solving = true;
        StubbedBlackBoxSolver(pedantic_solving)
    }
}

impl StubbedBlackBoxSolver {
    fn fail(black_box_function: BlackBoxFunc) -> BlackBoxResolutionError {
        BlackBoxResolutionError::Failed(
            black_box_function,
            format!("{} is not supported", black_box_function.name()),
        )
    }
}

impl<F> BlackBoxFunctionSolver<F> for StubbedBlackBoxSolver {
    fn pedantic_solving(&self) -> bool {
        self.0
    }
    fn multi_scalar_mul(
        &self,
        _points: &[F],
        _scalars_lo: &[F],
        _scalars_hi: &[F],
    ) -> Result<(F, F, F), BlackBoxResolutionError> {
        Err(Self::fail(BlackBoxFunc::MultiScalarMul))
    }
    fn ec_add(
        &self,
        _input1_x: &F,
        _input1_y: &F,
        _input1_infinite: &F,
        _input2_x: &F,
        _input2_y: &F,
        _input2_infinite: &F,
    ) -> Result<(F, F, F), BlackBoxResolutionError> {
        Err(Self::fail(BlackBoxFunc::EmbeddedCurveAdd))
    }
    fn poseidon2_permutation(
        &self,
        _inputs: &[F],
        _len: u32,
    ) -> Result<Vec<F>, BlackBoxResolutionError> {
        Err(Self::fail(BlackBoxFunc::Poseidon2Permutation))
    }
}
