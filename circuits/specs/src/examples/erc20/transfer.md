# Transfer

A transfer is an entirely private function.

- A user's client generates a proof over the 'transfer' circuit, with a particular set of public/private inputs. 
- The user's client then passes the proof to the Private Kernel Circuit and generates a proof.
  - This proof hides the function which has been executed.
  - A padding proof could then be generated to add more commitments/nullifiers to the kernel snark's public inputs, as a way of further obfuscating which tx was executed. Then that would all be passed into a second Private Kernel Circuit and a second such proof generated.
- The 'transfer' function doesn't make any public calls or contract deployment calls.
- The user sends their final kernel snark to the rollup provider
- The rollup provider adds the user's tx to a rollup (by passing the snark as input to a Base Rollup circuit, and then into a Merge Rollup circuit).
- Eventually the Rollup Processor contract receives the rollup and verifies the final rollup proof.