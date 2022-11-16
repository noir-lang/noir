use dep::std;

fn main(input : Field) -> pub Field {
    let digest = std::hash::hash_to_field([input]);
    digest
}