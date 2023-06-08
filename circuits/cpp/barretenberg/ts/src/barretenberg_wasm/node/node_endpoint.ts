import { NodeEndpoint } from 'comlink/dist/esm/node-adapter.js';

export function nodeEndpoint(nep: NodeEndpoint) {
  const listeners = new WeakMap();
  return {
    postMessage: nep.postMessage.bind(nep),
    addEventListener: (_: any, eh: any) => {
      const l = (data: any) => {
        if ('handleEvent' in eh) {
          eh.handleEvent({ data });
        } else {
          eh({ data });
        }
      };
      nep.on('message', l);
      listeners.set(eh, l);
    },
    removeEventListener: (_: any, eh: any) => {
      const l = listeners.get(eh);
      if (!l) {
        return;
      }
      nep.off('message', l);
      listeners.delete(eh);
    },
    start: nep.start && nep.start.bind(nep),
  };
}
