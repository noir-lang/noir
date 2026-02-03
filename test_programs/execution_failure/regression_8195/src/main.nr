global G_A: u16 = 41618;
fn main(a: u16) -> pub u16 {
    ((13834 - a) % (G_A * G_A))
}
