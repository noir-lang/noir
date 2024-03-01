// Binomial approximation of exponential
// using lower than desired precisions for everything due to u128 limit
// (1+x)^n = 1+n*x+[n/2*(n-1)]*x^2+[n/6*(n-1)*(n-2)*x^3]...
// we are loosing around almost 8 digits of precision from yearly -> daily interest
// dividing with 31536000 (seconds per year).
// rate must be measured with higher precision than 10^9.
// we use e18, and rates >= 4% yearly. Otherwise need more precision
pub fn compute_multiplier(rate_per_second: U128, dt: u64) -> U128 {
    let base = U128::from_integer(1000000000); // 1e9
    let WAD = U128::from_integer(1000000000000000000); // 1e18
    let diff = WAD.div(base);
    let mut res = base;
    if dt != 0 {
        let exp_minus_one = U128::from_integer(dt - 1);
        let exp_minus_two = U128::from_integer(if (dt > 2) { dt - 2 } else { 0 });
        let dt = U128::from_integer(dt);

        // if rate_per_second < sqrt(WAD), then base_power_two and base_power_three = 0
        let rate = rate_per_second;
        let base_power_two = (rate * rate) / WAD;
        let base_power_three = (base_power_two * rate) / WAD;

        let temp = dt.mul(exp_minus_one);
        let second_term = temp.mul(base_power_two).div(U128::from_integer(2));
        let third_term = temp.mul(exp_minus_two).mul(base_power_three).div(U128::from_integer(6));

        // throwing away precision to keep us under u128 :sob:
        let offset = dt.mul(rate).add(second_term).add(third_term).div(diff);

        res = base.add(offset);
    }
    res
}
