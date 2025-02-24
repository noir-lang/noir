pub trait Asd {
    fn asd(self) -> Field;
}

pub trait StaticTrait {
    fn static_function(slf: Self) -> Field {
        let _ = slf;
        100
    }
}
