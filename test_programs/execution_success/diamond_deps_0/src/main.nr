use dep::dep1::call_dep1_then_dep2;
use dep::dep2::call_dep2;
use dep::dep2::RESOLVE_THIS;

fn main(x: field, y: pub field) -> pub field {
    call_dep1_then_dep2(x, y) + call_dep2(x, y) + RESOLVE_THIS
}
