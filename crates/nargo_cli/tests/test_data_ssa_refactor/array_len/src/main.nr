use dep::std;

fn len_plus_1<T>(array: [T]) -> Field {
    array.len() + 1
}

fn add_lens<T>(a: [T], b: [Field]) -> Field {
    a.len() + b.len()
}

fn nested_call(b: [Field]) -> Field {
    len_plus_1(b)
}

fn bar(mut a: [Field]) -> [Field] {
   // a.push_back(a[1]);
    a
}

fn foo(mut a: [Field]) -> Field{
    let mut d = -(a[0] as i32);
    d = -5;
    //let mut b=a;
    let mut sum = 0;
let mut b:[Field] = [0,0];//  hum..
let k = b.push_back(3);
    if a[0]==0 {
        a.push_back(10);
    } else {

    }
    a = bar(a);
   // let c=[0;a.len()]; marche po
    for i in 0..a.len()-1 {
      //  b[i] = a[i];
        b.push_back(a[i]);
    }
   // for i in b {
   //     sum = sum+i;
   // } 
   // sum
   a[2]
}

fn main(len3: [u8; 3], len4: [Field; 4]) {

    let toto = foo(len4);
    std::println(toto);
    assert(len_plus_1(len3) == 4);
    assert(len_plus_1(len4) == 5);
    assert(add_lens(len3, len4) == 7);
    assert(nested_call(len4) == 5);

    // std::array::len returns a comptime value
    assert(len4[len3.len()] == 4);
}
