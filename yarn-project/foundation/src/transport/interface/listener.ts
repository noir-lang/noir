import EventEmitter from 'events';

import { Socket } from './socket.js';

/**
 * Once opened, an implementation of a TransportListener will emit `new_socket` events as new clients connect.
 * Possible implementations could include MessageChannels or WebSockets.
 */
export interface Listener extends EventEmitter {
  open(): void;

  close(): void;

  on(name: 'new_socket', cb: (client: Socket) => void): this;
}
