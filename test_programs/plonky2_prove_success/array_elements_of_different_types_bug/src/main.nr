// regression test for the bug that causes this program to produce this error:
// "PLONKY2 backend does not support array elements of different types (Array(Integer(32), 16) and Integer(32)); array=Array { array: [Id(5), Id(6), Id(7), Id(8), Id(9), Id(10), Id(11), Id(2), Id(2), Id(2), Id(2), Id(2), Id(2), Id(2), Id(2), Id(2)], typ: Array([Numeric(Unsigned { bit_size: 32 })], 16) }; instr: \"ArrayGet { array: Id(12), index: Id(0) }\""

fn main(x: u32, y: pub u32) {
    let mut some_array: [u32; 0x10] = [0; 0x10];
    some_array[0] = 5;
    some_array[1] = 10;
    some_array[2] = 3;
    some_array[3] = 7;
    some_array[4] = 8;
    some_array[5] = 1;
    some_array[6] = 2;
    some_array[7] = 0;
    some_array[8] = 0;
    some_array[9] = 0;
    some_array[10] = 0;
    some_array[11] = 0;
    assert(some_array[x] == y);
}
