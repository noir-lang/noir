use dep::aztec::context::PublicContext;

pub fn compute_rebate(context: PublicContext, initial_amount: Field) -> Field {
    let actual_fee = context.transaction_fee();
    assert(!initial_amount.lt(actual_fee), "Initial amount paid to the paymaster does not cover actual fee");
    initial_amount - actual_fee
}
