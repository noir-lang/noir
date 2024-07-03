---
title: Published Data Format
---

The "Effects" of a transaction are the collection of state changes and metadata that resulted from executing a transaction. These include:

| Field                | Type                                                                    | Description                                                                          |
| -------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| `revertCode`         | `RevertCode`                                                            | Indicates the reason for reverting in public application logic. 0 indicates success. |
| `note_hashes`        | `Tuple<Fr, typeof MAX_NOTE_HASHES_PER_TX>`                          | The note hashes to be inserted into the note hash tree.                              |
| `nullifiers`         | `Tuple<Fr, typeof MAX_NULLIFIERS_PER_TX>`                           | The nullifiers to be inserted into the nullifier tree.                               |
| `l2_to_l2_msgs`      | `Tuple<Fr, typeof MAX_L2_TO_L1_MSGS_PER_TX>`                        | The L2 to L1 messages to be inserted into the messagebox on L1.                      |
| `public_data_writes` | `Tuple<PublicDataWrite, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>` | Public data writes to be inserted into the public data tree                          |
| `encrypted_logs`     | `TxL2Logs`                                                              | Buffers containing the emitted encrypted logs.                                       |
| `unencrypted_logs`   | `TxL2Logs`                                                              | Buffers containing the emitted unencrypted logs.                                     |

Each can have several transactions. Thus, an block is presently encoded as:

| byte start                                                                                               | num bytes | name                                    |
| -------------------------------------------------------------------------------------------------------- | --------- | --------------------------------------- |
| 0x0                                                                                                      | 0x4       | len(newL1ToL2Msgs) (denoted a)          |
| 0x4                                                                                                      | a \* 0x20 | newL1ToL2Msgs                           |
| 0x4 + a \* 0x20 = tx0Start                                                                               | 0x4       | len(numTxs) (denoted t)                 |
|                                                                                                          |           | TxEffect 0                            |
| tx0Start                                                                                                 | 0x20      | revertCode                              |
| tx0Start + 0x20                                                                                          | 0x1       | len(noteHashes) (denoted b)          |
| tx0Start + 0x20 + 0x1                                                                                    | b \* 0x20 | noteHashes                           |
| tx0Start + 0x20 + 0x1 + b \* 0x20                                                                        | 0x1       | len(nullifiers) (denoted c)          |
| tx0Start + 0x20 + 0x1 + b \* 0x20 + 0x1                                                                  | c \* 0x20 | nullifiers                           |
| tx0Start + 0x20 + 0x1 + b \* 0x20 + 0x1 + c \* 0x20                                                      | 0x1       | len(l2ToL1Msgs) (denoted d)          |
| tx0Start + 0x20 + 0x1 + b \* 0x20 + 0x1 + c \* 0x20 + 0x1                                                | d \* 0x20 | l2ToL1Msgs                           |
| tx0Start + 0x20 + 0x1 + b \* 0x20 + 0x1 + c \* 0x20 + 0x1 + d \* 0x20                                    | 0x1       | len(newPublicDataWrites) (denoted e)    |
| tx0Start + 0x20 + 0x1 + b \* 0x20 + 0x1 + c \* 0x20 + 0x1 + d \* 0x20 + 0x01                             | e \* 0x40 | newPublicDataWrites                     |
| tx0Start + 0x20 + 0x1 + b \* 0x20 + 0x1 + c \* 0x20 + 0x1 + d \* 0x20 + 0x01 + e \* 0x40                 | 0x04      | byteLen(newEncryptedLogs) (denoted f)   |
| tx0Start + 0x20 + 0x1 + b \* 0x20 + 0x1 + c \* 0x20 + 0x1 + d \* 0x20 + 0x01 + e \* 0x40 + 0x4           | f         | newEncryptedLogs                        |
| tx0Start + 0x20 + 0x1 + b \* 0x20 + 0x1 + c \* 0x20 + 0x1 + d \* 0x20 + 0x01 + e \* 0x40 + 0x4 + f       | 0x04      | byteLen(newUnencryptedLogs) (denoted g) |
| tx0Start + 0x20 + 0x1 + b \* 0x20 + 0x1 + c \* 0x20 + 0x1 + d \* 0x20 + 0x01 + e \* 0x40 + 0x4 + f + 0x4 | g         | newUnencryptedLogs                      |
|                                                                                                          |           | },                                      |
|                                                                                                          |           | TxEffect 1                            |
|                                                                                                          |           | ...                                     |
|                                                                                                          |           | },                                      |
|                                                                                                          |           | ...                                     |
|                                                                                                          |           | TxEffect (t - 1)                      |
|                                                                                                          |           | ...                                     |
|                                                                                                          |           | },                                      |
