struct FunctionSelector {
}

impl FunctionSelector {
    pub fn from_signature<let N: u32>(s: str<N>) -> Field {
        if s.as_bytes_vec().len() == "one(Field)".as_bytes_vec().len() {
            0
        } else if s.as_bytes_vec().len() == "two(Field, Point)".as_bytes_vec().len() {
            1
        } else {
            2
        }
    }
}
