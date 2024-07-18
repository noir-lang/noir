use dep::protocol_types::address::AztecAddress;

// TODO: this is awful but since we can't have a fn that maps [Field; N] -> [u8; 416 + N * 32]
// (where N is the note pre-image size and 416 + N * 32 is the encryption output size)
// The fns for LensForEncryptedLog are never used, it's just to tell the compiler what the lens are

// The to_bytes fn for ToBytesForUnencryptedLog is used to allow us to hash some generic T

// I could have omitted N from the trait, but wanted to keep it strictly for field arrs
// TODO(1139): Once we enc inside the circuit, we will no longer need the oracle to return
// anything, so we can remove this trait
trait LensForEncryptedLog<let N: u32, let M: u32> {
    // N = note preimage input in fields
    // M = encryption output len in bytes (= 416 + N * 32)
    fn output_fields(self: [Field; N]) -> [Field; N];
    fn output_bytes(self: [Field; N]) -> [u8; M];
}

impl LensForEncryptedLog<1, 448> for [Field; 1] {
    fn output_fields(self) -> [Field; 1] {[self[0]; 1]}
    fn output_bytes(self) -> [u8; 448] {[self[0] as u8; 448]}
}
impl LensForEncryptedLog<2, 480> for [Field; 2] {
    fn output_fields(self) -> [Field; 2] {[self[0]; 2]}
    fn output_bytes(self) -> [u8; 480] {[self[0] as u8; 480]}
}
impl LensForEncryptedLog<3, 512> for [Field; 3] {
    fn output_fields(self) -> [Field; 3] {[self[0]; 3]}
    fn output_bytes(self) -> [u8; 512] {[self[0] as u8; 512]}
}
impl LensForEncryptedLog<4, 544> for [Field; 4] {
    fn output_fields(self) -> [Field; 4] {[self[0]; 4]}
    fn output_bytes(self) -> [u8; 544] {[self[0] as u8; 544]}
}
impl LensForEncryptedLog<5, 576> for [Field; 5] {
    fn output_fields(self) -> [Field; 5] {[self[0]; 5]}
    fn output_bytes(self) -> [u8; 576] {[self[0] as u8; 576]}
}
impl LensForEncryptedLog<6, 608> for [Field; 6] {
    fn output_fields(self) -> [Field; 6] {[self[0]; 6]}
    fn output_bytes(self) -> [u8; 608] {[self[0] as u8; 608]}
}
impl LensForEncryptedLog<7, 640> for [Field; 7] {
    fn output_fields(self) -> [Field; 7] {[self[0]; 7]}
    fn output_bytes(self) -> [u8; 640] {[self[0] as u8; 640]}
}
impl LensForEncryptedLog<8, 672> for [Field; 8] {
    fn output_fields(self) -> [Field; 8] {[self[0]; 8]}
    fn output_bytes(self) -> [u8; 672] {[self[0] as u8; 672]}
}
impl LensForEncryptedLog<9, 704> for [Field; 9] {
    fn output_fields(self) -> [Field; 9] {[self[0]; 9]}
    fn output_bytes(self) -> [u8; 704] {[self[0] as u8; 704]}
}
impl LensForEncryptedLog<10, 736> for [Field; 10] {
    fn output_fields(self) -> [Field; 10] {[self[0]; 10]}
    fn output_bytes(self) -> [u8; 736] {[self[0] as u8; 736]}
}
impl LensForEncryptedLog<11, 768> for [Field; 11] {
    fn output_fields(self) -> [Field; 11] {[self[0]; 11]}
    fn output_bytes(self) -> [u8; 768] {[self[0] as u8; 768]}
}
impl LensForEncryptedLog<12, 800> for [Field; 12] {
    fn output_fields(self) -> [Field; 12] {[self[0]; 12]}
    fn output_bytes(self) -> [u8; 800] {[self[0] as u8; 800]}
}

trait LensForEncryptedEvent<let N: u32, let M: u32> {
    // N = event preimage input in bytes
    // M = encryption output len in bytes (= 416 + M)
    fn output(self: [u8; N]) -> [u8; M];
}

impl LensForEncryptedEvent<96, 448> for [u8; 96] {
    fn output(self) -> [u8; 448] {[self[0] as u8; 448]}
}
impl LensForEncryptedEvent<128, 480> for [u8; 128] {
    fn output(self) -> [u8; 480] {[self[0] as u8; 480]}
}
impl LensForEncryptedEvent<160, 512> for [u8; 160] {
    fn output(self) -> [u8; 512] {[self[0] as u8; 512]}
}
impl LensForEncryptedEvent<192, 544> for [u8; 192] {
    fn output(self) -> [u8; 544] {[self[0] as u8; 544]}
}
impl LensForEncryptedEvent<224, 576> for [u8; 224] {
    fn output(self) -> [u8; 576] {[self[0] as u8; 576]}
}
impl LensForEncryptedEvent<256, 608> for [u8; 256] {
    fn output(self) -> [u8; 608] {[self[0] as u8; 608]}
}

// This trait defines the length of the inputs in bytes to
// the unencrypted log hash fn, where the log can be any type T
// as long as the ACVM can convert to fields.
trait ToBytesForUnencryptedLog<let N: u32, let M: u32> {
    // N = preimage input in bytes (32 * num fields or chars)
    // M = full log input in bytes ( = N + 40 = N + 32 for addr, + 4 for selector, + 4 for len)
    fn to_be_bytes_arr(self) -> [u8; N];
    fn output_bytes(self) -> [u8; M];
}

impl ToBytesForUnencryptedLog<32, 68> for Field {
    fn to_be_bytes_arr(self) -> [u8; 32] {
        self.to_be_bytes(32).as_array()
    }
    fn output_bytes(self) -> [u8; 68] {[self as u8; 68]}
}

impl ToBytesForUnencryptedLog<32, 68> for AztecAddress {
    fn to_be_bytes_arr(self) -> [u8; 32] {
        self.to_field().to_be_bytes(32).as_array()
    }
    fn output_bytes(self) -> [u8; 68] {[self.to_field() as u8; 68]}
}

fn arr_to_be_bytes_arr<let N: u32, let L: u32>(fields: [Field; L]) -> [u8; N] {
    let mut bytes: [u8] = &[];
    for i in 0..L {
        // Note that bytes.append() results in bound error
        let to_add = fields[i].to_be_bytes(32);
        for j in 0..32 {
            bytes = bytes.push_back(to_add[j]);
        }
    }
    bytes.as_array()
}

// each character of a string is converted into a byte
// then an ACVM field via the oracle => we recreate here
fn str_to_be_bytes_arr<let N: u32, let L: u32>(string: str<L>) -> [u8; N] {
    let chars_bytes = string.as_bytes();
    let mut bytes: [u8] = &[];
    for i in 0..L {
        let to_add = (chars_bytes[i] as Field).to_be_bytes(32);
        for j in 0..32 {
            bytes = bytes.push_back(to_add[j]);
        }
    }
    bytes.as_array()
}

impl ToBytesForUnencryptedLog<32, 68> for [Field; 1] {
    fn to_be_bytes_arr(self) -> [u8; 32] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 68] {
        [self[0] as u8; 68]
    }
}

impl ToBytesForUnencryptedLog<64, 100> for [Field; 2] {
    fn to_be_bytes_arr(self) -> [u8; 64] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 100] {
        [self[0] as u8; 100]
    }
}

impl ToBytesForUnencryptedLog<96, 132> for [Field; 3] {
    fn to_be_bytes_arr(self) -> [u8; 96] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 132] {
        [self[0] as u8; 132]
    }
}

impl ToBytesForUnencryptedLog<128, 164> for [Field; 4] {
    fn to_be_bytes_arr(self) -> [u8; 128] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 164] {
        [self[0] as u8; 164]
    }
}

impl ToBytesForUnencryptedLog<160, 196> for [Field; 5] {
    fn to_be_bytes_arr(self) -> [u8; 160] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 196] {
        [self[0] as u8; 196]
    }
}

impl ToBytesForUnencryptedLog<192, 228> for [Field; 6] {
    fn to_be_bytes_arr(self) -> [u8; 192] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 228] {
        [self[0] as u8; 228]
    }
}

impl ToBytesForUnencryptedLog<224, 260> for [Field; 7] {
    fn to_be_bytes_arr(self) -> [u8; 224] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 260] {
        [self[0] as u8; 260]
    }
}

impl ToBytesForUnencryptedLog<256, 292> for [Field; 8] {
    fn to_be_bytes_arr(self) -> [u8; 256] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 292] {
        [self[0] as u8; 292]
    }
}

impl ToBytesForUnencryptedLog<288, 324> for [Field; 9] {
    fn to_be_bytes_arr(self) -> [u8; 288] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 324] {
        [self[0] as u8; 324]
    }
}

impl ToBytesForUnencryptedLog<320, 356> for [Field; 10] {
    fn to_be_bytes_arr(self) -> [u8; 320] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 356] {
        [self[0] as u8; 356]
    }
}

impl ToBytesForUnencryptedLog<352, 388> for [Field; 11] {
    fn to_be_bytes_arr(self) -> [u8; 352] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 388] {
        [self[0] as u8; 388]
    }
}

impl ToBytesForUnencryptedLog<384, 420> for [Field; 12] {
    fn to_be_bytes_arr(self) -> [u8; 384] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 420] {
        [self[0] as u8; 420]
    }
}

impl ToBytesForUnencryptedLog<416, 452> for [Field; 13] {
    fn to_be_bytes_arr(self) -> [u8; 416] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 452] {
        [self[0] as u8; 452]
    }
}

impl ToBytesForUnencryptedLog<448, 484> for [Field; 14] {
    fn to_be_bytes_arr(self) -> [u8; 448] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 484] {
        [self[0] as u8; 484]
    }
}

impl ToBytesForUnencryptedLog<480, 516> for [Field; 15] {
    fn to_be_bytes_arr(self) -> [u8; 480] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 516] {
        [self[0] as u8; 516]
    }
}

impl ToBytesForUnencryptedLog<512, 548> for [Field; 16] {
    fn to_be_bytes_arr(self) -> [u8; 512] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 548] {
        [self[0] as u8; 548]
    }
}

impl ToBytesForUnencryptedLog<544, 580> for [Field; 17] {
    fn to_be_bytes_arr(self) -> [u8; 544] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 580] {
        [self[0] as u8; 580]
    }
}

impl ToBytesForUnencryptedLog<576, 612> for [Field; 18] {
    fn to_be_bytes_arr(self) -> [u8; 576] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 612] {
        [self[0] as u8; 612]
    }
}

impl ToBytesForUnencryptedLog<608, 644> for [Field; 19] {
    fn to_be_bytes_arr(self) -> [u8; 608] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 644] {
        [self[0] as u8; 644]
    }
}

impl ToBytesForUnencryptedLog<640, 676> for [Field; 20] {
    fn to_be_bytes_arr(self) -> [u8; 640] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 676] {
        [self[0] as u8; 676]
    }
}

impl ToBytesForUnencryptedLog<672, 708> for [Field; 21] {
    fn to_be_bytes_arr(self) -> [u8; 672] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 708] {
        [self[0] as u8; 708]
    }
}

impl ToBytesForUnencryptedLog<704, 740> for [Field; 22] {
    fn to_be_bytes_arr(self) -> [u8; 704] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 740] {
        [self[0] as u8; 740]
    }
}

impl ToBytesForUnencryptedLog<736, 772> for [Field; 23] {
    fn to_be_bytes_arr(self) -> [u8; 736] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 772] {
        [self[0] as u8; 772]
    }
}

impl ToBytesForUnencryptedLog<768, 804> for [Field; 24] {
    fn to_be_bytes_arr(self) -> [u8; 768] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 804] {
        [self[0] as u8; 804]
    }
}

impl<let N: u32, let M: u32, let L: u32> ToBytesForUnencryptedLog<N, M> for str<L> where [Field; L]: ToBytesForUnencryptedLog<N,M> {
    fn to_be_bytes_arr(self) -> [u8; N] {
        str_to_be_bytes_arr(self)
    }
    fn output_bytes(_self: Self) -> [u8; M] {
        [0; M]
    }
}
