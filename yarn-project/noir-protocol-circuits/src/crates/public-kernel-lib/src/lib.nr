// TODO: rename to be precise as to what its common to (the public kernel circuits).
mod common;

mod hash;
mod utils;

mod public_kernel_setup;
mod public_kernel_app_logic;

use public_kernel_setup::PublicKernelSetupCircuitPrivateInputs;
use public_kernel_app_logic::PublicKernelAppLogicCircuitPrivateInputs;
