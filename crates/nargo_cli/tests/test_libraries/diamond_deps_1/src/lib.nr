use dep::dep2::call_dep2;

fn call_dep1_then_dep2(x : Field, y : Field) -> Field {
    call_dep2(x, y)
}
