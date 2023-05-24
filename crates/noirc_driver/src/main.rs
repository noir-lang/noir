use acvm::Language;
use noirc_driver::{CompileOptions, Driver};
use noirc_frontend::graph::{CrateType, LOCAL_CRATE};
fn main() {
    const EXTERNAL_DIR: &str = "dep_b/lib.nr";
    const EXTERNAL_DIR2: &str = "dep_a/lib.nr";
    const ROOT_DIR_MAIN: &str = "example_real_project/main.nr";

    let mut driver = Driver::new(
        &Language::R1CS,
        #[allow(deprecated)]
        Box::new(acvm::default_is_opcode_supported(Language::R1CS)),
    );

    // Add local crate to dep graph
    driver.create_local_crate(ROOT_DIR_MAIN, CrateType::Binary);

    // Add libraries into Driver
    let crate_id1 = driver.create_non_local_crate(EXTERNAL_DIR2, CrateType::Library);
    let crate_id2 = driver.create_non_local_crate(EXTERNAL_DIR, CrateType::Library);

    // Add dependencies as package
    driver.add_dep(LOCAL_CRATE, crate_id1, "coo4");
    driver.add_dep(LOCAL_CRATE, crate_id2, "coo3");

    driver.compile_main(&CompileOptions::default()).ok();
}
