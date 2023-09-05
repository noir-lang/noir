use dep::std;

trait Default {
    fn default(x: Field, y: Field) -> Field;
}


impl Default for main {
    fn default(x: Field, y: Field) -> Field {
        x + y
    }
}

fn main(x: Field, y: Field) {
}
