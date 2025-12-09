#![cfg(test)]
use crate::tests::assert_no_errors;

use super::get_program_errors;
use std::collections::HashSet;

#[test]
fn resolve_shadowing() {
    let src = r#"
        fn main(x : Field) {
            let x = foo(x);
            let x = x;
            let (x, _) = (x, x);
            let _ = x;
        }

        fn foo(x : Field) -> Field {
            x
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn test_name_shadowing() {
    // cSpell:disable
    let src = "
    trait Default {
        fn default() -> Self;
    }

    impl Default for bool {
        fn default() -> bool {
            false
        }
    }

    impl Default for Field {
        fn default() -> Field {
            0
        }
    }

    impl<T, N> Default for [T; N] where T: Default {
        fn default() -> [T; N] {
            [Default::default(); N]
        }
    }

    impl<T, U> Default for (T, U) where T: Default, U: Default {
        fn default() -> (T, U) {
            (Default::default(), Default::default())
        }
    }

    fn drop_var<T, U>(_x: T, y: U) -> U { y }

    mod local_module {
        use crate::{Default, drop_var};

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

        trait DefinedInLocalModule1<A, B, N, M> {
            fn trait_fn1(self, x: A);
            fn trait_fn2(self, y: B);
            fn trait_fn3(&mut self, x: A, y: B);
            fn trait_fn4(self, x: [A; 0], y: [B]);
            fn trait_fn5(self, x: [A; N], y: [B; M]) -> [A; 0];
            fn trait_fn6(self, x: [A; N], y: [B; M]) -> [A; 0];
            fn trait_fn7(self, _x: fn([A; 0]) -> B) -> Field {
                drop_var(self, N + M)
            }
        }

        impl<N, M, A, B> DefinedInLocalModule1<A, B, N, M> for LocalStruct<N, M, A, B> {
            fn trait_fn1(self, _x: A) { drop_var(self, ()) }
            fn trait_fn2(self, _y: B) { drop_var(self, ()) }
            fn trait_fn3(&mut self, _x: A, _y: B) { drop_var(self, ()) }
            fn trait_fn4(self, _x: [A; 0], _y: [B]) { drop_var(self, ()) }
            fn trait_fn5(self, _x: [A; N], _y: [B; M]) -> [A; 0] { drop_var(self, []) }
            fn trait_fn6(self, _x: [A; N], _y: [B; M]) -> [A; 0] { drop_var(self, []) }
        }

        pub fn local_fn4<A, N, B, M>(_x: (A, B), _y: [Field; N], _z: [Field; M]) -> [A; 0] {
            assert(LOCAL_GLOBAL_N != LOCAL_GLOBAL_M);
            let x: Field = 0;
            assert(x == 0);
            let x: Field = 1;
            assert(x == 1);
            []
        }
    }

    mod library {
        use crate::{Default, drop_var};

        mod library2 {
            use crate::{Default, drop_var};

            global IMPORT_GLOBAL_N_2: Field = 4;

            global IMPORT_GLOBAL_M_2: Field = 5;

            // When we re-export this type from another library and then use it in
            // main, we get a panic
            struct ReExportMeFromAnotherLib1 {
                x : Field,
            }

            struct PubLibLocalStruct3<N, M, A, B> {
                pub_field1: A,
                pub_field2: B,
                pub_field3: [A; N],
                pub_field4: ([A; N], [B; M]),
                pub_field5: &mut A,
            }

            impl<N, M, A, B> Default for PubLibLocalStruct3<N, M, A, B> where A: Default, B: Default {
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

            trait PubLibDefinedInLocalModule3<A, B, N, M> {
                fn pub_trait_fn1(self, x: A);
                fn pub_trait_fn2(self, y: B);
                fn pub_trait_fn3(&mut self, x: A, y: B);
                fn pub_trait_fn4(self, x: [A; 0], y: [B]);
                fn pub_trait_fn5(self, x: [A; N], y: [B; M]) -> [A; 0];
                fn pub_trait_fn6(self, x: [A; N], y: [B; M]) -> [A; 0];
                fn pub_trait_fn7(self, _x: fn([A; 0]) -> B) -> Field {
                    drop_var(self, N + M)
                }
            }

            impl<N, M, A, B> PubLibDefinedInLocalModule3<A, B, N, M> for PubLibLocalStruct3<N, M, A, B> {
                fn pub_trait_fn1(self, _x: A) { drop_var(self, ()) }
                fn pub_trait_fn2(self, _y: B) { drop_var(self, ()) }
                fn pub_trait_fn3(&mut self, _x: A, _y: B) { drop_var(self, ()) }
                fn pub_trait_fn4(self, _x: [A; 0], _y: [B]) { drop_var(self, ()) }
                fn pub_trait_fn5(self, _x: [A; N], _y: [B; M]) -> [A; 0] { drop_var(self, []) }
                fn pub_trait_fn6(self, _x: [A; N], _y: [B; M]) -> [A; 0] { drop_var(self, []) }
            }

            pub fn PubLiblocal_fn3<A, N, B, M>(_x: (A, B), _y: [Field; N], _z: [Field; M]) -> [A; 0] {
                assert(IMPORT_GLOBAL_N_2 != IMPORT_GLOBAL_M_2);
                []
            }
        }

        // Re-export 
        use library2::ReExportMeFromAnotherLib1;

        global IMPORT_GLOBAL_N_1: Field = 2;

        global IMPORT_GLOBAL_M_1: Field = 3;

        struct LibLocalStruct1<N, M, A, B> {
            lib_field1: A,
            lib_field2: B,
            lib_field3: [A; N],
            lib_field4: ([A; N], [B; M]),
            lib_field5: &mut A,
        }

        impl<N, M, A, B> Default for LibLocalStruct1<N, M, A, B> where A: Default, B: Default {
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

        trait LibDefinedInLocalModule1<A, B, N, M> {
            fn lib_trait_fn1(self, x: A);
            fn lib_trait_fn2(self, y: B);
            fn lib_trait_fn3(&mut self, x: A, y: B);
            fn lib_trait_fn4(self, x: [A; 0], y: [B]);
            fn lib_trait_fn5(self, x: [A; N], y: [B; M]) -> [A; 0];
            fn lib_trait_fn6(self, x: [A; N], y: [B; M]) -> [A; 0];
            fn lib_trait_fn7(self, _x: fn([A; 0]) -> B) -> Field {
                drop_var(self, N + M)
            }
        }

        impl<N, M, A, B> LibDefinedInLocalModule1<A, B, N, M> for LibLocalStruct1<N, M, A, B> {
            fn lib_trait_fn1(self, _x: A) { drop_var(self, ()) }
            fn lib_trait_fn2(self, _y: B) { drop_var(self, ()) }
            fn lib_trait_fn3(&mut self, _x: A, _y: B) { drop_var(self, ()) }
            fn lib_trait_fn4(self, _x: [A; 0], _y: [B]) { drop_var(self, ()) }
            fn lib_trait_fn5(self, _x: [A; N], _y: [B; M]) -> [A; 0] { drop_var(self, []) }
            fn lib_trait_fn6(self, _x: [A; N], _y: [B; M]) -> [A; 0] { drop_var(self, []) }
        }

        pub fn Liblocal_fn1<A, N, B, M>(_x: (A, B), _y: [Field; N], _z: [Field; M]) -> [A; 0] {
            assert(IMPORT_GLOBAL_N_1 != IMPORT_GLOBAL_M_1);
            []
        }
    }

    mod library3 {
        use crate::{Default, drop_var};

        global IMPORT_GLOBAL_N_3: Field = 6;

        global IMPORT_GLOBAL_M_3: Field = 7;

        struct ReExportMeFromAnotherLib2 {
            x : Field,
        }

        struct PubCrateLibLocalStruct2<N, M, A, B> {
            crate_field1: A,
            crate_field2: B,
            crate_field3: [A; N],
            crate_field4: ([A; N], [B; M]),
            crate_field5: &mut A,
        }

        impl<N, M, A, B> Default for PubCrateLibLocalStruct2<N, M, A, B> where A: Default, B: Default {
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

        trait PubCrateLibDefinedInLocalModule2<A, B, N, M> {
            fn crate_trait_fn1(self, x: A);
            fn crate_trait_fn2(self, y: B);
            fn crate_trait_fn3(&mut self, x: A, y: B);
            fn crate_trait_fn4(self, x: [A; 0], y: [B]);
            fn crate_trait_fn5(self, x: [A; N], y: [B; M]) -> [A; 0];
            fn crate_trait_fn6(self, x: [A; N], y: [B; M]) -> [A; 0];
            fn crate_trait_fn7(self, _x: fn([A; 0]) -> B) -> Field {
                drop_var(self, N + M)
            }
        }

        impl<N, M, A, B> PubCrateLibDefinedInLocalModule2<A, B, N, M> for PubCrateLibLocalStruct2<N, M, A, B> {
            fn crate_trait_fn1(self, _x: A) { drop_var(self, ()) }
            fn crate_trait_fn2(self, _y: B) { drop_var(self, ()) }
            fn crate_trait_fn3(&mut self, _x: A, _y: B) { drop_var(self, ()) }
            fn crate_trait_fn4(self, _x: [A; 0], _y: [B]) { drop_var(self, ()) }
            fn crate_trait_fn5(self, _x: [A; N], _y: [B; M]) -> [A; 0] { drop_var(self, ()); [] }
            fn crate_trait_fn6(self, _x: [A; N], _y: [B; M]) -> [A; 0] { drop_var(self, ()); [] }
        }

        pub(crate) fn PubCrateLiblocal_fn2<A, N, B, M>(_x: (A, B), _y: [Field; N], _z: [Field; M]) -> [A; 0] {
            assert(IMPORT_GLOBAL_N_3 != IMPORT_GLOBAL_M_3);
            []
        }
    }


    use crate::local_module::{local_fn4, LocalStruct, DefinedInLocalModule1, LOCAL_GLOBAL_N, LOCAL_GLOBAL_M};

    use library::{ReExportMeFromAnotherLib1, LibLocalStruct1, LibDefinedInLocalModule1, Liblocal_fn1, IMPORT_GLOBAL_N_1, IMPORT_GLOBAL_M_1};

    // overlapping
    // use library::library2::ReExportMeFromAnotherLib1;
    use crate::library::library2::{PubLibLocalStruct3, PubLibDefinedInLocalModule3, PubLiblocal_fn3, IMPORT_GLOBAL_N_2, IMPORT_GLOBAL_M_2};

    use library3::{ReExportMeFromAnotherLib2, PubCrateLibLocalStruct2, PubCrateLibDefinedInLocalModule2, PubCrateLiblocal_fn2, IMPORT_GLOBAL_N_3, IMPORT_GLOBAL_M_3};


    fn main(_x: ReExportMeFromAnotherLib1, _y: ReExportMeFromAnotherLib2) {
        assert(LOCAL_GLOBAL_N != LOCAL_GLOBAL_M);
        assert(IMPORT_GLOBAL_N_1 != IMPORT_GLOBAL_M_1);
        assert(IMPORT_GLOBAL_N_2 != IMPORT_GLOBAL_M_2);
        assert(IMPORT_GLOBAL_N_3 != IMPORT_GLOBAL_M_3);

        let x: LocalStruct<LOCAL_GLOBAL_N, LOCAL_GLOBAL_M, Field, bool> = Default::default();
        assert(drop_var(x.trait_fn5([0; LOCAL_GLOBAL_N], [false; LOCAL_GLOBAL_M]), true));
        assert(drop_var(x.trait_fn6([0; LOCAL_GLOBAL_N], [false; LOCAL_GLOBAL_M]), true));

        let x: LibLocalStruct1<IMPORT_GLOBAL_N_1, IMPORT_GLOBAL_M_1, Field, bool> = Default::default();
        assert(drop_var(x.lib_trait_fn5([0; IMPORT_GLOBAL_N_1], [false; IMPORT_GLOBAL_M_1]), true));
        assert(drop_var(x.lib_trait_fn6([0; IMPORT_GLOBAL_N_1], [false; IMPORT_GLOBAL_M_1]), true));

        let x: PubLibLocalStruct3<IMPORT_GLOBAL_N_2, IMPORT_GLOBAL_M_2, Field, bool> = Default::default();
        assert(drop_var(x.pub_trait_fn5([0; IMPORT_GLOBAL_N_2], [false; IMPORT_GLOBAL_M_2]), true));
        assert(drop_var(x.pub_trait_fn6([0; IMPORT_GLOBAL_N_2], [false; IMPORT_GLOBAL_M_2]), true));

        let x: PubCrateLibLocalStruct2<IMPORT_GLOBAL_N_3, IMPORT_GLOBAL_M_3, Field, bool> = Default::default();
        assert(drop_var(x.crate_trait_fn5([0; IMPORT_GLOBAL_N_3], [false; IMPORT_GLOBAL_M_3]), true));
        assert(drop_var(x.crate_trait_fn6([0; IMPORT_GLOBAL_N_3], [false; IMPORT_GLOBAL_M_3]), true));

        assert(drop_var(local_fn2((0, 1), [], []), true));
        assert(drop_var(Liblocal_fn1((0, 1), [], []), true));
        assert(drop_var(PubLiblocal_fn4((0, 1), [], []), true));
        assert(drop_var(PubCrateLiblocal_fn3((0, 1), [], []), true));
    }";
    // cSpell:enable

    // NOTE: these names must be "replacement-unique", i.e.
    // replacing one in a distinct name should do nothing
    let names_to_collapse = [
        "DefinedInLocalModule1",
        "IMPORT_GLOBAL_M_1",
        "IMPORT_GLOBAL_M_2",
        "IMPORT_GLOBAL_M_3",
        "IMPORT_GLOBAL_N_1",
        "IMPORT_GLOBAL_N_2",
        "IMPORT_GLOBAL_N_3",
        "LOCAL_GLOBAL_M",
        "LOCAL_GLOBAL_N",
        "LibDefinedInLocalModule1",
        "LibLocalStruct1",
        "Liblocal_fn1", // cSpell:disable-line
        "LocalStruct",
        "PubCrateLibDefinedInLocalModule2",
        "PubCrateLibLocalStruct2",
        "PubCrateLiblocal_fn2", // cSpell:disable-line
        "PubLibDefinedInLocalModule3",
        "PubLibLocalStruct3",
        "PubLiblocal_fn3", // cSpell:disable-line
        "ReExportMeFromAnotherLib1",
        "ReExportMeFromAnotherLib2",
        "local_fn4",
        "crate_field1",
        "crate_field2",
        "crate_field3",
        "crate_field4",
        "crate_field5",
        "crate_trait_fn1",
        "crate_trait_fn2",
        "crate_trait_fn3",
        "crate_trait_fn4",
        "crate_trait_fn5",
        "crate_trait_fn6",
        "crate_trait_fn7",
        "field1",
        "field2",
        "field3",
        "field4",
        "field5",
        "lib_field1",
        "lib_field2",
        "lib_field3",
        "lib_field4",
        "lib_field5",
        "lib_trait_fn1",
        "lib_trait_fn2",
        "lib_trait_fn3",
        "lib_trait_fn4",
        "lib_trait_fn5",
        "lib_trait_fn6",
        "lib_trait_fn7",
        "pub_field1",
        "pub_field2",
        "pub_field3",
        "pub_field4",
        "pub_field5",
        "pub_trait_fn1",
        "pub_trait_fn2",
        "pub_trait_fn3",
        "pub_trait_fn4",
        "pub_trait_fn5",
        "pub_trait_fn6",
        "pub_trait_fn7",
        "trait_fn1",
        "trait_fn2",
        "trait_fn3",
        "trait_fn4",
        "trait_fn5",
        "trait_fn6",
        "trait_fn7",
    ];

    // TODO(https://github.com/noir-lang/noir/issues/4973):
    // Name resolution panic from name shadowing test
    let cases_to_skip = [
        (1, 21),
        (2, 11),
        (2, 21),
        (3, 11),
        (3, 18),
        (3, 21),
        (4, 21),
        (5, 11),
        (5, 21),
        (6, 11),
        (6, 18),
        (6, 21),
    ];
    let cases_to_skip: HashSet<(usize, usize)> = cases_to_skip.into_iter().collect();

    for (i, x) in names_to_collapse.iter().enumerate() {
        for (j, y) in names_to_collapse.iter().enumerate().filter(|(j, _)| i < *j) {
            if !cases_to_skip.contains(&(i, j)) {
                let modified_src = src.replace(x, y);
                let errors = get_program_errors(&modified_src);
                assert!(!errors.is_empty(), "Expected errors, got: {errors:?}");
            }
        }
    }
}
