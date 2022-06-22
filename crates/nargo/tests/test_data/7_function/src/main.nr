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
       buffer[i as u32] = x[i as u32] ;
     };
     buffer
}

fn main(x : u32 , y : u32 , a : Field) {  
   test0(a);
   test1(a);
   test2(x as Field,y);

     let b: [3]u8 =[0 as u8, 5 as u8, 2 as u8];
     let c = test3(b);
     constrain b == c;
}
