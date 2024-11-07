use super::Ssa;

impl Ssa {
    fn from_str(_str: &str) -> Result<Ssa, ()> {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::Ssa;

    #[test]
    fn test_ssa_from_str() {
        let src = "
            acir(inline) fn main f0 {
            b0():
                return
            }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        println!("{}", ssa);
    }
}
