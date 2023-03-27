export interface DispatchMsg {
  fn: string;
  args: any[];
}

export function createDispatchFn(targetFn: () => any, debug = console.error) {
  return async ({ fn, args }: DispatchMsg) => {
    const target = targetFn();
    debug(`dispatching to ${target}: ${fn}`, args);
    return await target[fn](...args);
  };
}
