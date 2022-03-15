//Basic tests for arrays 

fn main(x : [5]u32 , y : [5]u32 , z : u32, t : u32) {  
   let c = 2301 as u32;
   //t= t+x[0]-x[0];
   z=y[4];
   //Test 1: 
   for i in 0..5 {
      c = z*z*y[i];
      z  = z - c;
   };
   constrain (z==0);    //y[4]=0, so c and z are always 0

   //Test 2:
   c = 2301 as u32;
   for i in 0..5 {
      c = t+2;
      c = z*z*x[i];
      z  = z + x[i]*y[i] - c;
   };
   constrain (z==3814912846);

   //Test 3:
   c = 2300001 as u32;
   z=y[4];
   for i in 0..5 {
      z  = z + x[i]*y[i];
       for _i in 0..3 {
           c = i as u32 - 2 as u32;
           z=c*z;
       };
   };
   constrain (z==41472);

   //Test 4:
   z=y[4];
   for i in 0..3 {
      z  = z + x[i]*y[i];
      for j in 0..2 {
         z = z + x[i+j]-y[i+j];
      };
   };
   constrain (z ==11539);
}