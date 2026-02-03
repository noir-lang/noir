pub trait Deserialize {
    let N: u32;

    fn deserialize(fields: [Field; Self::N]) -> Self;
}

pub struct PrivateCallInterface<T> {
  return_type: T
}

pub struct PrivateVoidCallInterface {
  return_type: ()
}

trait SuperCallInterface where Self::T: Deserialize<N = Self::M> {
    type T;
    let M: u32;

    fn call(self) -> Self::T;
}

impl<T> SuperCallInterface for PrivateCallInterface<T>
    where T: Deserialize
{
    type T = T;
    let M: u32 = <T as Deserialize>::N;

    fn call(self) -> T {
        let preimage: [Field; Self::M] = std::mem::zeroed();

        let returns: T = T::deserialize(preimage);
        returns
    }
}

// Removing this impl fixes the panic despite this type being unused
impl SuperCallInterface<()> for PrivateVoidCallInterface {
    let M: u32 = 0;

    fn call(self) -> () {
        ()
    }
}

unconstrained fn main() {
  let _should_be_field = PrivateCallInterface {return_type: 1}.call();
}
