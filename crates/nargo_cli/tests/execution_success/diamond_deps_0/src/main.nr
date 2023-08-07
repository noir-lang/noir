use dep::dep1::call_dep1_then_dep2;
use dep::dep2::call_dep2;
use dep::dep2::RESOLVE_THIS;

fn main(x : Field, y : pub Field) -> pub Field {
    call_dep1_then_dep2(x, y) + call_dep2(x, y) + RESOLVE_THIS
}