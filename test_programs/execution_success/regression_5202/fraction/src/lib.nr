use dep::std;

global MAX: Fraction = Fraction { sign: true, num: 3050913689, den: 1 };
global MIN: Fraction = Fraction { sign: false, num: 3050913689, den: 1 };
global ZERO: Fraction = Fraction { sign: true, num: 0, den: 1 };
global ONE: Fraction = Fraction { sign: true, num: 1, den: 1 };
global NEGATIVE_ONE: Fraction = Fraction { sign: false, num: 1, den: 1 };

struct Fraction {
    sign: bool,
    num: u32,
    den: u32,
}

// Create a Fraction type variable without lenghtier code
pub fn toFraction(s: bool, n: u32, d: u32) -> Fraction {
    assert(d != 0);
    Fraction { sign: s, num: n, den: d }
}

// Swaps the numerator and denominator
fn invertFraction(f: Fraction) -> Fraction {
    assert(f.num != 0);
    Fraction { sign: f.sign, num: f.den, den: f.num }
}

// Changes the Sign of the Fraction
fn signChangeFraction(f: Fraction) -> Fraction {
    Fraction { sign: !f.sign, num: f.num, den: f.den }
}

// this method will only work till numerator and denominator values are under 100
// this has been set for efficiency reasons, and will be modified once the Noir team
// can implement dynamic limit for loops
fn reduceFraction(f: Fraction) -> Fraction {
    let mut a = f.num;
    let mut b = f.den;
    let mut j = 1;
    let mut gcd = 1;

    let min = if a > b { b } else { a };

    for i in 2..100 {
        j = i as u32;
        if (j <= min) {
            if (a % j == 0) & (b % j == 0) {
                gcd = j;
            }
        }
    }

    Fraction { sign: f.sign, num: f.num / gcd, den: f.den / gcd }
}

// Adds two fractions
pub fn addFraction(f1: Fraction, f2: Fraction) -> Fraction {
    let mut an = U128::from_integer(f1.num);
    let mut ad = U128::from_integer(f1.den);
    let mut bn = U128::from_integer(f2.num);
    let mut bd = U128::from_integer(f2.den);
    let mut m = f1;
    let mut n = f2;

    if f1.sign == f2.sign {
        if ((ad * bd > U128::from_integer(2000000000))
            | ((an * bd + ad * bn) > U128::from_integer(2000000000))
            | ((an * bd) > U128::from_integer(2000000000))
            | ((ad * bn) > U128::from_integer(2000000000))) {
            m = reduceFraction(m);
            n = reduceFraction(n);
        }
        an = U128::from_integer(m.num);
        ad = U128::from_integer(m.den);
        bn = U128::from_integer(n.num);
        bd = U128::from_integer(n.den);
        if ((ad * bd > U128::from_integer(2000000000))
            | ((an * bd + ad * bn) > U128::from_integer(2000000000))
            | ((an * bd) > U128::from_integer(2000000000))
            | ((ad * bn) > U128::from_integer(2000000000))) {
            let mut ddd = (an * bd + ad * bn) / (ad * bd);
            let mut factor = U128::from_integer(1);
            for _ in 1..5 {
                if ddd * U128::from_integer(10) < U128::from_integer(2000000000) {
                    ddd *= U128::from_integer(10);
                    factor *= U128::from_integer(10);
                }
            }
            let np: u32 = U128::to_integer(((an * bd + ad * bn) * factor) / (ad * bd));
            let fx: u32 = U128::to_integer(factor);
            Fraction { sign: f1.sign, num: np, den: fx }
        } else {
            Fraction { sign: f1.sign, num: (m.num * n.den + n.num * m.den), den: m.den * n.den }
        }
    } else if ((an * bd) > (bn * ad)) {
        if ((ad * bd > U128::from_integer(2000000000))
            | ((an * bd - ad * bn) > U128::from_integer(2000000000))
            | ((an * bd) > U128::from_integer(2000000000))) {
            m = reduceFraction(m);
            n = reduceFraction(n);
        }
        an = U128::from_integer(m.num);
        ad = U128::from_integer(m.den);
        bn = U128::from_integer(n.num);
        bd = U128::from_integer(n.den);

        if ((ad * bd > U128::from_integer(2000000000))
            | ((an * bd - ad * bn) > U128::from_integer(2000000000))
            | ((an * bd) > U128::from_integer(2000000000))) {
            let mut ddd = (an * bd - ad * bn) / (ad * bd);
            let mut factor = U128::from_integer(1);
            for _ in 1..5 {
                if ddd * U128::from_integer(10) < U128::from_integer(2000000000) {
                    ddd *= U128::from_integer(10);
                    factor *= U128::from_integer(10);
                }
            }
            let np: u32 = U128::to_integer(((an * bd - ad * bn) * factor) / (ad * bd));
            let fx: u32 = U128::to_integer(factor);
            Fraction { sign: f1.sign, num: np, den: fx }
        } else {
            Fraction { sign: f1.sign, num: (m.num * n.den - n.num * m.den), den: m.den * n.den }
        }
    } else {
        if ((ad * bd > U128::from_integer(2000000000))
            | ((bn * ad - bd * an) > U128::from_integer(2000000000))
            | ((bn * ad) > U128::from_integer(2000000000))) {
            m = reduceFraction(m);
            n = reduceFraction(n);
        }
        an = U128::from_integer(m.num);
        ad = U128::from_integer(m.den);
        bn = U128::from_integer(n.num);
        bd = U128::from_integer(n.den);
        if ((ad * bd > U128::from_integer(2000000000))
            | ((bn * ad - bd * an) > U128::from_integer(2000000000))
            | ((bn * ad) > U128::from_integer(2000000000))) {
            let mut ddd = (bn * ad - bd * an) / (ad * bd);
            let mut factor = U128::from_integer(1);
            for _ in 1..5 {
                if ddd * U128::from_integer(10) < U128::from_integer(2000000000) {
                    ddd *= U128::from_integer(10);
                    factor *= U128::from_integer(10);
                }
            }
            let np: u32 = U128::to_integer(((bn * ad - bd * an) * factor) / (ad * bd));
            let fx: u32 = U128::to_integer(factor);
            Fraction { sign: f2.sign, num: np, den: fx }
        } else {
            Fraction { sign: f2.sign, num: (n.num * m.den - m.num * n.den), den: m.den * n.den }
        }
    }
}

// Returns the closest but smaller Integer to the Given Fraction, but typecast to Fraction for convenience
pub fn floor(f: Fraction) -> Fraction {
    let q = f.num / f.den;
    if q * f.den == f.num {
        Fraction { sign: f.sign, num: f.num, den: f.den }
    } else if f.sign {
        Fraction { sign: f.sign, num: q, den: 1 }
    } else {
        Fraction { sign: f.sign, num: q + 1, den: 1 }
    }
}

#[test]
fn test_sum() {
    let f1 = toFraction(true, 3, 5);
    let f2 = toFraction(true, 2, 5);
    let f = addFraction(f1, f2);
    assert(f.num == f.den);
}

#[test]
fn test_reduce() {
    let f1 = toFraction(true, 2, 10);
    let f2 = reduceFraction(f1);
    assert(f2.num == 1);
}

#[test]
fn test_floor() {
    let f = toFraction(true, 7, 5);
    let fl = floor(f);
    assert(fl.num == 1);
    assert(fl.den == 1);
}

#[test]
fn test_floor2() {
    let f = toFraction(false, 12, 5);
    let fl = floor(f);
    assert(fl.num == 3);
    assert(fl.den == 1);
}

#[test]
fn test_globals() {
    let a = addFraction(ONE, NEGATIVE_ONE);
    assert(a.num == ZERO.num);
}
