// This program will fail when the end of the loop is reached
#[fuzz(only_fail_with="reached_end_of_loop")]
fn fuzz(a:[u8; 10]) {
    let mut t=1;
    for i in 0..10{
        assert(a[i]==t);
        t=t+1;
    }
    assert(false, "reached_end_of_loop");
}
