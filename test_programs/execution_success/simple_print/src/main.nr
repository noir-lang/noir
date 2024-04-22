// Simple program for testing the logging
// of single witnesses and witness arrays.
use dep::std;

fn main(x: field, y: pub field) {
    std::println(x);
    std::println([x, y]);
}
