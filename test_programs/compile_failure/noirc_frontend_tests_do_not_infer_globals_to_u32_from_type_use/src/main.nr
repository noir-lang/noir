
        global ARRAY_LEN = 3;
        global STR_LEN: _ = 2;
        global FMT_STR_LEN = 2;

        fn main() {
            let _a: [u32; ARRAY_LEN] = [1, 2, 3];
            let _b: str<STR_LEN> = "hi";
            let _c: fmtstr<FMT_STR_LEN, _> = f"hi";
        }
    