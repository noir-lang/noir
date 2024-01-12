use dep::std::cmp::Eq;

// Trait: is_empty
//
// The general is_empty trait checks if a data type is is empty,
// and it defines empty for the basic data types as 0.
//
// If a Field is equal to zero, then it is regarded as zero.
// We will go with this definition for now, however it can be problematic 
// if a value can actually be zero. In a future refactor, we can 
// use the optional type for safety. Doing it now would lead to a worse devex
// and would make it harder to sync up with the cpp code.
// Preferred over Default trait to convey intent, as default doesn't necessarily mean empty.
trait Empty {
    fn empty() -> Self;
}

impl Empty for Field { fn empty() -> Self {0} }

impl Empty for u1 { fn empty() -> Self {0} }
impl Empty for u8 { fn empty() -> Self {0} }
impl Empty for u16 { fn empty() -> Self {0} }
impl Empty for u32 { fn empty() -> Self {0} }
impl Empty for u64 { fn empty() -> Self {0} }

pub fn is_empty<T>(item: T) -> bool where T: Empty + Eq {
    item.eq(T::empty())
}

pub fn is_empty_array<T, N>(array: [T; N]) -> bool where T: Empty + Eq {
    array.all(|elem| is_empty(elem))
}

trait Hash {
    fn hash(self) -> Field;
}

trait ToField {
    fn to_field(self) -> Field;
}

impl ToField for Field {
    fn to_field(self) -> Field {
        self
    }
}
