// Re-export 
use dep::library2::ReExportMeFromAnotherLib;

global IMPORT_GLOBAL_N: Field = 2;

global IMPORT_GLOBAL_M: Field = 3;



struct LibLocalStruct<N, M, A, B> {
    field1: A,
    field2: B,
    field3: [A; N],
    field4: ([A; N], [B; M]),
    field5: &mut A,
}

trait LibDefinedInLocalModule<A, B, N, M> {
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

pub fn Liblocal_fn<A, N, B, M>(x: (A, B), y: [Field; N], z: [Field; M]) -> Option<A> {
    assert(IMPORT_GLOBAL_N != IMPORT_GLOBAL_M);
    Option::none()
}
