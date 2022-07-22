use mysubmodule::my_helper;

fn main(x : Field, y : Field) {
    my_helper();
    mysubmodule::my_bool_or(x, y);
}

mod mysubmodule {
    use dep::std;

    fn my_bool_or(x: Field, y: Field) {
        constrain std::bool_or(x, y) == 1;
    }

    fn my_helper() {}
}
    
