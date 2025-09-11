// ---
fn new_concrete_c_over_d() -> C<D> {
    let d_method_interface = get_d_method_interface();
    C::new(d_method_interface)
}
// ---
// Map<V>
struct B<T_C> {
    new_concrete_t_c_constructor: fn() -> T_C,
}

impl<T_C> B<T_C> {
    fn new(new_concrete_t_c_constructor: fn() -> T_C) -> B<T_C> {
        B { new_concrete_t_c_constructor }
    }

    fn get_t_c(self) -> T_C {
        let new_concrete_t_c_constructor = self.new_concrete_t_c_constructor;
        new_concrete_t_c_constructor()
    }
}
// ---
// PrivateSet<Note>
struct C<T_D> {
    t_d_interface: MethodInterface<T_D>,
}

impl<T_D> C<T_D> {
    fn new(t_d_interface: MethodInterface<T_D>) -> Self {
        C { t_d_interface }
    }

    fn call_method_of_t_d(self, t_d: T_D) -> Field {
        let some_method_on_t_d = self.t_d_interface.some_method_on_t_d;
        some_method_on_t_d(t_d)
    }
}
// ---
struct MethodInterface<T_D> {
    some_method_on_t_d: fn(T_D) -> Field,
}
// ---
// Note
struct D {
    d: Field,
}

fn d_method(input: D) -> Field {
    input.d * input.d
}

fn get_d_method_interface() -> MethodInterface<D> {
    MethodInterface { some_method_on_t_d: d_method }
}
// ---
fn main(input: Field) -> pub Field {
    let b: B<C<D>> = B::new(new_concrete_c_over_d);
    let c: C<D> = b.get_t_c(); // PrivateMutable<Note>
    let d: D = D { d: input }; // Note
    let output = c.call_method_of_t_d(d);

    output
}
// ---
