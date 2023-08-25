use dep::safe_math::SafeU120;

// Binomial approximation of exponential
// using lower than decired precisions for everything due to u120 limit
// (1+x)^n = 1+n*x+[n/2*(n-1)]*x^2+[n/6*(n-1)*(n-2)*x^3]...
// we are loosing around almost 8 digits of precision from yearly -> daily interest
// dividing with 31536000 (seconds per year).
// rate must be measured with higher precision than 10^9.
// we use e18, and rates >= 4% yearly. Otherwise need more precision
fn compute_multiplier(
    rate_per_second: u120,
    dt: SafeU120,
) -> SafeU120 {
    let base = SafeU120{value: 1000000000}; // 1e9
    let WAD = SafeU120{value: 1000000000000000000}; // 1e18
    let diff = WAD.div(base);
    let mut res = base;
    if (!dt.is_zero()) {
        let exp_minus_one = SafeU120{value: dt.value - 1};
        let exp_minus_two = SafeU120 {value: if (dt.value > 2) {dt.value - 2} else { 0 }};

        // if rate_per_second < sqrt(WAD), then base_power_two and base_power_three = 0
        let rate = SafeU120 { value: rate_per_second };
        let base_power_two = rate.mul_div(rate, WAD);
        let base_power_three = base_power_two.mul_div(rate, WAD);

        let temp = dt.mul(exp_minus_one);
        let second_term = temp.mul(base_power_two).div(SafeU120 {value: 2});
        let third_term = temp.mul(exp_minus_two).mul(base_power_three).div(SafeU120{value: 6});

        // throwing away precision to keep us under u120 :sob:
        let offset = dt.mul(rate).add(second_term).add(third_term).div(diff);

        res = base.add(offset);
    }
    res
}