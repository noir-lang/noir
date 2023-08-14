use acvm::BlackBoxFunctionSolver;
pub(crate) use acvm_backend_barretenberg::Barretenberg as ConcreteBackend;

pub(crate) fn get_black_box_solver() -> impl BlackBoxFunctionSolver {
    #[cfg(feature = "plonk_bn254")]
    return acvm_backend_barretenberg::BarretenbergBlackBoxSolver;

    #[cfg(not(feature = "plonk_bn254"))]
    #[allow(deprecated)]
    return acvm::blackbox_solver::BarretenbergSolver::new();
}

#[cfg(not(any(feature = "plonk_bn254", feature = "plonk_bn254_wasm")))]
compile_error!("please specify a backend to compile with");

#[cfg(all(feature = "plonk_bn254", feature = "plonk_bn254_wasm"))]
compile_error!(
    "feature \"plonk_bn254\"  and feature \"plonk_bn254_wasm\" cannot be enabled at the same time"
);
