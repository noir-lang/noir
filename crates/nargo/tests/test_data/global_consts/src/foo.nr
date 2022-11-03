mod bar;

const MAGIC_NUMBER: Field = 3;

fn from_foo(x : [Field; bar::N]) {
     for i in 0..bar::N {
          constrain x[i] == bar::N;
     };
}