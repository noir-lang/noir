mod a;
mod b;

#[assert_run_order_function(18)]
fn main() {
    // TODO: This implies an ordering constraint but we don't actually guarantee ordering of comptime blocks within functions.
    assert_eq(comptime { attributes_run }, 19);
}

comptime mut global attributes_run: Field = 0;

pub comptime fn assert_run_order_function(_f: FunctionDefinition, order: Field) {
    assert_eq(order, attributes_run);
    attributes_run += 1;
}

pub comptime fn assert_run_order_struct(_s: TypeDefinition, order: Field) {
    assert_eq(order, attributes_run);
    attributes_run += 1;
}

pub comptime fn assert_run_order_trait(_t: TraitDefinition, order: Field) {
    assert_eq(order, attributes_run);
    attributes_run += 1;
}

pub comptime fn assert_run_order_module(_m: Module, order: Field) {
    assert_eq(order, attributes_run);
    attributes_run += 1;
}
