# VM threat model, security requirements

An honest Prover must always be able to construct a satisfiable proof for an AVM program, even if the program throws an error.
This implies constraints produced by the AVM **must** be satisfiable.
