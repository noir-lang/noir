use dep::aztec::context::interface::PublicContextInterface;

pub fn calculate_fee<TPublicContext>(_context: TPublicContext) -> U128 where TPublicContext: PublicContextInterface {
    U128::from_integer(1)
}
