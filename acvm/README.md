# ACIR - Abstract Circuit Intermediate Representation

ACIR is an NP complete language that generalizes R1CS and arithmetic circuits while not losing proving system specific optimizations through the use of black box functions.

# ACVM - Abstract Circuit Virtual Machine

This can be seen as the ACIR compiler. It will take an ACIR instance and convert it to the format required
by a particular proving system to create a proof.