use mysubmodule::my_helper;

fn main(x: u1, y: u1) {
    my_helper();
    mysubmodule::my_bool_or(x, y);
}

mod mysubmodule {
    use dep::std;

    fn my_bool_or(x: u1, y: u1) {
        constrain x | y == 1;
    }

    fn my_helper() {}
}
    
