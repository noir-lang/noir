# Module `std::meta::typ`

## `Type` methods

### as_array

```noir
fn as_array(self) -> Option<(Type, Type)>
```

### as_constant

```noir
fn as_constant(self) -> Option<u32>
```

### as_integer

```noir
fn as_integer(self) -> Option<(bool, u8)>
```

### as_slice

```noir
fn as_slice(self) -> Option<Type>
```

### as_str

```noir
fn as_str(self) -> Option<Type>
```

### as_struct

```noir
fn as_struct(self) -> Option<(StructDefinition, [Type])>
```

### as_tuple

```noir
fn as_tuple(self) -> Option<[Type]>
```

### get_trait_impl

```noir
fn get_trait_impl(self, constraint: TraitConstraint) -> Option<TraitImpl>
```

### implements

```noir
fn implements(self, constraint: TraitConstraint) -> bool
```

### is_bool

```noir
fn is_bool(self) -> bool
```

### is_field

```noir
fn is_field(self) -> bool
```

## fresh_type_variable

```noir
fn fresh_type_variable() -> Type
```

