export function findValuesHelper(obj: object, key: string) {
  let list: string[] = [];
  if (!obj) return list;
  if (obj instanceof Array) {
    for (var i in obj) {
      list = list.concat(findValuesHelper(obj[i], key));
    }
    return list;
  }
  if (obj[key]) list.push(obj[key]);

  if (typeof obj == "object" && obj !== null) {
    let children = Object.keys(obj);
    if (children.length > 0) {
      for (let i = 0; i < children.length; i++) {
        list = list.concat(findValuesHelper(obj[children[i]], key));
      }
    }
  }
  return list;
}

export function assertIsError(error: unknown): asserts error is Error {
  if (!(error instanceof Error)) {
    throw error;
  }
}
