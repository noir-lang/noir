use crate::local_module::{local_fn, LocalStruct, DefinedInLocalModule, LOCAL_GLOBAL_N, LOCAL_GLOBAL_M};

use dep::library::{ReExportMeFromAnotherLib, LibLocalStruct, LibDefinedInLocalModule, Liblocal_fn, IMPORT_GLOBAL_N, IMPORT_GLOBAL_M};

// overlapping
// use dep::library2::ReExportMeFromAnotherLib;
use dep::library2::{PubLibLocalStruct, PubLibDefinedInLocalModule, PubLiblocal_fn, IMPORT_GLOBAL_N_2, IMPORT_GLOBAL_M_2};
use dep::library3::{ReExportMeFromAnotherLib2, PubCrateLibLocalStruct, PubCrateLibDefinedInLocalModule, PubCrateLiblocal_fn, IMPORT_GLOBAL_N_3, IMPORT_GLOBAL_M_3};

mod local_module;

fn main(_x: ReExportMeFromAnotherLib, _y: ReExportMeFromAnotherLib2) {
    assert(LOCAL_GLOBAL_N != LOCAL_GLOBAL_M);
    assert(IMPORT_GLOBAL_N != IMPORT_GLOBAL_M);
    assert(IMPORT_GLOBAL_N_2 != IMPORT_GLOBAL_M_2);
    assert(IMPORT_GLOBAL_N_3 != IMPORT_GLOBAL_M_3);

    let x: LocalStruct<LOCAL_GLOBAL_N, LOCAL_GLOBAL_M, Field, bool> = Default::default();
    assert(x.trait_fn5([0; LOCAL_GLOBAL_N], [false; LOCAL_GLOBAL_M]).is_none());
    assert(x.trait_fn6([0; LOCAL_GLOBAL_N], [false; LOCAL_GLOBAL_M]).is_none());

    let x: LibLocalStruct<IMPORT_GLOBAL_N, IMPORT_GLOBAL_M, Field, bool> = Default::default();
    assert(x.lib_trait_fn5([0; IMPORT_GLOBAL_N], [false; IMPORT_GLOBAL_M]).is_none());
    assert(x.lib_trait_fn6([0; IMPORT_GLOBAL_N], [false; IMPORT_GLOBAL_M]).is_none());

    let x: PubLibLocalStruct<IMPORT_GLOBAL_N_2, IMPORT_GLOBAL_M_2, Field, bool> = Default::default();
    assert(x.pub_trait_fn5([0; IMPORT_GLOBAL_N_2], [false; IMPORT_GLOBAL_M_2]).is_none());
    assert(x.pub_trait_fn6([0; IMPORT_GLOBAL_N_2], [false; IMPORT_GLOBAL_M_2]).is_none());

    let x: PubCrateLibLocalStruct<IMPORT_GLOBAL_N_3, IMPORT_GLOBAL_M_3, Field, bool> = Default::default();
    assert(x.crate_trait_fn5([0; IMPORT_GLOBAL_N_3], [false; IMPORT_GLOBAL_M_3]).is_none());
    assert(x.crate_trait_fn6([0; IMPORT_GLOBAL_N_3], [false; IMPORT_GLOBAL_M_3]).is_none());

    assert(local_fn((0, 1), [], []).is_none());
    assert(Liblocal_fn((0, 1), [], []).is_none());
    assert(PubLiblocal_fn((0, 1), [], []).is_none());
    assert(PubCrateLiblocal_fn((0, 1), [], []).is_none());
}

