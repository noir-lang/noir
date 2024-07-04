use crate::TECurveParameterTrait;
use crate::Curve;

struct BabyJubJubParams {}
impl TECurveParameterTrait for BabyJubJubParams {
    fn a() -> Field { 168700 }
    fn d() -> Field { 168696 }
    fn gen() -> (Field, Field) { (0x0bb77a6ad63e739b4eacb2e09d6277c12ab8d8010534e0b62893f3f6bb957051, 0x25797203f7a0b24925572e1cd16bf9edfce0051fb9e133774b3c257a872d7d8b)}
}
type BabyJubJub = Curve<BabyJubJubParams>;