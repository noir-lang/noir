
struct SafeU120 {
    value: u120,
}

impl SafeU120 {
    fn is_zero(
        self: Self,
    ) -> bool {
        self.value == 0
    }

    fn sub(
      self: Self,
      b: SafeU120,
    ) -> SafeU120 {
        assert(self.value >= b.value);
        SafeU120 {
            value: self.value - b.value
        }
    }

    fn add(
        self: Self,
        b: SafeU120,
    ) -> SafeU120 {
        let c: u120 = self.value + b.value;
        assert(c >= self.value);
        SafeU120 {
            value: c
        }
    }

    fn mul(
        self: Self,
        b: SafeU120,
    ) -> SafeU120 {
        let c: u120 = self.value * b.value;
        // checking for overflows. If both self.value and b are not 0, then c must be greater than or equal self.value.
        assert((self.value == 0) | (b.value == 0) | (c >= self.value));
        SafeU120 {
            value: c
        }
    }

    fn div(
        self: Self,
        b: SafeU120,
    ) -> SafeU120 {
        assert(b.value != 0);
        SafeU120 {
            value: self.value / b.value
        }
    }

    // todo: Implement a version that avoids shadow-overflows
    // (i.e., overflows by self*b where c would not overflow)
    fn mul_div(
        self: Self,
        b: SafeU120,
        divisor: SafeU120
    ) -> SafeU120 {
        let c = SafeU120::mul(self, b);
        SafeU120 {
            value: c.value / divisor.value
        }
    }

     fn mul_div_up(
        self: Self,
        b: SafeU120,
        divisor: SafeU120
    ) -> SafeU120 {
        let c = SafeU120::mul(self, b);
        let adder = ((self.value * b.value % divisor.value) as u120 > 0) as u120;
        SafeU120 {
            value: c.value / divisor.value + adder
        }
    }

    // todo: implement mul_div with 240 bit intermediate values.
}