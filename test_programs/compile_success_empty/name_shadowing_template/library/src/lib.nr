// Re-export 
use dep::library2::ReExportMeFromAnotherLib;

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
