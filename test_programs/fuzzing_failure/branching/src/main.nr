// This program will fail when the input is 0xdeadbeef
#[fuzz]
fn fuzz(a:u64){
    if a>=2147483648{let a=a-2147483648; if a>=1073741824{let a=a-1073741824; if a<536870912 {if a>=268435456{let a=a-268435456; if a>=134217728{let a=a-134217728; if a>=67108864{let a=a-67108864; if a>=33554432{let a=a-33554432; if a<16777216 {if a>=8388608{let a=a-8388608; if a<4194304 {if a>=2097152{let a=a-2097152; if a<1048576 {if a>=524288{let a=a-524288; if a>=262144{let a=a-262144; if a<131072 {if a>=65536{let a=a-65536; if a>=32768{let a=a-32768; if a<16384 {if a>=8192{let a=a-8192; if a>=4096{let a=a-4096; if a>=2048{let a=a-2048; if a>=1024{let a=a-1024; if a>=512{let a=a-512; if a<256 {if a>=128{let a=a-128; if a>=64{let a=a-64; if a>=32{let a=a-32; if a<16 {if a>=8{let a=a-8; if a>=4{let a=a-4; if a>=2{let a=a-2; if a>=1{let a=a-1; assert(false);}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}
}
