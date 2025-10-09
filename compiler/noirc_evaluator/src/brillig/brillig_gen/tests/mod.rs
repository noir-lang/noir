// The tests in the following files are simple tests checking the result of generating brillig bytecode
// from a simple brillig function in SSA form. The brillig function is doing one elementary operation to
// show how it is generated on the Brillig bytecode level.
// Brillig-gen is adding overhead to the elementary operation for hanling the function call.
// See the first test in binary_test.rs for a breakdown of the overhead.
#[cfg(test)]
mod binary_test;
#[cfg(test)]
mod black_box_test;
#[cfg(test)]
mod call_test;
#[cfg(test)]
mod memory_test;
