// issue #3639
fn main() {
    for i in 0..8 {
        for j in 0..8 {
            if  (i as u64 + j as u64 < 8) {
                // pass
            }
        }
    }
}
