// Simple program for testing the logging
// of single witnesses and witness arrays.

use dep::std;

fn main(x : Field, y : pub Field) {
    std::println(x);
    std::println([x, y]);
}
