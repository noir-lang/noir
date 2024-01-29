---
title: Debugging
---

import DocCardList from '@theme/DocCardList';

On this section you can learn how to debug your Aztec.nr smart contracts and common errors that you may run into.

# Logging in Aztec.nr

You can log statements from Aztec.nr contracts that will show ups in the Sandbox.

**Import debug_log**
Import the [`debug_log`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/oracle/debug_log.nr) dependency from Aztec oracles:

```rust
use dep::aztec::oracle::debug_log::{ debug_log };
```

**Write log**

Write `debug_log()` in the appropriate place in your contract.

```rust
debug_log("here");
```

Other methods for logging include:

`debug_log_format()`: for logging Field values along arbitrary strings.

```rust
debug_log_format("get_2(slot:{0}) =>\n\t0:{1}\n\t1:{2}", [storage_slot, note0_hash, note1_hash]);
```

`debug_log_field()`: for logging Fields.

```rust
debug_log_field(my_field);
```

`debug_log_array()`: for logging array types.

```rust
debug_log_array(my_array);
```

**Start Sandbox in debug mode**

Prepend the command to start the sandbox with `DEBUG=aztec:*` to log everything or `DEBUG=aztec:simulator:oracle` to only log your `debug_log()` statements.

```bash
# Using the docker-compose.yml setup
cd ~./aztec && DEBUG=aztec:* docker-compose up
```

Alternatively you can update the `DEBUG` environment variable in docker-compose.yml and start the sandbox normally.

```yml
environment:
  DEBUG: aztec:*
```

<DocCardList/>
