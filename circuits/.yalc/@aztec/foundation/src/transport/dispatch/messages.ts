export interface RequestMessage<Payload> {
  msgId: number;
  payload: Payload;
}

export interface ResponseMessage<Payload> {
  msgId: number;
  payload?: Payload;
  error?: string;
}

export interface EventMessage<Payload> {
  payload: Payload;
}

export function isEventMessage<Payload>(
  msg: ResponseMessage<Payload> | EventMessage<Payload>,
): msg is EventMessage<Payload> {
  return (msg as ResponseMessage<Payload>).msgId === undefined;
}
