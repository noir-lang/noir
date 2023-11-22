---
title: Elliptic Curve Primitives
keywords: [cryptographic primitives, Noir project]
---

Data structures and methods on them that allow you to carry out computations involving elliptic
curves over the (mathematical) field corresponding to `Field`. For the field currently at our
disposal, applications would involve a curve embedded in BN254, e.g. the
[Baby Jubjub curve](https://eips.ethereum.org/EIPS/eip-2494).

## Data structures

### Elliptic curve configurations

(`std::ec::{tecurve,montcurve,swcurve}::{affine,curvegroup}::Curve`), i.e. the specific elliptic
curve you want to use, which would be specified using any one of the methods
`std::ec::{tecurve,montcurve,swcurve}::{affine,curvegroup}::new` which take the coefficients in the
defining equation together with a generator point as parameters. You can find more detail in the
comments in
[`noir_stdlib/src/ec.nr`](https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ec.nr), but
the gist of it is that the elliptic curves of interest are usually expressed in one of the standard
forms implemented here (Twisted Edwards, Montgomery and Short Weierstraß), and in addition to that,
you could choose to use `affine` coordinates (Cartesian coordinates - the usual (x,y) - possibly
together with a point at infinity) or `curvegroup` coordinates (some form of projective coordinates
requiring more coordinates but allowing for more efficient implementations of elliptic curve
operations). Conversions between all of these forms are provided, and under the hood these
conversions are done whenever an operation is more efficient in a different representation (or a
mixed coordinate representation is employed).

### Points

(`std::ec::{tecurve,montcurve,swcurve}::{affine,curvegroup}::Point`), i.e. points lying on the
elliptic curve. For a curve configuration `c` and a point `p`, it may be checked that `p`
does indeed lie on `c` by calling `c.contains(p1)`.

## Methods

(given a choice of curve representation, e.g. use `std::ec::tecurve::affine::Curve` and use
`std::ec::tecurve::affine::Point`)

- The **zero element** is given by `Point::zero()`, and we can verify whether a point `p: Point` is
  zero by calling `p.is_zero()`.
- **Equality**: Points `p1: Point` and `p2: Point` may be checked for equality by calling
  `p1.eq(p2)`.
- **Addition**: For `c: Curve` and points `p1: Point` and `p2: Point` on the curve, adding these two
  points is accomplished by calling `c.add(p1,p2)`.
- **Negation**: For a point `p: Point`, `p.negate()` is its negation.
- **Subtraction**: For `c` and `p1`, `p2` as above, subtracting `p2` from `p1` is accomplished by
  calling `c.subtract(p1,p2)`.
- **Scalar multiplication**: For `c` as above, `p: Point` a point on the curve and `n: Field`,
  scalar multiplication is given by `c.mul(n,p)`. If instead `n :: [u1; N]`, i.e. `n` is a bit
  array, the `bit_mul` method may be used instead: `c.bit_mul(n,p)`
- **Multi-scalar multiplication**: For `c` as above and arrays `n: [Field; N]` and `p: [Point; N]`,
  multi-scalar multiplication is given by `c.msm(n,p)`.
- **Coordinate representation conversions**: The `into_group` method converts a point or curve
  configuration in the affine representation to one in the CurveGroup representation, and
  `into_affine` goes in the other direction.
- **Curve representation conversions**: `tecurve` and `montcurve` curves and points are equivalent
  and may be converted between one another by calling `into_montcurve` or `into_tecurve` on their
  configurations or points. `swcurve` is more general and a curve c of one of the other two types
  may be converted to this representation by calling `c.into_swcurve()`, whereas a point `p` lying
  on the curve given by `c` may be mapped to its corresponding `swcurve` point by calling
  `c.map_into_swcurve(p)`.
- **Map-to-curve methods**: The Elligator 2 method of mapping a field element `n: Field` into a
  `tecurve` or `montcurve` with configuration `c` may be called as `c.elligator2_map(n)`. For all of
  the curve configurations, the SWU map-to-curve method may be called as `c.swu_map(z,n)`, where
  `z: Field` depends on `Field` and `c` and must be chosen by the user (the conditions it needs to
  satisfy are specified in the comments
  [here](https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ec.nr)).

## Examples

The
[ec_baby_jubjub test](https://github.com/noir-lang/noir/blob/master/crates/nargo_cli/tests/test_data/ec_baby_jubjub/src/main.nr)
illustrates all of the above primitives on various forms of the Baby Jubjub curve. A couple of more
interesting examples in Noir would be:

Public-key cryptography: Given an elliptic curve and a 'base point' on it, determine the public key
from the private key. This is a matter of using scalar multiplication. In the case of Baby Jubjub,
for example, this code would do:

```rust
use dep::std::ec::tecurve::affine::{Curve, Point};

fn bjj_pub_key(priv_key: Field) -> Point
{

 let bjj = Curve::new(168700, 168696, G::new(995203441582195749578291179787384436505546430278305826713579947235728471134,5472060717959818805561601436314318772137091100104008585924551046643952123905));

 let base_pt = Point::new(5299619240641551281634865583518297030282874472190772894086521144482721001553, 16950150798460657717958625567821834550301663161624707787222815936182638968203);

 bjj.mul(priv_key,base_pt)
}
```

This would come in handy in a Merkle proof.

- EdDSA signature verification: This is a matter of combining these primitives with a suitable hash
  function. See
  [feat(stdlib): EdDSA sig verification noir#1136](https://github.com/noir-lang/noir/pull/1136) for
  the case of Baby Jubjub and the Poseidon hash function.
