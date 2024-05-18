use dep::protocol_types::address::AztecAddress;

// TODO: this is awful but since we can't have a fn that maps [Field; N] -> [u8; 480 + N * 32]
// (where N is the note pre-image size and 480 + N * 32 is the encryption output size)
// The fns for LensForEncryptedLog are never used, it's just to tell the compiler what the lens are

// The to_bytes fn for ToBytesForUnencryptedLog is used to allow us to hash some generic T

// I could have omitted N from the trait, but wanted to keep it strictly for field arrs
// TODO(1139): Once we enc inside the circuit, we will no longer need the oracle to return
// anything, so we can remove this trait
trait LensForEncryptedLog<N, M> {
    // N = note preimage input in fields
    // M = encryption output len in bytes (= 480 + N * 32)
    fn output_fields(self: [Field; N]) -> [Field; N];
    fn output_bytes(self: [Field; N]) -> [u8; M];
}

impl LensForEncryptedLog<1, 512> for [Field; 1] {
    fn output_fields(self) -> [Field; 1] {[self[0]; 1]}
    fn output_bytes(self) -> [u8; 512] {[self[0] as u8; 512]}
}
impl LensForEncryptedLog<2, 544> for [Field; 2] {
    fn output_fields(self) -> [Field; 2] {[self[0]; 2]}
    fn output_bytes(self) -> [u8; 544] {[self[0] as u8; 544]}
}
impl LensForEncryptedLog<3, 576> for [Field; 3] {
    fn output_fields(self) -> [Field; 3] {[self[0]; 3]}
    fn output_bytes(self) -> [u8; 576] {[self[0] as u8; 576]}
}
impl LensForEncryptedLog<4, 608> for [Field; 4] {
    fn output_fields(self) -> [Field; 4] {[self[0]; 4]}
    fn output_bytes(self) -> [u8; 608] {[self[0] as u8; 608]}

}
impl LensForEncryptedLog<5, 640> for [Field; 5] {
    fn output_fields(self) -> [Field; 5] {[self[0]; 5]}
    fn output_bytes(self) -> [u8; 640] {[self[0] as u8; 640]}
}
impl LensForEncryptedLog<6, 672> for [Field; 6] {
    fn output_fields(self) -> [Field; 6] {[self[0]; 6]}
    fn output_bytes(self) -> [u8; 672] {[self[0] as u8; 672]}

}

// This trait defines the length of the inputs in bytes to
// the unencrypted log hash fn, where the log can be any type T
// as long as the ACVM can convert to fields.
trait ToBytesForUnencryptedLog<N, M> {
    // N = preimage input in bytes (32 * num fields or chars)
    // M = full log input in bytes ( = N + 40 = N + 32 for addr, + 4 for selector, + 4 for len)
    fn to_be_bytes_arr(self) -> [u8; N];
    fn output_bytes(self) -> [u8; M];
}

impl ToBytesForUnencryptedLog<32, 72> for Field {
    fn to_be_bytes_arr(self) -> [u8; 32] {
        self.to_be_bytes(32).as_array()
    }
    fn output_bytes(self) -> [u8; 72] {[self as u8; 72]}
}

impl ToBytesForUnencryptedLog<32, 72> for AztecAddress {
    fn to_be_bytes_arr(self) -> [u8; 32] {
        self.to_field().to_be_bytes(32).as_array()
    }
    fn output_bytes(self) -> [u8; 72] {[self.to_field() as u8; 72]}
}

fn arr_to_be_bytes_arr<N, L>(fields: [Field; L]) -> [u8; N] {
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
fn str_to_be_bytes_arr<N, L>(string: str<L>) -> [u8; N] {
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

impl ToBytesForUnencryptedLog<32, 72> for [Field; 1] {
    fn to_be_bytes_arr(self) -> [u8; 32] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 72] {
        [self[0] as u8; 72]
    }
}

impl ToBytesForUnencryptedLog<64, 104> for [Field; 2] {
    fn to_be_bytes_arr(self) -> [u8; 64] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 104] {
        [self[0] as u8; 104]
    }
}

impl ToBytesForUnencryptedLog<96, 136> for [Field; 3] {
    fn to_be_bytes_arr(self) -> [u8; 96] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 136] {
        [self[0] as u8; 136]
    }
}

impl ToBytesForUnencryptedLog<128, 168> for [Field; 4] {
    fn to_be_bytes_arr(self) -> [u8; 128] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 168] {
        [self[0] as u8; 168]
    }
}

impl ToBytesForUnencryptedLog<160, 200> for [Field; 5] {
    fn to_be_bytes_arr(self) -> [u8; 160] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 200] {
        [self[0] as u8; 200]
    }
}

impl ToBytesForUnencryptedLog<192, 232> for [Field; 6] {
    fn to_be_bytes_arr(self) -> [u8; 192] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 232] {
        [self[0] as u8; 232]
    }
}

impl ToBytesForUnencryptedLog<224, 264> for [Field; 7] {
    fn to_be_bytes_arr(self) -> [u8; 224] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 264] {
        [self[0] as u8; 264]
    }
}

impl ToBytesForUnencryptedLog<256, 296> for [Field; 8] {
    fn to_be_bytes_arr(self) -> [u8; 256] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 296] {
        [self[0] as u8; 296]
    }
}

impl ToBytesForUnencryptedLog<288, 328> for [Field; 9] {
    fn to_be_bytes_arr(self) -> [u8; 288] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 328] {
        [self[0] as u8; 328]
    }
}

impl ToBytesForUnencryptedLog<320, 360> for [Field; 10] {
    fn to_be_bytes_arr(self) -> [u8; 320] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 360] {
        [self[0] as u8; 360]
    }
}

impl ToBytesForUnencryptedLog<352, 392> for [Field; 11] {
    fn to_be_bytes_arr(self) -> [u8; 352] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 392] {
        [self[0] as u8; 392]
    }
}

impl ToBytesForUnencryptedLog<384, 424> for [Field; 12] {
    fn to_be_bytes_arr(self) -> [u8; 384] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 424] {
        [self[0] as u8; 424]
    }
}

impl ToBytesForUnencryptedLog<416, 456> for [Field; 13] {
    fn to_be_bytes_arr(self) -> [u8; 416] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 456] {
        [self[0] as u8; 456]
    }
}

impl ToBytesForUnencryptedLog<448, 488> for [Field; 14] {
    fn to_be_bytes_arr(self) -> [u8; 448] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 488] {
        [self[0] as u8; 488]
    }
}

impl ToBytesForUnencryptedLog<480, 520> for [Field; 15] {
    fn to_be_bytes_arr(self) -> [u8; 480] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 520] {
        [self[0] as u8; 520]
    }
}

impl ToBytesForUnencryptedLog<512, 552> for [Field; 16] {
    fn to_be_bytes_arr(self) -> [u8; 512] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 552] {
        [self[0] as u8; 552]
    }
}

impl ToBytesForUnencryptedLog<544, 584> for [Field; 17] {
    fn to_be_bytes_arr(self) -> [u8; 544] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 584] {
        [self[0] as u8; 584]
    }
}

impl ToBytesForUnencryptedLog<576, 616> for [Field; 18] {
    fn to_be_bytes_arr(self) -> [u8; 576] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 616] {
        [self[0] as u8; 616]
    }
}

impl ToBytesForUnencryptedLog<608, 648> for [Field; 19] {
    fn to_be_bytes_arr(self) -> [u8; 608] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 648] {
        [self[0] as u8; 648]
    }
}

impl ToBytesForUnencryptedLog<640, 680> for [Field; 20] {
    fn to_be_bytes_arr(self) -> [u8; 640] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 680] {
        [self[0] as u8; 680]
    }
}

impl ToBytesForUnencryptedLog<672, 712> for [Field; 21] {
    fn to_be_bytes_arr(self) -> [u8; 672] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 712] {
        [self[0] as u8; 712]
    }
}

impl ToBytesForUnencryptedLog<704, 744> for [Field; 22] {
    fn to_be_bytes_arr(self) -> [u8; 704] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 744] {
        [self[0] as u8; 744]
    }
}

impl ToBytesForUnencryptedLog<736, 776> for [Field; 23] {
    fn to_be_bytes_arr(self) -> [u8; 736] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 776] {
        [self[0] as u8; 776]
    }
}

impl ToBytesForUnencryptedLog<768, 808> for [Field; 24] {
    fn to_be_bytes_arr(self) -> [u8; 768] {
        arr_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; 808] {
        [self[0] as u8; 808]
    }
}

impl<N,M,L> ToBytesForUnencryptedLog<N, M> for str<L> where [Field; L]: ToBytesForUnencryptedLog<N,M> {
    fn to_be_bytes_arr(self) -> [u8; N] {
        str_to_be_bytes_arr(self)
    }
    fn output_bytes(self) -> [u8; M] {
        [0; M]
    }
}
