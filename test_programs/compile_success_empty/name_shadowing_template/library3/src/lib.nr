
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
