global INPUT_SIZE: Field = 8;

fn main(input: [u1; INPUT_SIZE], input_length: u64) {
  constrain input_length <= (INPUT_SIZE as u64);

  for i in 0..INPUT_SIZE + 1 {
    if (i as u64) < input_length {
      constrain input[i] == 0;
    }
  };
}
