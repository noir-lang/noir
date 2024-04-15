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
