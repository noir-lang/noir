# bb.js

Prover/verifier executable and API for barretenberg. Default cli arguments are appropriate for running within Noir
project structures.

## Performance and limitations

Max circuit size is 2^19 gates (524,288). This is due to the underlying WASM 4GB memory limit. This should improve
with future proving systems, and/or introduction of wasm64.

If running from terminal, or within browser where you can set shared memory COOP/COEP headers, multithreading is enabled.
Note there are two independent WASM builds, one with threading enabled and one without. This is because the shared
memory flag is set within the WASM itself. If you're running in a context where you can't have shared memory, we want
to fallback to single threaded performance.

Performance for 2^19 (small witness generation phase):

- 16 core (not hyperthreads) x86: ~13s.
- 10 core M1 Mac Pro: ~18s.

Linear scaling was observed up to 32 cores.

Witness generation phase is not multithreaded, and an interesting 512k circuit can take ~12s. This results in:

- 16 core (not hyperthreads) x86: ~26s.
- 10 core M1 Mac Pro: (TBD)

## Using as a standalone binary

### Installing

To install the package globally for running as a terminal application:

```
npm install -g @aztec/bb.js
```

Then should alias `$(npm root -g)/@aztec/bb.js/dest/node/main.js` as `bb.js` (or use that full string).

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

To create a multithreaded version of the API:

```typescript
const api = await newBarretenbergApiAsync(/* num_threads */);
// Use.
const input = Buffer.from('hello world!');
const result = await api.blake2s(input);
await api.destroy();
```

All methods are asynchronous. If no threads are specified, will default to number of cores with a maximum of 16.
If `1` is specified, fallback to non multi-threaded wasm that doesn't need shared memory.

You can also create a synchronous version of the api that also has no multi-threading. This is only useful in the
browser if you don't call any multi-threaded functions. It's probably best to just always use async version of the api
unless you're really trying to avoid the small overhead of worker communication.

```typescript
const api = await newBarretenbergApiSync();
// Use.
const input = Buffer.from('hello world!');
const result = api.blake2s(input);
await api.destroy();
```

See `src/main.ts` for one example of how to use.

## Development

Create a symlink to the root script `bb.js-dev` in your path. You can now run the current state of the code from
anywhere in your filesystem with no `yarn build` required.

If you change the C++ code run `yarn build:wasm`.

To run the tests run `yarn test`.

To run a continuous "stress test" run `yarn simple_test` to do 10 full pk/proof/vk iterations.

To run the same test in the browser run `yarn serve`, navigate to appropriate URL and open the console.

