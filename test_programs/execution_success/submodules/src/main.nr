use my_submodule::my_helper;

fn main(x: u1, y: u1) {
    my_helper();
    my_submodule::my_bool_or(x, y);
}

mod my_submodule {
    pub fn my_bool_or(x: u1, y: u1) {
        assert(x | y == 1);
    }

    pub fn my_helper() {}
}
