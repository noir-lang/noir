/**
 * Represents a request message.
 * Contains a unique identifier (msgId) and a payload object.
 */
export interface RequestMessage<Payload> {
  /**
   * A unique identifier for a message.
   */
  msgId: number;
  /**
   * The data content carried within a message.
   */
  payload: Payload;
}

/**
 * Represents a structured response message.
 * Contains an identifier to match with the corresponding request.
 */
export interface ResponseMessage<Payload> {
  /**
   * A unique identifier for the message.
   */
  msgId: number;
  /**
   * The data content carried within the message.
   */
  payload?: Payload;
  /**
   * An optional error description in case the response contains an error instead of a payload.
   */
  error?: string;
}

/**
 * Represents an event-based message in a communication system.
 * Contains a payload with the relevant data associated with a specific event occurrence.
 */
export interface EventMessage<Payload> {
  /**
   * The data content associated with a message.
   */
  payload: Payload;
}

/**
 * Determines if the given 'msg' is an EventMessage by checking if its 'msgId' property is undefined.
 * Returns true if the input message is of type EventMessage, otherwise false. This utility function can be used
 * to differentiate between instances of ResponseMessage and EventMessage that share a common Payload type.
 *
 * @param msg - The message object that can be either a ResponseMessage or EventMessage with a specific payload.
 * @returns A boolean value indicating whether the input message is an EventMessage (true) or not (false).
 */
export function isEventMessage<Payload>(
  msg: ResponseMessage<Payload> | EventMessage<Payload>,
): msg is EventMessage<Payload> {
  return (msg as ResponseMessage<Payload>).msgId === undefined;
}
