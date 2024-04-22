---
title: How to deploy a contract with a Portal
---

- Deploy to L1 using Viem, Foundry or your preferred tool;
- Deploy to L2 and supply the L1 portal as an argument so you can store it in the contract;
  ```typescript
  const deploymentTx = Contract.deploy(wallet, tokenPortalAddress).send();
  ```
- Initialize l1 with l2 address for access control.

Follow the [token bridge tutorial](../../../tutorials/token_portal/main.md) for hands-on experience writing and deploying a Portal contract.