pub struct Context {
    pub get_field: fn() -> Field,
}

pub struct ContextWrapper {
    context: &mut Context,
}

pub struct ContextGetter<V> {
    context: &mut Context,
    getter: fn(&mut Context) -> V,
}

impl<V> ContextGetter<V> {
    pub fn get(self) -> V {
        (self.getter)(self.context)
    }
}

unconstrained fn main() {
    {
        let mut context = Context { get_field: || -> Field { 0 } };
        let wrapper_getter =
            ContextGetter { context: &mut context, getter: |context| ContextWrapper { context } };
        let _wrapper = wrapper_getter.get();
    }
}
