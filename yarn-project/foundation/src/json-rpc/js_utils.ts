// Make sure this property was not inherited

/**
 * Does this own the property?
 * @param obj - An object.
 * @param method - A property name.
 */
export const hasOwnProperty = (obj: any, propertyName: string) =>
  Object.prototype.hasOwnProperty.call(obj, propertyName);

export const assert = (x: any, err: string) => {
  if (!x) {
    throw new Error(err);
  }
};
