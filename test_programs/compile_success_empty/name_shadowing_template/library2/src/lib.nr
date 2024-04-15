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
