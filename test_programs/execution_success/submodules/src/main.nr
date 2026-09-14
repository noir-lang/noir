use my_submodule::my_helper;

fn main(x: bool, y: bool) {
    my_helper();
    my_submodule::my_bool_or(x, y);
}

mod my_submodule {
    pub fn my_bool_or(x: bool, y: bool) {
        assert(x | y);
    }

    pub fn my_helper() {}
}
