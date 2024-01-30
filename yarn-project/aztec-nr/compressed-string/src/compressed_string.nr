use dep::aztec::protocol_types::{
  utils::field::field_from_bytes,
  traits::{Serialize, Deserialize}  
};
use dep::std;

// A Fixedsize Compressed String.
// Essentially a special version of Compressed String for practical use.
struct FieldCompressedString{
  value: Field
}

impl Serialize<1> for FieldCompressedString {
  fn serialize(self) -> [Field; 1] {
    [self.value]
  }
}

impl Deserialize<1> for FieldCompressedString {
  fn deserialize(input: [Field; 1]) -> Self {
    Self { value: input[0] }
  }
}

impl FieldCompressedString{
  pub fn is_eq(self, other: FieldCompressedString) -> bool {
    self.value == other.value
  }

  pub fn from_field(input_field: Field) -> Self {
    Self {value: input_field}
  }

  pub fn from_string(input_string: str<31>) -> Self {
    Self {value: field_from_bytes(input_string.as_bytes(), true)}
  }

  pub fn to_bytes(self) -> [u8; 31] {
    let mut result = [0; 31];
    let bytes = self.value.to_be_bytes(31);
    for i in 0..31 {
        result[i] = bytes[i];
    }
    result
  }
}

// The general Compressed String.
// Compresses M bytes into N fields.
// Can be used for longer strings that don't fit in a single field.
// Each field can store 31 characters, so N should be M/31 rounded up.
struct CompressedString<N, M> {
  value: [Field; N]
}

impl<N, M> CompressedString<N, M> {
  pub fn from_string(input_string: str<M>) -> Self {
    let mut fields = [0; N];
    let byts = input_string.as_bytes();

    let mut r_index = 0 as u32;

    for i in 0..N {
      let mut temp = [0 as u8; 31];
      for j in 0..31 {
        if r_index < M {
          temp[j] = byts[r_index];
          r_index += 1;
        }
      }

      fields[i] = field_from_bytes(temp, true);
    }

    Self { value: fields }
  }

  pub fn to_bytes(self) -> [u8; M] {
    let mut result = [0; M];
    let mut w_index = 0 as u32;
    for i in 0..N {
      let bytes = self.value[i].to_be_bytes(31);
      for j in 0..31 {
        if w_index < M {
          result[w_index] = bytes[j];
          w_index += 1;
        }
      }
    }
    result
  }

  pub fn serialize(self) -> [Field; N] {
    self.value
  }

  pub fn deserialize(input: [Field; N]) -> Self {
    Self { value: input }
  }
}

#[test]
fn test_short_string() {
    let i = "Hello world";
    let b = i.as_bytes();
    let name: CompressedString<1,11> = CompressedString::from_string(i);
    let p = b == name.to_bytes();
    assert(p, "invalid recover");
}

#[test]
fn test_long_string() {
    let i = "Hello world. I'm setting up a very long text of blibbablubb such that we can see if works as planned for longer names.";
    let b = i.as_bytes();
    let name: CompressedString<4,118> = CompressedString::from_string(i);
    let p = b == name.to_bytes();
    assert(p, "invalid recover");
}

#[test]
fn test_long_string_work_with_too_many_fields() {
    let i = "Hello world. I'm setting up a very long text of blibbablubb such that we can see if works as planned for longer names.";
    let b = i.as_bytes();
    let name: CompressedString<5,118> = CompressedString::from_string(i);
    let p = b == name.to_bytes();
    assert(p, "invalid recover");
}

#[test(should_fail)]
fn test_long_string_fail_with_too_few_fields() {
    let i = "Hello world. I'm setting up a very long text of blibbablubb such that we can see if works as planned for longer names.";
    let b = i.as_bytes();
    let name: CompressedString<3,118> = CompressedString::from_string(i);
    let p = b == name.to_bytes();
    assert(p, "invalid recover");
}
