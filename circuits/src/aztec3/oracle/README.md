# aztec3::circuits::oracle

When Aztec3 makes a call to a circuit executor (such as Noir or the test apps in aztec3/circuits/apps/test_apps), we don't pass _all_ public inputs when calling the circuit; only the `custom_inputs` inputs are sent initially.

Why?

This has two main benefits:

- We only need one implementation of the circuit's logic: the circuit itself. We don't need to implement a separate native version of the circuit to 'predict' what commitments/nullifiers and other values will need to ultimately become inputs to the circuit.
- There are occasions where it's difficult to predict the inputs required in advance. E.g. if a circuit_1 makes a call to another circuit, circuit_2, at some other contract_address, but that address is calculated within the body of circuit_1. We'd need to independently deduce the address which would be called so that we could grab the correct contract_address, VK, vkIndex, proving key (etc) from the DB.

Instead, we provide an oracle to the circuit executor when we call it. As the circuit is executed, it'll occasionally reach points where it needs more input data. The circuit executor can query the oracle (the Aztec3 node) for more data. The oracle can grab data from its DB and return it the circuit executor. The circuit executor can then convert the data into witnesses and continue with some more of the circuit's execution.

