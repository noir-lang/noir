// See https://github.com/DefinitelyTyped/DefinitelyTyped/pull/64936/files
declare namespace jest {
  /**
   * Replaces property on an object with another value.
   *
   * @remarks
   * For mocking functions, and 'get' or 'set' accessors, use `jest.spyOn()` instead.
   */
  function replaceProperty<T extends {}, K extends keyof T>(obj: T, key: K, value: T[K]): ReplaceProperty<T[K]>;
}
