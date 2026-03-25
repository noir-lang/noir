/// Generates a Deserialize implementation for the give type name using the FromStr
/// for the give type.
#[macro_export]
macro_rules! fromstr_deser {
  ($e:tt) => {
    impl<'de> Deserialize<'de> for $e {
      fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
      {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
      }
    }
  }
}

#[macro_export]
macro_rules! tostr_ser {
  ($e:tt) => {
    impl Serialize for $e {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where
        S: serde::Serializer,
      {
        serializer.serialize_str(&self.to_string())
      }
    }
  }
}
