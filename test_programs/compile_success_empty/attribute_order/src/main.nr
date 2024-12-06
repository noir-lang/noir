mod a;
mod b;

#[assert_run_order_function(18)]
fn main() {
    assert_eq(attributes_run, 19);
}

comptime mut global attributes_run: Field = 0;

pub comptime fn assert_run_order_function(_f: FunctionDefinition, order: Field) {
    assert_eq(order, attributes_run);
    attributes_run += 1;
}

pub comptime fn assert_run_order_struct(_s: StructDefinition, order: Field) {
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
