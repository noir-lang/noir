let resolveFunction: ((source_id: string) => string) | null = null;

export const read_file = function (source_id: string): string {
  if (resolveFunction) {
    const result = resolveFunction(source_id);

    if (typeof result === 'string') {
      return result;
    } else {
      throw new Error(
        'Noir source resolver function MUST return String synchronously. Are you trying to return anything else, eg. `Promise`?',
      );
    }
  } else {
    throw new Error('Not yet initialized. Use initializeResolver(() => string)');
  }
};

function initialize(noir_resolver: (source_id: string) => string): (source_id: string) => string {
  if (typeof noir_resolver === 'function') {
    return noir_resolver;
  } else {
    throw new Error(
      'Provided Noir Resolver is not a function, hint: use function(module_id) => NoirSource as second parameter',
    );
  }
}

export function initializeResolver(resolver: (source_id: string) => string): void {
  resolveFunction = initialize(resolver);
}
