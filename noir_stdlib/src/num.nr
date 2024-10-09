// docs:start:int-constants
pub trait IntConstants {
    let BITS: Field;
    let MAX: Field;
    let MIN: Field;
}
// docs:end:int-constants

impl IntConstants for i8 {
    let BITS = 8;
    let MAX = 127;
    let MIN = -128;
}

impl IntConstants for i16 {
    let BITS = 16;
    let MAX = 32767;
    let MIN = -32768;
}

impl IntConstants for i32 {
    let BITS = 32;
    let MAX = 2147483647;
    let MIN = -2147483648;
}

impl IntConstants for u8 {
    let BITS = 8;
    let MAX = 255;
    let MIN = 0;
}

impl IntConstants for u16 {
    let BITS = 16;
    let MAX = 65535;
    let MIN = 0;
}

impl IntConstants for u32 {
    let BITS = 32;
    let MAX = 4294967295;
    let MIN = 0;
}
