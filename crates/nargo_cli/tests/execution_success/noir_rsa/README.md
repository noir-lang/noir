# noir-rsa

[![CI][ci-shield-rsa]][ci-url-rsa]
[![CI][ci-shield-dkim]][ci-url-dkim]
[![CI][ci-shield-bigint]][ci-url-biguint]
[![MIT License][license-shield]][license-url]

This library contains an implementation of a RSA signature verify for the Noir language. RSA is one of the most widely used digital signature schemes in Web2 applications, such as DKIM email verification, TLS, message encryption etc

The repo contains 2 crates and 1 example circuit:
- rsa-biguint - Fork of shuklaayush's [Noir BigInt](https://github.com/shuklaayush/noir-bigint/) v0.1.0 with additional functionality
- rsa - RSA signature verification library
- dkim - DKIM email verification circuit

### RSA BigUint
Fork of v0.1.0 of [Noir BigInt](https://github.com/shuklaayush/noir-bigint) with the following updates:
- Updated constants for a max 2048 bit RSA. The BigInt lib only supports 5 limbs
- Added mulmod and powmod functions
- Use unconstrained division for powmod

### RSA
Currently, Noir RSA supports pkcs1v15, sha256, max RSA modulus of 2048 bits and a max exponent value of 17 bits. Typical RSA modulus sizes are 512, 1024 and 2048 bits. And typically, 65537 is used as the public exponent (which is <2^17). 

### DKIM Verification
Verifies a email DKIM signature which signs an email header. The email header comprises of fields such as `From`, `To`, `Subject` and `bh` (hash of the email body encoded in base64). The circuit does the following:
1. Decodes the base64 body hash
2. Hash email body (in bytes) using sha256
3. Compares the hash of the email body with the decoded base64 body hash
4. Hash email header (in bytes) using sha256
5. Verifies the RSA signature matches the public key and the hash of the email header

This repo is under heavy development and should not be used in production.

## Installation
In your Nargo.toml file, add the following dependency:
```
[dependencies]
noir-rsa = { tag = "v0.1.0", git = "https://github.com/SetProtocol/noir-rsa" }
```

## Usage
Running tests

For the RSA crate:
```
cd crates/rsa
nargo test --show-output
```

For the RSA biguint crate:
```
cd crates/rsa-biguint
nargo test --show-output
```

For the DKIM example
```
cd examples/dkim
nargo test --show-output

nargo compile
nargo execute
```

NOTE: The `main` branch only allows RSA bits up to 1024. This is due to proving time being significantly slower if you increase the BigInt limbs to support 2048 bit RSA. Currently proving time for 1024 bits takes ~45 min for the DKIM example. If you need to use a 2048 bit RSA, we support it in the `richard/2048-rsa` [branch](https://github.com/SetProtocol/noir-rsa/tree/richard/2048-rsa)

## Benchmarks
TODO

## Ref
- [Noir BigInt](https://github.com/shuklaayush/noir-bigint/)
- [Halo2 RSA](https://github.com/zkemail/halo2-rsa) 
- [Circom RSA](https://github.com/zkp-application/circom-rsa-verify)
- [Noir RSA Test Generation Scripts](https://github.com/SetProtocol/noir_rsa_scripts)

[ci-shield-dkim]: https://img.shields.io/github/actions/workflow/status/SetProtocol/noir-rsa/test-dkim.yml?branch=main&label=test-dkim
[ci-shield-rsa]: https://img.shields.io/github/actions/workflow/status/SetProtocol/noir-rsa/test-rsa.yml?branch=main&label=test-rsa
[ci-shield-bigint]: https://img.shields.io/github/actions/workflow/status/SetProtocol/noir-rsa/test-rsa-biguint.yml?branch=main&label=test-rsa-biguint
[ci-url-dkim]: https://github.com/SetProtocol/noir-rsa/actions/workflows/test-dkim.yml
[ci-url-rsa]: https://github.com/SetProtocol/noir-rsa/actions/workflows/test-rsa.yml
[ci-url-biguint]: https://github.com/SetProtocol/noir-rsa/actions/workflows/test-rsa-biguint.yml

[license-shield]: https://img.shields.io/badge/License-MIT-green.svg
[license-url]: https://github.com/SetProtocol/noir-rsa/blob/main/LICENSE