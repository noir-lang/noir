# AztecRPCServer & AztecRPCClient

`AztecRPCServer` is a server-side software that provides a set of apis for interacting with the Aztec network. It acts as a bridge between the network and client applications by exposing methods to manage accounts, deploy contracts, create transactions, and retrieve public and account-specific information. It provides a secure environment for the execution of sensitive operations, ensuring private information and decrypted data are not accessible to unauthorized applications. The AztecRPCServer is a critical component of the Aztec network, and its security and reliability are essential to the overall trustworthiness of user's data.

`AztecRPCClient` is the interface the AztecRPCServer and every client-side instance implement. Various implementations of the client-side can exist, including those for different platforms such as mobile apps and web browsers. It is a relay between the dApps and the actual AztecRPCServer.

### Main Components in an AztecRPCServer

- [Acir Simulator](../acir-simulator/)
- [Key Store](../key-store/)
- [Account State](./src/account_state/account_state.ts): It coordinates other components to synchronize and decrypt data, simulate transactions, and generate kernel proofs, for a specific account.
