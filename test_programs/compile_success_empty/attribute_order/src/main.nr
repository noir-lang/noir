mod a;
mod b;

#[assert_run_order_function(15)]
fn main() {
    assert_eq(attributes_run, 16);
}

comptime mut global attributes_run: Field = 0;

pub comptime fn assert_run_order_function(_f: FunctionDefinition, order: Field) {
    let a = attributes_run;
    println(f"{order} == {a}");
    assert_eq(order, attributes_run);
    attributes_run += 1;
}

pub comptime fn assert_run_order_struct(_s: StructDefinition, order: Field) {
    let a = attributes_run;
    println(f"{order} == {a}");
    assert_eq(order, attributes_run);
    attributes_run += 1;
}

pub comptime fn assert_run_order_trait(_t: TraitDefinition, order: Field) {
    let a = attributes_run;
    println(f"{order} == {a}");
    assert_eq(order, attributes_run);
    attributes_run += 1;
}

pub comptime fn assert_run_order_module(_m: Module, order: Field) {
    let a = attributes_run;
    println(f"{order} == {a}");
    assert_eq(order, attributes_run);
    attributes_run += 1;
}
