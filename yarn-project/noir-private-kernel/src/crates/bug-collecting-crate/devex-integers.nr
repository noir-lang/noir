Could allow for u128 , but then just make it really slow
so u128 addition would be slow because internally the compiler will be 
decomposing them.
Probably possible to just have a u128 struct or something of the kind