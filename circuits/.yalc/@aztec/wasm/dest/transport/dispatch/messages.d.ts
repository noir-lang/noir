/**
 * Represents a transport bus request.
 */
export interface RequestMessage<Payload> {
    /**
     * The message ID.
     */
    msgId: number;
    /**
     * The data.
     */
    payload: Payload;
}
/**
 * Represents a transport bus response.
 */
export interface ResponseMessage<Payload> {
    /**
     * The message ID.
     */
    msgId: number;
    /**
     * The data.
     */
    payload?: Payload;
    /**
     * The error, if any.
     */
    error?: string;
}
/**
 * A message stemming from an event.
 */
export interface EventMessage<Payload> {
    /**
     * The event data.
     */
    payload: Payload;
}
/**
 * Is this an event message?
 * @returns If the msgId was blank.
 */
export declare function isEventMessage<Payload>(msg: ResponseMessage<Payload> | EventMessage<Payload>): msg is EventMessage<Payload>;
//# sourceMappingURL=messages.d.ts.map