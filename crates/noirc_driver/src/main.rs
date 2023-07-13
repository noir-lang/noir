use noirc_driver::{
    add_dep, compile_main, create_local_crate, create_non_local_crate, CompileOptions,
};
use noirc_frontend::{
    graph::{CrateType, LOCAL_CRATE},
    hir::Context,
};
fn main() {
    const EXTERNAL_DIR: &str = "dep_b/lib.nr";
    const EXTERNAL_DIR2: &str = "dep_a/lib.nr";
    const ROOT_DIR_MAIN: &str = "example_real_project/main.nr";

    let mut context = Context::default();

    // Add local crate to dep graph
    create_local_crate(&mut context, ROOT_DIR_MAIN, CrateType::Binary);

    // Add libraries into Driver
    let crate_id1 = create_non_local_crate(&mut context, EXTERNAL_DIR2, CrateType::Library);
    let crate_id2 = create_non_local_crate(&mut context, EXTERNAL_DIR, CrateType::Library);

    // Add dependencies as package
    add_dep(&mut context, LOCAL_CRATE, crate_id1, "coo4");
    add_dep(&mut context, LOCAL_CRATE, crate_id2, "coo3");

    compile_main(&mut context, &CompileOptions::default()).ok();
}
