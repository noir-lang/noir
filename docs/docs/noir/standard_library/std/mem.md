---
title: mem
---

# Module `std::mem`

## zeroed

```noir
fn zeroed<T>() -> T
```

For any type, return an instance of that type by initializing
all of its fields to 0. This is considered to be unsafe since there
is no guarantee that all zeroes is a valid bit pattern for every type.

