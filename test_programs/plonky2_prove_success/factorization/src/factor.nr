// factor.nr
pub fn print_factors(mut number: u32) {
    // the number of primes limits the input to (59*59)^2 - 1 = 12117360
    // the maximum factor limits the input to 256*256 - 1 = 65535
    assert(number < 65536);
    let mut factor_is_first = true;
    //if number < 0 {
    //    number = number * -1;
    //    println(-1);
    //    factor_is_first = false;
    //}

    // at most 16 primes
    let mut primes: [u32; 0x10] = [0; 0x10];
    let mut num_of_primes = 0;

    // The first 16 primes are enough to figure out whether numbers up to 3480
    // are prime; the 17th prime is 59, which squared is 3481.
    for factor in 2..256 {
        if factor * factor > number {
            // performance-break;
        } else {
            let mut factor_is_prime = true;

            for prime_index in 0..16 {
                if (prime_index >= num_of_primes) {
                    // semantic-break;
                } else {
                    let p = primes[prime_index];
                    assert(p != 0);
                    let quotient = factor / p;
                    if quotient * p == factor {
                        factor_is_prime = false;
                        // performance-break;
                    }
                    if p * p > factor {
                        // performance-break;
                    }
                }
            }

            if factor_is_prime & (num_of_primes < 16) {
                primes[num_of_primes] = factor;
                num_of_primes += 1;
            }

            let mut count = 0;
            // number is at most factor ^ 16
            for _ in 0..16 {
                let quotient = number / factor;
                if quotient * factor == number {
                    count += 1;
                    number = quotient;
                } else {
                    // performance-break
                }
            }

            factor_is_first = print_factor(factor, factor_is_first, count);
        }
    }

    if number > 1 {
        factor_is_first = print_factor(number, factor_is_first, 1);
    }

    println("");
}

fn print_factor(factor: u32, mut factor_is_first: bool, count: u32) -> bool {
    if count > 0 {
        if !factor_is_first {
            print(" ");
        }

        print(factor);

        if count > 1 {
            print("(");
            print(count);
            print(")");
        }

        if factor_is_first {
            factor_is_first = false;
        }
    }
    factor_is_first
}
