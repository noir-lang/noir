mod bar;

global N: Field = 5;
global MAGIC_NUMBER: Field = 3;
global TYPE_INFERRED = 42;

fn from_foo(x : [Field; bar::N]) {
     for i in 0..bar::N {
          constrain x[i] == bar::N;
     };
}