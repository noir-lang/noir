//Tests for function calling 
fn f1(x: Field) -> Field
{
     x = x +1;
     x = f2(x);
     x
}

fn f2(x: Field) -> Field{
     x=x+2;
     x
}

// Simple example
fn test0(a : Field) {
     a = f2(a);
     constrain(a == 3);
}

// Nested call
fn test1(a : Field) {
     a = f1(a);
     constrain(a == 4);
}


fn test2(z : u32, t : u32 ) { 
   let a = z+t;
   constrain a == 64;

}

fn main(x : u32 , y : u32 , a : Field) {  
   test0(a);
   test1(a);
   test2(x,y);
}

