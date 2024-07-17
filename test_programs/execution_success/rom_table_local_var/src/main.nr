fn main(x: [u8; 44]) {
    // for _ in 0..1 {
    //     let r: [u8; 32] = base64_decode_without_struct(x);
    //     // let r: [u8; 32] = base64_decode_with_struct(x);
    //     println(f"{r}");
    // }

    let _: [u8; 2] = base64_decode_without_struct(x);
    // let _: [u8; 2] = base64_decode_with_struct(x);
    // println(f"{r}");
}

global BASE64_DECODE_BE: [Field; 128] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,// 0-9
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,// 10-19
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,// 20-29
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,// 30-39
        0, 0, 0,// 40-42
        62,// 43
        0, 0, 0,// 44-46
        63,// 47
        52, 53, 54, 55, 56, 57, 58, 59, 60, 61,// 48-57
        0, 0, 0, 0, 0, 0, 0,// 58-64
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,// 65-90 (A-Z)
        0, 0, 0, 0, 0, 0,// 91-96
        26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,// 97-122 (a-z)
        0, 0, 0, 0, 0// 123-127
    ];

fn base64_decode_without_struct<let InputElements: u64, let OutputBytes: u64>(input: [u8; InputElements]) -> [u8; OutputBytes] {
    // let BASE64_DECODE_BE: [Field; 128] = [
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0,// 0-9
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0,// 10-19
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0,// 20-29
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0,// 30-39
    //     0, 0, 0,// 40-42
    //     62,// 43
    //     0, 0, 0,// 44-46
    //     63,// 47
    //     52, 53, 54, 55, 56, 57, 58, 59, 60, 61,// 48-57
    //     0, 0, 0, 0, 0, 0, 0,// 58-64
    //     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,// 65-90 (A-Z)
    //     0, 0, 0, 0, 0, 0,// 91-96
    //     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,// 97-122 (a-z)
    //     0, 0, 0, 0, 0// 123-127
    // ];

    // 240 bits fits 40 6-bit chunks and 30 8-bit chunks
    // we pack 40 base64 values into a field element and convert into 30 bytes
    // TODO: once we support arithmetic ops on generics, derive OutputBytes from InputBytes
    let mut result: [u8; OutputBytes] = [0; OutputBytes];


    let mut slice: Field = 0;
    for j in 0..InputElements {
        slice += BASE64_DECODE_BE[input[j]];
    }

    let _: [u8; 30] = slice.to_be_bytes(30).as_array();

    result
}

// #### METHOD 2. When lookup table is in a struct, lookups are much cheaper and ROM table is used in backend
struct Lookup {
    table: [Field; 128]
}
impl Lookup {
    fn new() -> Self {
        Lookup {
            table: BASE64_DECODE_BE
        }
    }

    fn get(self, idx: Field) -> Field {
        self.table[idx]
    }
}

fn base64_decode_with_struct<let InputElements: u64, let OutputBytes: u64>(input: [u8; InputElements]) -> [u8; OutputBytes] {
    // 240 bits fits 40 6-bit chunks and 30 8-bit chunks
    // we pack 40 base64 values into a field element and convert into 30 bytes
    // TODO: once we support arithmetic ops on generics, derive OutputBytes from InputBytes
    let mut BASE64_DECODE_BE = Lookup::new();

    // 240 bits fits 40 6-bit chunks and 30 8-bit chunks
    // we pack 40 base64 values into a field element and convert into 30 bytes
    // TODO: once we support arithmetic ops on generics, derive OutputBytes from InputBytes
    let mut result: [u8; OutputBytes] = [0; OutputBytes];

    let mut slice: Field = 0;
    for j in 0..InputElements {
        slice += BASE64_DECODE_BE.get(input[j] as Field);
    }
    // for _ in base64_elements_in_final_chunk..BASE64_ELEMENTS_PER_CHUNK {
    //     slice *= 64;
    // }

    // TODO: check is it cheaper to use a constant value in `to_be_bytes` or can we use `bytes_in_final_chunk`?
    let _: [u8; 30] = slice.to_be_bytes(30).as_array();
    // let num_bytes_in_final_chunk = OutputBytes - ((num_chunks - 1) * BYTES_PER_CHUNK);
    // for i in 0..num_bytes_in_final_chunk {
    //     result[(num_chunks - 1) * BYTES_PER_CHUNK + i] = slice_bytes[i];
    // }
    result
}
