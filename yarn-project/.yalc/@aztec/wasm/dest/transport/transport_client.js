import { createDebugLogger } from "@aztec/foundation";
import EventEmitter from "events";
import { isEventMessage } from "./dispatch/messages.js";
const debug = createDebugLogger("aztec:transport_client");
/**
 * A TransportClient provides a request/response and event api to a corresponding TransportServer.
 * If `broadcast` is called on TransportServer, TransportClients will emit an `event_msg`.
 * The `request` method will block until a response is returned from the TransportServer's dispatch function.
 * Request multiplexing is supported.
 */
export class TransportClient extends EventEmitter {
  constructor(transportConnect) {
    super();
    this.transportConnect = transportConnect;
    this.msgId = 0;
    this.pendingRequests = [];
  }
  /**
   * Create and register our socket using our Connector.
   */
  async open() {
    this.socket = await this.transportConnect.createSocket();
    this.socket.registerHandler((msg) => this.handleSocketMessage(msg));
  }
  /**
   * Close this and stop listening for messages.
   */
  close() {
    this.socket?.close();
    this.socket = undefined;
    this.removeAllListeners();
  }
  /**
   * Queue a request.
   * @param payload - The request payload.
   * @param transfer - Objects to transfer ownership of.
   * @returns A promise of the query result.
   */
  request(payload, transfer) {
    if (!this.socket) {
      throw new Error("Socket not open.");
    }
    const msgId = this.msgId++;
    const msg = { msgId, payload };
    debug(`->`, msg);
    return new Promise((resolve, reject) => {
      this.pendingRequests.push({ resolve, reject, msgId });
      this.socket.send(msg, transfer).catch(reject);
    });
  }
  /**
   * Handle an incoming socket message.
   * @param msg - The message.
   */
  handleSocketMessage(msg) {
    if (msg === undefined) {
      // The remote socket closed.
      this.close();
      return;
    }
    debug(`<-`, msg);
    if (isEventMessage(msg)) {
      this.emit("event_msg", msg.payload);
      return;
    }
    const reqIndex = this.pendingRequests.findIndex(
      (r) => r.msgId === msg.msgId
    );
    if (reqIndex === -1) {
      return;
    }
    const [pending] = this.pendingRequests.splice(reqIndex, 1);
    if (msg.error) {
      pending.reject(new Error(msg.error));
    } else {
      pending.resolve(msg.payload);
    }
  }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidHJhbnNwb3J0X2NsaWVudC5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy90cmFuc3BvcnQvdHJhbnNwb3J0X2NsaWVudC50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsaUJBQWlCLEVBQUUsTUFBTSxZQUFZLENBQUM7QUFDL0MsT0FBTyxZQUFZLE1BQU0sUUFBUSxDQUFDO0FBQ2xDLE9BQU8sRUFBZ0IsY0FBYyxFQUFtQixNQUFNLHdCQUF3QixDQUFDO0FBSXZGLE1BQU0sS0FBSyxHQUFHLGlCQUFpQixDQUFDLHdCQUF3QixDQUFDLENBQUM7QUFzQjFEOzs7OztHQUtHO0FBQ0gsTUFBTSxPQUFPLGVBQXlCLFNBQVEsWUFBWTtJQUt4RCxZQUFvQixnQkFBMkI7UUFDN0MsS0FBSyxFQUFFLENBQUM7UUFEVSxxQkFBZ0IsR0FBaEIsZ0JBQWdCLENBQVc7UUFKdkMsVUFBSyxHQUFHLENBQUMsQ0FBQztRQUNWLG9CQUFlLEdBQXFCLEVBQUUsQ0FBQztJQUsvQyxDQUFDO0lBRUQ7O09BRUc7SUFDSCxLQUFLLENBQUMsSUFBSTtRQUNSLElBQUksQ0FBQyxNQUFNLEdBQUcsTUFBTSxJQUFJLENBQUMsZ0JBQWdCLENBQUMsWUFBWSxFQUFFLENBQUM7UUFDekQsSUFBSSxDQUFDLE1BQU0sQ0FBQyxlQUFlLENBQUMsR0FBRyxDQUFDLEVBQUUsQ0FBQyxJQUFJLENBQUMsbUJBQW1CLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQztJQUNwRSxDQUFDO0lBRUQ7O09BRUc7SUFDSCxLQUFLO1FBQ0gsSUFBSSxDQUFDLE1BQU0sRUFBRSxLQUFLLEVBQUUsQ0FBQztRQUNyQixJQUFJLENBQUMsTUFBTSxHQUFHLFNBQVMsQ0FBQztRQUN4QixJQUFJLENBQUMsa0JBQWtCLEVBQUUsQ0FBQztJQUM1QixDQUFDO0lBRUQ7Ozs7O09BS0c7SUFDSCxPQUFPLENBQUMsT0FBZ0IsRUFBRSxRQUF5QjtRQUNqRCxJQUFJLENBQUMsSUFBSSxDQUFDLE1BQU0sRUFBRTtZQUNoQixNQUFNLElBQUksS0FBSyxDQUFDLGtCQUFrQixDQUFDLENBQUM7U0FDckM7UUFDRCxNQUFNLEtBQUssR0FBRyxJQUFJLENBQUMsS0FBSyxFQUFFLENBQUM7UUFDM0IsTUFBTSxHQUFHLEdBQUcsRUFBRSxLQUFLLEVBQUUsT0FBTyxFQUFFLENBQUM7UUFDL0IsS0FBSyxDQUFDLElBQUksRUFBRSxHQUFHLENBQUMsQ0FBQztRQUNqQixPQUFPLElBQUksT0FBTyxDQUFNLENBQUMsT0FBTyxFQUFFLE1BQU0sRUFBRSxFQUFFO1lBQzFDLElBQUksQ0FBQyxlQUFlLENBQUMsSUFBSSxDQUFDLEVBQUUsT0FBTyxFQUFFLE1BQU0sRUFBRSxLQUFLLEVBQUUsQ0FBQyxDQUFDO1lBQ3RELElBQUksQ0FBQyxNQUFPLENBQUMsSUFBSSxDQUFDLEdBQUcsRUFBRSxRQUFRLENBQUMsQ0FBQyxLQUFLLENBQUMsTUFBTSxDQUFDLENBQUM7UUFDakQsQ0FBQyxDQUFDLENBQUM7SUFDTCxDQUFDO0lBRUQ7OztPQUdHO0lBQ0ssbUJBQW1CLENBQUMsR0FBaUU7UUFDM0YsSUFBSSxHQUFHLEtBQUssU0FBUyxFQUFFO1lBQ3JCLDRCQUE0QjtZQUM1QixJQUFJLENBQUMsS0FBSyxFQUFFLENBQUM7WUFDYixPQUFPO1NBQ1I7UUFDRCxLQUFLLENBQUMsSUFBSSxFQUFFLEdBQUcsQ0FBQyxDQUFDO1FBQ2pCLElBQUksY0FBYyxDQUFDLEdBQUcsQ0FBQyxFQUFFO1lBQ3ZCLElBQUksQ0FBQyxJQUFJLENBQUMsV0FBVyxFQUFFLEdBQUcsQ0FBQyxPQUFPLENBQUMsQ0FBQztZQUNwQyxPQUFPO1NBQ1I7UUFDRCxNQUFNLFFBQVEsR0FBRyxJQUFJLENBQUMsZUFBZSxDQUFDLFNBQVMsQ0FBQyxDQUFDLENBQUMsRUFBRSxDQUFDLENBQUMsQ0FBQyxLQUFLLEtBQUssR0FBRyxDQUFDLEtBQUssQ0FBQyxDQUFDO1FBQzVFLElBQUksUUFBUSxLQUFLLENBQUMsQ0FBQyxFQUFFO1lBQ25CLE9BQU87U0FDUjtRQUNELE1BQU0sQ0FBQyxPQUFPLENBQUMsR0FBRyxJQUFJLENBQUMsZUFBZSxDQUFDLE1BQU0sQ0FBQyxRQUFRLEVBQUUsQ0FBQyxDQUFDLENBQUM7UUFDM0QsSUFBSSxHQUFHLENBQUMsS0FBSyxFQUFFO1lBQ2IsT0FBTyxDQUFDLE1BQU0sQ0FBQyxJQUFJLEtBQUssQ0FBQyxHQUFHLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQztTQUN0QzthQUFNO1lBQ0wsT0FBTyxDQUFDLE9BQU8sQ0FBQyxHQUFHLENBQUMsT0FBTyxDQUFDLENBQUM7U0FDOUI7SUFDSCxDQUFDO0NBQ0YifQ==
