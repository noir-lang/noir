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

Performance for 2^19 (small witness generation phase):

- 16 core (not hyperthreads) x86: ~15s.
- 10 core M1 Mac Pro: ~20s.

Linear scaling was observed up to 32 cores.

Witness generation phase is not multithreaded, and an interesting 512k circuit can take ~12s. This results in:

- 16 core (not hyperthreads) x86: ~28s.
- 10 core M1 Mac Pro: ~32s.

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
  vk_as_fields [options]      Return the verifiation key represented as fields elements. Also return the verification key hash.
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

const api = await Barretenberg.new(/* num_threads */ 1);
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
