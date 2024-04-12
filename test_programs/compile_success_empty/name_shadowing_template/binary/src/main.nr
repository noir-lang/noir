use crate::local_module::local_fn;

use dep::library::{ReExportMeFromAnotherLib, LibLocalStruct, Liblocal_fn};

// overlapping
// use dep::library2::ReExportMeFromAnotherLib;
use dep::library2::{PubLibLocalStruct, PubLiblocal_fn};
use dep::library3::{ReExportMeFromAnotherLib2, PubCrateLibLocalStruct, PubCrateLiblocal_fn};

mod local_module;

fn main(_x: ReExportMeFromAnotherLib, _y: ReExportMeFromAnotherLib2) {
    assert(local_fn((0, 1), [], []).is_none());
    assert(Liblocal_fn((0, 1), [], []).is_none());
    assert(PubLiblocal_fn((0, 1), [], []).is_none());
    assert(PubCrateLiblocal_fn((0, 1), [], []).is_none());
}

