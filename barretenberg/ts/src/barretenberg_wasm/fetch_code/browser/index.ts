import barretenbergModule from '../../barretenberg.wasm';
import barretenbergThreadsModule from '../../barretenberg-threads.wasm';

// Annoyingly the wasm declares if it's memory is shared or not. So now we need two wasms if we want to be
// able to fallback on "non shared memory" situations.
export async function fetchCode(multithreaded: boolean) {
  const res = await fetch(multithreaded ? barretenbergThreadsModule : barretenbergModule);
  return await res.arrayBuffer();
}
