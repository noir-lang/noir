# PXE Service & PXE

`PXE Service` (pronounced "pixie service") is a server-side software that provides a set of apis for interacting with the Aztec network. It acts as a bridge between the network and client applications by exposing methods to manage accounts, deploy contracts, create transactions, and retrieve public and account-specific information. It provides a secure environment for the execution of sensitive operations, ensuring private information and decrypted data are not accessible to unauthorized applications. The PXE Service is a critical component of the Aztec network, and its security and reliability are essential to the overall trustworthiness of user's data.

`PXE` is the interface the PXE Service and every client-side instance implement. Various implementations of the client-side can exist, including those for different platforms such as mobile apps and web browsers. It is a relay between the dApps and the actual PXE Service.

### Main Components in an PXE Service

- [Acir Simulator](../acir-simulator/)
- [Key Store](../key-store/)
- [Account State](./src/account_state/account_state.ts): It coordinates other components to synchronize and decrypt data, simulate transactions, and generate kernel proofs, for a specific account.

![Pixie](./pixie.png)