use dep::std;

fn array_sum(a: []u32) -> u32 {
    let mut sum=0;
    for i in 0..std::array_len(a) {
        sum = sum +a[i] ;
    };
    sum
}


fn main(x : [5]u32, 
    ){
    let s = array_sum(x);
    constrain s == 532;
}
