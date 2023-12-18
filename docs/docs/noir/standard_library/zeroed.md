---
title: Zeroed Function
description:
  The zeroed function returns a zeroed value of any type.
keywords:
  [
    zeroed
  ]
---

Implements `fn zeroed<T>() -> T` to return a zeroed value of any type. This function is generally unsafe to use as the zeroed bit pattern is not guaranteed to be valid for all types. It can however, be useful in cases when the value is guaranteed not to be used such as in a BoundedVec library implementing a growable vector, up to a certain length, backed by an array. The array can be initialized with zeroed values which are guaranteed to be inaccessible until the vector is pushed to. Similarly, enumerations in noir can be implemented using this method by providing zeroed values for the unused variants.

You can access the function at `std::unsafe::zeroed`.

This function currently supports the following types:

- Field
- Bool
- Uint
- Array
- String
- Tuple
- Function
  
Using it on other types could result in unexpected behavior.
