//Tests for function calling 
use dep::std;

fn f1(mut x: Field) -> Field
{
     x = x + 1;
     x = f2(x);
     x
}

fn f2(mut x: Field) -> Field{
     x=x+2;
     x
}

// Simple example
fn test0(mut a : Field) {
     a = f2(a);
     constrain(a == 3);
     
}

// Nested call
fn test1(mut a : Field){
     a = f1(a);
     constrain(a == 4);
     
}


fn test2(z : Field, t : u32 ) { 
     let a = z + t as Field;
     constrain a == 64;
     let e = pow(z,t);
     constrain e ==714924299;
}


fn pow(base : Field, exponent : Field) -> Field {
    let mut r: Field = 1 as Field;
    let b = std::to_bits(exponent, 32 as u32);
    for i in 1..33 {
        r = r*r;
        r = (b[32-i] as Field)*(r*base) + (1-b[32-i] as Field)*r;
    };
    r
}

fn test3(x: [3]u8) -> [3]u8 {
      let mut buffer: [3]u8 = [0 as u8,0 as u8, 0 as u8];
       for i in 0..3   {
       buffer[i] = x[i] ;
     };
     buffer
}

fn test_multiple(x: u32, y: u32) -> (u32, u32)
{
    (y,x)
}

fn test_multiple2() -> my_struct
{
     my_struct{a:5, b:7}
}

fn test_multiple3(x: u32, y: u32)
{
     constrain x == y;
}

struct my_struct {
     a: u32,
     b: u32,
}
struct my2 {
     aa: my_struct,
     bb: my_struct,
}

fn test_multiple4(s : my_struct) {
     constrain s.a == s.b+2;

}

fn test_multiple5(a: (u32, u32)) {
     constrain a.0 == a.1+2;
}


fn test_multiple6(a: my2, b: my_struct, c: (my2, my_struct)) {
     test_multiple4(a.aa);
     test_multiple5((b.a, b.b));
     constrain c.0.aa.a == c.1.a;
}

fn main(x : u32 , y : u32 , a : Field) {  
     let mut ss: my_struct =  my_struct{ b:x,a:x+2,};
     test_multiple4(ss);
     test_multiple5((ss.a,ss.b));
     let my: my2 = my2{
          aa: ss,
          bb: ss,
     };
     ss.a = 61;
     test_multiple6(my, ss, (my,ss));

     test0(a);
     test1(a);
     test2(x as Field,y);

     let b: [3]u8 =[0 as u8, 5 as u8, 2 as u8];
     let c = test3(b);
     constrain b == c;

     let e = test_multiple(x,y) ;
     constrain e.1==e.0+54;
     let d = test_multiple2();
     constrain d.b == d.a + 2;
     test_multiple3(y,y);
}
