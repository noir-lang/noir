fn main(year: u32, min_age: u8) {
    if min_age == 0 {
        if year > 2028 {
            assert((((year - 1) - 2028) / 4 + 1) as i32 >= 0);
        }
    } else {
        if year > 2028 {
            // Uncomment the next line and the assertion on line 10 will pass
            // assert(year > 2028);
            assert((((year - 1) - 2028) / 4 + 1) as i32 >= 0);
        }
    }
}
