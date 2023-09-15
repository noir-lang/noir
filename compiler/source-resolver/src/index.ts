let resolveFunction: Function | null = null;

export let read_file = function (source_id: any): string {

    if (resolveFunction) {

        const result = resolveFunction(source_id);

        if (typeof result === "string") {
            return result;
        } else {
            throw new Error("Noir source resolver function MUST return String synchronously. Are you trying to return anything else, eg. `Promise`?");
        }
    } else {
        throw new Error('Not yet initialized. Use initializeResolver(() => string)');
    }

};

function initialize(noir_resolver: (source_id: String) => string): (source_id: String) => string {

    if (typeof noir_resolver === "function") {
        return noir_resolver;
    } else {
        throw new Error("Provided Noir Resolver is not a function, hint: use function(module_id) => NoirSource as second parameter");
    }
}

export function initializeResolver(resolver: (source_id: String) => string): void {
    resolveFunction = initialize(resolver);
}