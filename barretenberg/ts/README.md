# bb.js

Prover/verifier executable and API for barretenberg. Default cli arguments are appropriate for running within Noir
project structures.

## Performance and limitations

Max circuit size is 2^19 gates (524,288). This is due to the underlying WASM 4GB memory limit. This should improve
with future proving systems, and/or introduction of wasm64.

If running from node, or within browser where you can set shared memory COOP/COEP headers, multithreading is enabled.
Note there are two independent WASM builds, one with threading enabled and one without. This is because the shared
memory flag is set within the WASM itself. If you're running in a context where you can't have shared memory, we want
to fallback to single threaded performance.

The following output is from `bench_acir_tests.sh` script.

Table represents time in ms to build circuit and proof for each test on n threads.
Ignores proving key construction.

```
+--------------------------+------------+---------------+-----------+-----------+-----------+-----------+-----------+
| Test                     | Gate Count | Subgroup Size |         1 |         4 |        16 |        32 |        64 |
+--------------------------+------------+---------------+-----------+-----------+-----------+-----------+-----------+
| sha256                   | 38799      | 65536         |     18764 |      5116 |      1854 |      1524 |      1635 |
| ecdsa_secp256k1          | 41049      | 65536         |     19129 |      5595 |      2255 |      2097 |      2166 |
| ecdsa_secp256r1          | 67331      | 131072        |     38815 |     11257 |      4744 |      3633 |      3702 |
| schnorr                  | 33740      | 65536         |     18649 |      5244 |      2019 |      1498 |      1702 |
| double_verify_proof      | 505513     | 524288        |    149652 |     45702 |     20811 |     16979 |     15679 |
+--------------------------+------------+---------------+-----------+-----------+-----------+-----------+-----------+
```

## Using as a standalone binary

### Installing

To install the package globally for running as a terminal application:

```
npm install -g @aztec/bb.js
```

Assuming `$(npm prefix -g)/bin` is in your `PATH`, you can now run the command `bb.js`.

### Usage

Run `bb.js` for further usage information, you'll see e.g.

```
% bb.js
Usage: bb.js [options] [command]

Options:
  -v, --verbose               enable verbose logging (default: false)
  -h, --help                  display help for command

Commands:
  prove_and_verify [options]  Generate a proof and verify it. Process exits with success or failure code.
  prove [options]             Generate a proof and write it to a file.
  gates [options]             Print gate count to standard output.
  verify [options]            Verify a proof. Process exists with success or failure code.
  contract [options]          Output solidity verification key contract.
  write_vk [options]          Output verification key.
  proof_as_fields [options]   Return the proof as fields elements
  vk_as_fields [options]      Return the verification key represented as field elements. Also return the verification key hash.
  help [command]              display help for command
```

## Using as a library

### Installing

To install as a package to be used as a library:

```
npm install @aztec/bb.js
```

or with yarn

```
yarn add @aztec/bb.js
```

### Usage

To create the API and do a blake2s hash:

```typescript
import { Crs, Barretenberg, RawBuffer } from './index.js';

const api = await Barretenberg.new(/* num_threads */ { threads: 1 });
const input = Buffer.from('hello world!');
const result = await api.blake2s(input);
await api.destroy();
```

All methods are asynchronous. If no threads are specified, will default to number of cores with a maximum of 32.
If `1` is specified, fallback to non multi-threaded wasm that doesn't need shared memory.

See `src/main.ts` for larger example of how to use.

### Browser Context

It's recommended to use a dynamic import. This allows the developer to pick the time at which the package (several MB
in size) is loaded and keeps page load times responsive.

```typescript
const { Barretenberg, RawBuffer, Crs } = await import('@aztec/bb.js');
```

## Development

Create a symlink to the root script `bb.js-dev` in your path. You can now run the current state of the code from
anywhere in your filesystem with no `yarn build` required.

If you change the C++ code run `yarn build:wasm` to rebuild the webassembly.

To run the tests run `yarn test`.

To run a continuous "stress test" run `yarn simple_test` to do 10 full pk/proof/vk iterations. This is useful for
inspecting memory growth as we continuously use the library.
