// Map<V>
 struct B<T_C> {}

impl<T_C> B<T_C> {
    fn new(new_concrete_t_c_constructor: fn () -> T_C) -> B<T_C> {
        B { new_concrete_t_c_constructor }
    }
}