# Sequencer API

## Rollup Blocks

Used to request a list of blocks or a specific block by ID

### List of Rollup Blocks

Returns an array of rollup block information, starting from **latest** one.

**URL** : `/falafel/rollups`

**Method** : `GET`

**Query Params:**

- `skip`: number of blocks to skip. **DEFAULT**: `0`
- `take`: number of blocks to be returned. **DEFAULT**: `5`

**Example:**

```
GET https://api.aztec.network/aztec-connect-prod/falafel/rollups?take=10&skip=100
```

### Success Response

**Code** : `200 OK`

**Content example**

```json
[
  {
    "id": 4429,
    "hash": "541bce9d62eb99447a5969d9791b12274ed199998c9842f3ec4135180daec118",
    "numTxs": 59,
    "ethTxHash": "0xa1f15bd3954c5bb0528d79e7d49b295e9e84458f3e16fb04877aef571d817ea2",
    "created": "2022-11-22T13:20:50.882Z",
    "mined": "2022-11-22T13:21:23.000Z"
  },
  {
    "id": 4428,
    "hash": "e787f8ccdbb33ecfd13f493249854aa1e3350b16b0dd04a9d78cd7a1202358bb",
    "numTxs": 76,
    "ethTxHash": "0xf4a436dfd79dac7bac76bcd4f9dc0133171c7a1c44e32f28a686bdfb5c9ca2d1",
    "created": "2022-11-22T12:42:59.330Z",
    "mined": "2022-11-22T12:43:23.000Z"
  }
]
```

### Error Response

**Condition** : If `skip` param is larger than number of rollups.

**Code** : `200 OK`

**Content** :

```json
[]
```

## Rollup Block by ID

Returns the block specified by :id URL parameter

**URL** : `/falafel/rollup/:id`

**Method** : `GET`

**URL Params:**

- `id`: ID of block to be returned

**Example:**

```
GET https://api.aztec.network/aztec-connect-prod/falafel/rollup/33
```

### Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "id": 33,
  "hash": "56c40928e4b30edcec955e54be7a85bcb2a66683ab99d44f03947bb2b5f3684b",
  "numTxs": 1,
  "ethTxHash": "0x626731c6f6c80022ad326a93596007a874d2626ce450063d17ff498ba3169256",
  "proofData": "0000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000350001107608e76a6681a7ee2c262dec3779019e883074439e7726329072244061b8f1b19dee8de86ee5e51bf2cefeac752dd5cba0bca7a5af86883fe54cc077375f60e5904c609a7108f221e502e04d108840cb1e1f8af1ee032bdbeadef29c46f9525c2e654d49c8f1855a74c4ec3f72c2bb96b97d91117e26a313a134d342ed2650367ee1e2f899579b7e1f50ed3bfc6e6b51daa80a94e918fb2944308180c552306fea3c4533c41dc0ad599636e35dfa3f9ad0471713dd45e97944d5a89cdfdde202442a46aff012ff1ddeeb35b22b35517a498d83d0b3b3dfffc817b9d72e3f0202442a46aff012ff1ddeeb35b22b35517a498d83d0b3b3dfffc817b9d72e3f<.......>",
  "dataRoot": "1b19dee8de86ee5e51bf2cefeac752dd5cba0bca7a5af86883fe54cc077375f6",
  "nullifierRoot": "25c2e654d49c8f1855a74c4ec3f72c2bb96b97d91117e26a313a134d342ed265",
  "created": "2022-06-27T08:18:35.412Z",
  "mined": "2022-06-27T08:18:40.000Z",
  "txs": [
    {
      "id": "5d8da46236f3b33d3f1c672167f219aaf2eaa6854eec7b7786ae4eff62ce03ad",
      "proofId": 3
    }
  ]
}
```

### Error Response

**Condition** : If `:id` param is invalid

**Code** : `404 Not Found`

---

&nbsp;

## Transactions

Used to request Aztec txs

### Transaction by ID

Returns a specific Aztec tx, given the URL `:id` param

**URL** : `/falafel/tx/:id`

**Method** : `GET`

**URL Params:**

- `id`: Hash ID of Aztec transaction to be returned

**Example:**

```
GET https://api.aztec.network/aztec-connect-prod/falafel/tx/5d8da46236f3b33d3f1c672167f219aaf2eaa6854eec7b7786ae4eff62ce03ad
```

### Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "id": "5d8da46236f3b33d3f1c672167f219aaf2eaa6854eec7b7786ae4eff62ce03ad",
  "proofId": 3,
  "proofData": "00000000000000000000000000000000000000000000000000000000000000032972ea1122f29feb673a4820b23eb791b63e758d1c165f379787ddb34750816d18572b5cd1901d05dd148745ddfa9645bd35c299449a16e145f4bb12d78ceb791e33da5958997253453c218cd2fe305f6eb7eb10e1b4f7bcb331068159d341a12e3a0a735c3eacb2df4283172161667f....",
  "offchainTxData": "c11fc47744e518d95a3813b25ceb1c02c90272291ec0f3a089103e5797351a746b339b01deaa9ca5d63bfbc295056e57034f199400f5380a1bf0e10d23b89f82c4928f0184d95dc985160a92a26463cc2ae617a91c5ff8fcf8de085666094848b86bbe9532866f30842d2d8a49d046c6117e317a2fdf9e090a17d4408d9974ebe82aa2b4ab53bf8c07c7f842557503e86a01877c20525432ad9e12bc32de66e5cbabcef7877e0686d0318f80210f57886f0396e6c1eda3c8abd4888cff12c741fb306b874b4b0a4476b56a46ec2d55feb03ff12764765e7a1a2db361fb4abd85202f230a3b675d23b5be1cb5d305b91427ceda4c83e5927377337b740ec5dbc521283d786a3d04001a6092cad7b06ab992e2285376d73ca735c6a18fdb9ec72600000000",
  "newNote1": "2972ea1122f29feb673a4820b23eb791b63e758d1c165f379787ddb34750816d",
  "newNote2": "18572b5cd1901d05dd148745ddfa9645bd35c299449a16e145f4bb12d78ceb79",
  "nullifier1": "1e33da5958997253453c218cd2fe305f6eb7eb10e1b4f7bcb331068159d341a1",
  "nullifier2": "2e3a0a735c3eacb2df42831721616674f96ff97d4fafdf3c5010532aedb4ea94",
  "publicInput": "0000000000000000000000000000000000000000000000000000000000000000",
  "publicOutput": "0000000000000000000000000000000000000000000000000000000000000000",
  "inputOwner": "0000000000000000000000000000000000000000000000000000000000000000",
  "block": {
    "id": 33,
    "dataRoot": {
      "type": "Buffer",
      "data": [
        27, 25, 222, 232, 222, 134, 238, 94, 81, 191, 44, 239, 234, 199, 82, ...
      ]
    },
    "created": "2022-06-27T08:18:35.412Z",
    "processRollupCalldata": {
      "type": "Buffer",
      "data": [
        248, 28, 204, 190, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ...
      ]
    },
    "interactionResult": {
      "type": "Buffer",
      "data": [0, 0, 0, 0]
    },
    "ethTxHash": {
      "buffer": {
        "type": "Buffer",
        "data": [
          98, 103, 49, 198, 246, 200, 0, 34, 173, 50, 106, 147, 89, 96, 7, 168,
          116, 210, 98, 108, 228, 80, 6, 61, 23, 255, 73, 139, 163, 22, 146, 86
        ]
      }
    },
    "gasPrice": {
      "type": "Buffer",
      "data": [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 4, 210, 155, 236, 253
      ]
    },
    "gasUsed": 427934,
    "mined": "2022-06-27T08:18:40.000Z",
    "subtreeRoot": {
      "type": "Buffer",
      "data": [
        37, 49, 84, 252, 200, 85, 197, 212, 41, 251, 150, 72, 251, 241, 163,
        103, 28, 55, 98, 190, 223, 16, 200, 134, 244, 212, 219, 183, 219, 197,
        161, 111
      ]
    }
  }
}
```

### Error Response

**Condition** : If `id` param is invalid

**Code** : `404 Not Found`
