
      fn do_closure(x: Field) -> Field {
        let y = x;
        let ret_capture = || {
          y
        };
        ret_capture()
      }

      fn main(x: Field) {
          assert(do_closure(x) == 100);
      }

      