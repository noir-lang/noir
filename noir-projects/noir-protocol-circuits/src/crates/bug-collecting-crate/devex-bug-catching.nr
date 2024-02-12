When an array is passed in, it is copied in and so if it is written to and not read from, 
we should get an error saying:

array written to, but never read.

This may prompt the developer to change fn foo(x : [Field ; 2]) to fn foo(x : &mut [Field ; 2]) or to return it 
fn foo(x : [Field ; 2]) -> [Field ;2] 