# Module `std::hash`

## Public exports

 - `use sha256::digest`
 - `use sha256::sha256`
 - `use sha256::sha256_compression`
 - `use sha256::sha256_var`
### Methods

## blake2s

```noir
fn blake2s<let N: u32>(input: [u8; N]) -> [u8; 32]
```

## blake3

```noir
fn blake3<let N: u32>(input: [u8; N]) -> [u8; 32]
```

## pedersen_commitment

```noir
fn pedersen_commitment<let N: u32>(input: [Field; N]) -> EmbeddedCurvePoint
```

## pedersen_hash_with_separator

```noir
fn pedersen_hash_with_separator<let N: u32>(input: [Field; N], separator: u32) -> Field
```

## pedersen_hash

```noir
fn pedersen_hash<let N: u32>(input: [Field; N]) -> Field
```

## hash_to_field

```noir
fn hash_to_field(inputs: [Field]) -> Field
```

## keccak256

```noir
fn keccak256<let N: u32>(input: [u8; N], message_size: u32) -> [u8; 32]
```

## poseidon2_permutation

```noir
fn poseidon2_permutation<let N: u32>(_input: [Field; N], _state_length: u32) -> [Field; N]
```

