use dep::std;

fn call_dep1_then_dep2(x : Field, y : Field) {
    std::assert_eq(x, y);
}
