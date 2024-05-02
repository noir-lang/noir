
mod local_module {
    global LOCAL_GLOBAL_N: Field = 0;

    global LOCAL_GLOBAL_M: Field = 1;

    struct LocalStruct<N, M, A, B> {
        field1: A,
        field2: B,
        field3: [A; N],
        field4: ([A; N], [B; M]),
        field5: &mut A,
    }

    impl<N, M, A, B> Default for LocalStruct<N, M, A, B> where A: Default, B: Default {
        fn default() -> Self {
            let mut mut_field = &mut Default::default();
            Self {
                field1: Default::default(),
                field2: Default::default(),
                field3: Default::default(),
                field4: Default::default(),
                field5: mut_field,
            }
        }
    }

    trait DefinedInLocalModule<A, B, N, M> {
        fn trait_fn1(self, x: A);
        fn trait_fn2(self, y: B);
        fn trait_fn3(&mut self, x: A, y: B);
        fn trait_fn4(self, x: Option<A>, y: [B]);
        fn trait_fn5(self, x: [A; N], y: [B; M]) -> Option<A>;
        fn trait_fn6(self, x: [A; N], y: [B; M]) -> Option<A>;
        fn trait_fn7(self, x: fn(Option<A>) -> B) -> Field {
            N + M
        }
    }

    impl<N, M, A, B> DefinedInLocalModule<A, B, N, M> for LocalStruct<N, M, A, B> {
        fn trait_fn1(self, x: A) { }
        fn trait_fn2(self, y: B) { }
        fn trait_fn3(&mut self, x: A, y: B) { }
        fn trait_fn4(self, x: Option<A>, y: [B]) { }
        fn trait_fn5(self, x: [A; N], y: [B; M]) -> Option<A> { Option::none() }
        fn trait_fn6(self, x: [A; N], y: [B; M]) -> Option<A> { Option::none() }
    }

    pub fn local_fn<A, N, B, M>(x: (A, B), y: [Field; N], z: [Field; M]) -> Option<A> {
        assert(LOCAL_GLOBAL_N != LOCAL_GLOBAL_M);
        let x: Field = 0;
        assert(x == 0);
        let x: Field = 1;
        assert(x == 1);
        Option::none()
    }
}

mod library {
    mod library2 {
        use dep::std::option::Option;

        global IMPORT_GLOBAL_N_2: Field = 4;

        global IMPORT_GLOBAL_M_2: Field = 5;

        // When we re-export this type from another library and then use it in
        // main, we get a panic
        struct ReExportMeFromAnotherLib {
            x : Field,
        }

        struct PubLibLocalStruct<N, M, A, B> {
            pub_field1: A,
            pub_field2: B,
            pub_field3: [A; N],
            pub_field4: ([A; N], [B; M]),
            pub_field5: &mut A,
        }

        impl<N, M, A, B> Default for PubLibLocalStruct<N, M, A, B> where A: Default, B: Default {
            fn default() -> Self {
                let mut mut_field = &mut Default::default();
                Self {
                    pub_field1: Default::default(),
                    pub_field2: Default::default(),
                    pub_field3: Default::default(),
                    pub_field4: Default::default(),
                    pub_field5: mut_field,
                }
            }
        }

        trait PubLibDefinedInLocalModule<A, B, N, M> {
            fn pub_trait_fn1(self, x: A);
            fn pub_trait_fn2(self, y: B);
            fn pub_trait_fn3(&mut self, x: A, y: B);
            fn pub_trait_fn4(self, x: Option<A>, y: [B]);
            fn pub_trait_fn5(self, x: [A; N], y: [B; M]) -> Option<A>;
            fn pub_trait_fn6(self, x: [A; N], y: [B; M]) -> Option<A>;
            fn pub_trait_fn7(self, x: fn(Option<A>) -> B) -> Field {
                N + M
            }
        }

        impl<N, M, A, B> PubLibDefinedInLocalModule<A, B, N, M> for PubLibLocalStruct<N, M, A, B> {
            fn pub_trait_fn1(self, x: A) { }
            fn pub_trait_fn2(self, y: B) { }
            fn pub_trait_fn3(&mut self, x: A, y: B) { }
            fn pub_trait_fn4(self, x: Option<A>, y: [B]) { }
            fn pub_trait_fn5(self, x: [A; N], y: [B; M]) -> Option<A> { Option::none() }
            fn pub_trait_fn6(self, x: [A; N], y: [B; M]) -> Option<A> { Option::none() }
        }

        pub fn PubLiblocal_fn<A, N, B, M>(x: (A, B), y: [Field; N], z: [Field; M]) -> Option<A> {
            assert(IMPORT_GLOBAL_N_2 != IMPORT_GLOBAL_M_2);
            Option::none()
        }
    }

    // Re-export 
    use library2::ReExportMeFromAnotherLib;

    global IMPORT_GLOBAL_N: Field = 2;

    global IMPORT_GLOBAL_M: Field = 3;

    struct LibLocalStruct<N, M, A, B> {
        lib_field1: A,
        lib_field2: B,
        lib_field3: [A; N],
        lib_field4: ([A; N], [B; M]),
        lib_field5: &mut A,
    }

    impl<N, M, A, B> Default for LibLocalStruct<N, M, A, B> where A: Default, B: Default {
        fn default() -> Self {
            let mut mut_field = &mut Default::default();
            Self {
                lib_field1: Default::default(),
                lib_field2: Default::default(),
                lib_field3: Default::default(),
                lib_field4: Default::default(),
                lib_field5: mut_field,
            }
        }
    }

    trait LibDefinedInLocalModule<A, B, N, M> {
        fn lib_trait_fn1(self, x: A);
        fn lib_trait_fn2(self, y: B);
        fn lib_trait_fn3(&mut self, x: A, y: B);
        fn lib_trait_fn4(self, x: Option<A>, y: [B]);
        fn lib_trait_fn5(self, x: [A; N], y: [B; M]) -> Option<A>;
        fn lib_trait_fn6(self, x: [A; N], y: [B; M]) -> Option<A>;
        fn lib_trait_fn7(self, x: fn(Option<A>) -> B) -> Field {
            N + M
        }
    }

    impl<N, M, A, B> LibDefinedInLocalModule<A, B, N, M> for LibLocalStruct<N, M, A, B> {
        fn lib_trait_fn1(self, x: A) { }
        fn lib_trait_fn2(self, y: B) { }
        fn lib_trait_fn3(&mut self, x: A, y: B) { }
        fn lib_trait_fn4(self, x: Option<A>, y: [B]) { }
        fn lib_trait_fn5(self, x: [A; N], y: [B; M]) -> Option<A> { Option::none() }
        fn lib_trait_fn6(self, x: [A; N], y: [B; M]) -> Option<A> { Option::none() }
    }

    pub fn Liblocal_fn<A, N, B, M>(x: (A, B), y: [Field; N], z: [Field; M]) -> Option<A> {
        assert(IMPORT_GLOBAL_N != IMPORT_GLOBAL_M);
        Option::none()
    }
}

mod library3 {
    global IMPORT_GLOBAL_N_3: Field = 6;

    global IMPORT_GLOBAL_M_3: Field = 7;

    struct ReExportMeFromAnotherLib2 {
        x : Field,
    }

    struct PubCrateLibLocalStruct<N, M, A, B> {
        crate_field1: A,
        crate_field2: B,
        crate_field3: [A; N],
        crate_field4: ([A; N], [B; M]),
        crate_field5: &mut A,
    }

    impl<N, M, A, B> Default for PubCrateLibLocalStruct<N, M, A, B> where A: Default, B: Default {
        fn default() -> Self {
            let mut mut_field = &mut Default::default();
            Self {
                crate_field1: Default::default(),
                crate_field2: Default::default(),
                crate_field3: Default::default(),
                crate_field4: Default::default(),
                crate_field5: mut_field,
            }
        }
    }

    trait PubCrateLibDefinedInLocalModule<A, B, N, M> {
        fn crate_trait_fn1(self, x: A);
        fn crate_trait_fn2(self, y: B);
        fn crate_trait_fn3(&mut self, x: A, y: B);
        fn crate_trait_fn4(self, x: Option<A>, y: [B]);
        fn crate_trait_fn5(self, x: [A; N], y: [B; M]) -> Option<A>;
        fn crate_trait_fn6(self, x: [A; N], y: [B; M]) -> Option<A>;
        fn crate_trait_fn7(self, x: fn(Option<A>) -> B) -> Field {
            N + M
        }
    }

    impl<N, M, A, B> PubCrateLibDefinedInLocalModule<A, B, N, M> for PubCrateLibLocalStruct<N, M, A, B> {
        fn crate_trait_fn1(self, x: A) { }
        fn crate_trait_fn2(self, y: B) { }
        fn crate_trait_fn3(&mut self, x: A, y: B) { }
        fn crate_trait_fn4(self, x: Option<A>, y: [B]) { }
        fn crate_trait_fn5(self, x: [A; N], y: [B; M]) -> Option<A> { Option::none() }
        fn crate_trait_fn6(self, x: [A; N], y: [B; M]) -> Option<A> { Option::none() }
    }

    pub(crate) fn PubCrateLiblocal_fn<A, N, B, M>(x: (A, B), y: [Field; N], z: [Field; M]) -> Option<A> {
        assert(IMPORT_GLOBAL_N_3 != IMPORT_GLOBAL_M_3);
        Option::none()
    }
}


use local_module::{local_fn, LocalStruct, DefinedInLocalModule, LOCAL_GLOBAL_N, LOCAL_GLOBAL_M};

use library::{ReExportMeFromAnotherLib, LibLocalStruct, LibDefinedInLocalModule, Liblocal_fn, IMPORT_GLOBAL_N, IMPORT_GLOBAL_M};

// overlapping
// use library::library2::ReExportMeFromAnotherLib;
use library::library2::{PubLibLocalStruct, PubLibDefinedInLocalModule, PubLiblocal_fn, IMPORT_GLOBAL_N_2, IMPORT_GLOBAL_M_2};

use library3::{ReExportMeFromAnotherLib2, PubCrateLibLocalStruct, PubCrateLibDefinedInLocalModule, PubCrateLiblocal_fn, IMPORT_GLOBAL_N_3, IMPORT_GLOBAL_M_3};


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

