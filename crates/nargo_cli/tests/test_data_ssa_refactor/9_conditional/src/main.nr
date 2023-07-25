use dep::std;


fn main(a: Field, c: Field) -> pub Field{


    //Test case for short-circuit
    let mut data = [0 as Field; 4];
    let mut ba = a;
      for i in 0..4 {
        if i == a {
            data[i] = c;
            ba = ba + data[0];   
        }
    }
    
    ba
}
