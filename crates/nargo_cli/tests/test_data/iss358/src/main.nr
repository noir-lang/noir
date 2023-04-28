// x=1, y =4
fn main(mut x: Field, y: Field) {
    let t = x+1; // 1+1 =2
    let r2= t*t; // 2*2 = 4
    let r1 = y*t; // 4*2=8
    constrain r2*t == r1; //4*2== 8
}
