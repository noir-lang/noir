trait Trait1 {
  // types and consts with the same name are allowed
  type Tralala;
  let Tralala: u32;
}

trait Trait2 {
  // consts and types with the same name are allowed
  let Tralala: u32;
  type Tralala;
}

trait Trait3 {
  // types and functions with the same name are allowed
  type Tralala;
  fn Tralala();
}

trait Trait4 {
  // functions and types with the same name are allowed
  fn Tralala();
  type Tralala;
}

fn main() {
}
