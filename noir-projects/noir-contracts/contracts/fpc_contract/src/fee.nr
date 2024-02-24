use dep::safe_math::SafeU120;
use dep::aztec::context::PublicContext;

pub fn calculate_fee(_context: PublicContext) -> SafeU120 {
    SafeU120::new(1)
}
