pub use dep::std::embedded_curve_ops::EmbeddedCurveScalar as Scalar;
use crate::traits::{Empty, Serialize};

global SCALAR_SIZE: u32 = 2;

impl Empty for Scalar {
    fn empty() -> Self {
        Self { lo: 0, hi: 0 }
    }
}

impl Serialize<SCALAR_SIZE> for Scalar {
    fn serialize(self) -> [Field; SCALAR_SIZE] {
        [self.lo, self.hi]
    }
}
