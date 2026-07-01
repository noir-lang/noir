use std::append::Append;
use std::cmp::Eq;
use std::meta::ctstring::AsCtString;

fn main() {
    comptime {
        let msg1 = "msg1";
        let msg2 = f" (msg2 contains {msg1}) ";

        let s1 = msg1.as_ctstring();
        let s2 = msg2.as_ctstring();

        let s3 = [s1, s2, s1].as_vector().join("".as_ctstring());
        assert_eq(s3, "msg1 (msg2 contains msg1) msg1".as_ctstring());

        let mut s4 = s1;
        s4 = s4.append_fmtstr(msg2);
        s4 = s4.append_str(msg1);
        assert_eq(s3, s4);

        // Appending must preserve the raw bytes of non-UTF-8 `CtString`s instead of
        // routing them through a lossy UTF-8 conversion.
        let bad_ct: CtString = [0xC0].as_str_unchecked().as_ctstring();

        // Both monoid identity laws must hold even for non-UTF-8 bytes.
        assert_eq(bad_ct.append(CtString::empty()), bad_ct);
        assert_eq(CtString::empty().append(bad_ct), bad_ct);

        // Distinct byte strings must not collide after appending empty.
        let other_ct: CtString = [0xC1].as_str_unchecked().as_ctstring();
        assert(!bad_ct.eq(other_ct));
        assert(!bad_ct.append(CtString::empty()).eq(other_ct.append(CtString::empty())));

        // Interpolating a non-UTF-8 `CtString` into a fmtstr and converting back must be
        // byte-exact.
        assert(bad_ct.eq(f"{bad_ct}".as_ctstring()));

        // Byte-backed values nested inside a container must also be preserved: an array
        // renders as `[<element>]`, i.e. bytes `[`, 0xC0, `]`.
        let bad_str: str<1> = [0xC0].as_str_unchecked();
        let arr: [str<1>; 1] = [bad_str];
        let expected: CtString = [0x5B, 0xC0, 0x5D].as_str_unchecked().as_ctstring();
        assert(f"{arr}".as_ctstring().eq(expected));
    }
}
