
    fn main(mut x: u32, y: pub u32) {
        let x_ref = &mut x;
        if x == 5  {
            // Safety: test context
            unsafe {
                mut_ref_input(x_ref, y);        
            }
        }

        assert(x == 10);
    }

    unconstrained fn mut_ref_input(x: &mut u32, y: u32) {
        *x = y;
    }
    