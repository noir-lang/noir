fn main(mut x: [u32; 5], idx: Field) {
  // We should not hit out of bounds here as we have a predicate
  // that should not be hit
  if idx as u32 < 3 {    
    x[idx] = 10;
  }
  assert(x[4] == 111);
}