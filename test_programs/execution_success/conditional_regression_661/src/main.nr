fn main(a: u32, mut c: [u32; 4]) {
    // Regression for issue #661:
    let mut c_661: [u32; 1] = [0];
    if a > 5 {
        c_661 = issue_661_foo(issue_661_bar(c), a);
    } else {
        c_661 = issue_661_foo(issue_661_bar(c), a + 2);
    }
    assert(c_661[0] < 20000);
}

fn test5(a: u32) {
    if a > 1 {
        let q = a / 2;
        assert(q == 2);
    }
}

fn issue_661_foo(array: [u32; 4], b: u32) -> [u32; 1] {
    [array[0] + b]
}

fn issue_661_bar(a: [u32; 4]) -> [u32; 4] {
    let mut b: [u32; 4] = [0; 4];
    b[0] = a[0] + 1;
    b
}

