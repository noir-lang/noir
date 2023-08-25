mod interest_math;
use crate::interest_math::compute_multiplier;
use dep::std::hash::pedersen;
use dep::safe_math::SafeU120;


// Utility used to easily get a "id" for a private user that sits in the same
// "space" as the public users.
// It help us to have a single mapping for collateral that have both public and private users.
fn compute_identifier(
    secret: Field,
    on_behalf_of: Field,
    self: Field,
) -> Field {
    // EITHER secret OR on_behalf_of MUST be set. But not both
    assert (!((secret == 0) as bool & (on_behalf_of == 0) as bool));
    if (secret != 0) {
        pedersen([self, secret])[0]
    } else {
        on_behalf_of
    }
}

fn covered_by_collateral(
    price: u120,
    loan_to_value: u120,
    collateral: u120,
    increase: u120,
    decrease: u120,
) -> u120 {
    let price_precision = SafeU120{value: 1000000000};
    let ltv_precision = SafeU120{value: 10000};

    let price = SafeU120{value: price};
    let collateral = SafeU120{value: collateral}.add(SafeU120{value:increase}).sub(SafeU120{value:decrease});
    let loan_to_value = SafeU120{value: loan_to_value};

    let collateral_value = collateral.mul_div(price, price_precision);
    let debt_covered = collateral_value.mul_div(loan_to_value, ltv_precision);

    debt_covered.value
}

struct DebtReturn {
    debt_value: u120,
    static_debt: u120,
}

fn debt_updates(
    interest_accumulator: u120,
    static_debt: u120,
    increase: u120,
    decrease: u120,
) -> DebtReturn {
    assert(interest_accumulator > 0);
    let accumulator_precision = SafeU120{value: 1000000000};

    let static_debt = SafeU120{value: static_debt};
    let interest_accumulator = SafeU120{value: interest_accumulator};
    let increase = SafeU120{value: increase};
    let decrease = SafeU120{value: decrease};

    let current_debt_value = static_debt.mul_div(interest_accumulator, accumulator_precision);
    let new_debt_value = current_debt_value.add(increase).sub(decrease);

    // static_debt_increase = amount / accumulator
    // rounding up new debt.
    let static_debt_increase = increase.mul_div_up(accumulator_precision, interest_accumulator);
    // rounding down repayment.
    let static_debt_decrease = decrease.mul_div(accumulator_precision, interest_accumulator);

    // We need to allow repaying of the entire debt as well etc. This is very prone to failing
    // if you try to repay exact due to time diff between sim and execution.
    let new_static_debt = static_debt.add(static_debt_increase).sub(static_debt_decrease);

    DebtReturn {
        debt_value: new_debt_value.value,
        static_debt: new_static_debt.value,
    }
}

fn debt_value(
    static_debt: u120,
    interest_accumulator: u120,
) -> u120 {
    let static_debt = SafeU120{value: static_debt};
    let accumulator_precision = SafeU120{value: 1000000000};
    let interest_accumulator = SafeU120{value: interest_accumulator};
    static_debt.mul_div_up(interest_accumulator, accumulator_precision).value
}