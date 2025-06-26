
    pub struct Reader<let N: u32> {}

    pub trait Deserialize<T, let SERIALIZED_LEN: u32> {
        fn deserialize(x: T, _serialized: [T; SERIALIZED_LEN]) {
            let _: [T; SERIALIZED_LEN] = [x; SERIALIZED_LEN];
        }
    }

    impl Deserialize<i32, 1> for Field {}

    fn main() {}
    