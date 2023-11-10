---
title: Debugging
---

import DocCardList from '@theme/DocCardList';

On this section you can learn how to debug your Aztec.nr smart contracts and common errors that you may run into.

# Logging in Aztec.nr

You can log statements from Aztec.nr contracts that will show ups in the Sanbox.

**Import debug_log**
Import the [`debug_log`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/oracle/debug_log.nr) dependency from Aztec oracles:

```rust
use dep::aztec::oracle::debug_log::{ debug_log };
```

**Write log**
Write `debug_log()` in the appropriate place in your contract. 

```rust
debug_log("here")
```

**Start Sandbox in debug mode**

Prepend the command to start the sandbox with `DEBUG=aztec` to log everything or `DEBUG=aztec:simulator:oracle` to only log your `debug_log()` statements.

```bash
cd ~./aztec && DEBUG=aztec docker-compose up
```


<DocCardList/>
