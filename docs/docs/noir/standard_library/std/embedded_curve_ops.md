---
title: embedded_curve_ops
---

# Module `std::embedded_curve_ops`

## Struct `EmbeddedCurvePoint`

A point on the embedded elliptic curve
By definition, the base field of the embedded curve is the scalar field of the proof system curve, i.e the Noir Field.
x and y denotes the Weierstrass coordinates of the point, if is_infinite is false.

### Methods

##### multi_scalar_mul

```noir
fn multi_scalar_mul<let N: u32>(points: [EmbeddedCurvePoint; N], scalars: [EmbeddedCurveScalar; N]) -> EmbeddedCurvePoint
```

##### fixed_base_scalar_mul

```noir
fn fixed_base_scalar_mul(scalar: EmbeddedCurveScalar) -> EmbeddedCurvePoint
```

