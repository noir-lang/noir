use std::meta::ctstring::AsCtString;

fn main() {
    comptime {
        let msg1 = "msg1";
        let msg2 = f" (msg2 contains {msg1}) ";

        let s1 = msg1.as_ctstring();
        let s2 = msg2.as_ctstring();

        let s3 = &[s1, s2, s1].join("".as_ctstring());
        assert_eq(s3, "msg1 (msg2 contains msg1) msg1".as_ctstring());

        let mut s4 = s1;
        s4 = s4.append_fmtstr(msg2);
        s4 = s4.append_str(msg1);
        assert_eq(s3, s4);
    }
}
