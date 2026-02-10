// Checks that fmtstr and string compute lengths consistently, by
// measuring the byte length of the UTF-8 encoding.
fn main() {
    let _s: str<1> = "a";
    let _f: fmtstr<1, _> = f"a";

    let _s2: str<2> = "é";
    let _f2: fmtstr<2, _> = f"é";

    let _s3: str<3> = "中";
    let _f3: fmtstr<3, _> = f"中";
}
