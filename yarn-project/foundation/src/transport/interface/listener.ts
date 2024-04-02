import type EventEmitter from 'events';

import { type Socket } from './socket.js';

/**
 * Once opened, an implementation of a TransportListener will emit `new_socket` events as new clients connect.
 * Possible implementations could include MessageChannels or WebSockets.
 */
export interface Listener extends EventEmitter {
  // eslint-disable-next-line jsdoc/require-jsdoc
  open(): void;
  // eslint-disable-next-line jsdoc/require-jsdoc
  close(): void;
  // eslint-disable-next-line jsdoc/require-jsdoc
  on(name: 'new_socket', cb: (client: Socket) => void): this;
}
