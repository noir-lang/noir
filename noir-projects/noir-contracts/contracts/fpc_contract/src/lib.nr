use dep::aztec::context::interface::PublicContextInterface;

pub fn compute_rebate<TPublicContext>(
    context: TPublicContext,
    initial_amount: Field
) -> Field where TPublicContext: PublicContextInterface {
    let actual_fee = context.transaction_fee();
    assert(!initial_amount.lt(actual_fee), "Initial amount paid to the paymaster does not cover actual fee");
    initial_amount - actual_fee
}
