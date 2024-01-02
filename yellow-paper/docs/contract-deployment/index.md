---
title: Contract Deployment
---

# Contract Deployment

Contracts in Aztec are deployed as _instances_ of a contract _class_. Deploying a new contract then requires first registering the _class_, if it has not been registered before, and then creating an _instance_ that references the class. Both classes and instances are committed to in the nullifier tree in the global state, and are created via a call to a canonical class registry or instance deployer contract respectively.

import DocCardList from '@theme/DocCardList';

<DocCardList />
