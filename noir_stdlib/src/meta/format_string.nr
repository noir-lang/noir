use crate::option::Option;

impl <let N: u32, T> fmtstr<N, T> {
    fn as_identifier(self) -> Option<Quoted> {
        Option::none()
    }
}
