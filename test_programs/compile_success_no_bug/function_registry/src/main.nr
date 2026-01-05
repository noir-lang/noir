use std::collections::umap::UHashMap;
use std::hash::{BuildHasherDefault, Hasher};

// Key is a contract module, value is an array of function definitions.
comptime mut global REGISTRY: UHashMap<Module, [FunctionDefinition], BuildHasherDefault<DummyHasher>> =
    UHashMap::default();

comptime fn add_to_registry(
    registry: &mut UHashMap<Module, [FunctionDefinition], BuildHasherDefault<DummyHasher>>,
    f: FunctionDefinition,
) {
    let module = f.module();
    let current_functions = registry.get(module);
    let functions_to_insert = if current_functions.is_some() {
        current_functions.unwrap().push_back(f)
    } else {
        #[f]
    };
    registry.insert(module, functions_to_insert);
}

struct DummyHasher {}

impl Hasher for DummyHasher {
    fn finish(self) -> Field {
        0
    }

    fn write(&mut self, _input: Field) {}
}

impl Default for DummyHasher {
    fn default() -> Self {
        DummyHasher {}
    }
}

comptime fn example_attribute(function: FunctionDefinition) {
    add_to_registry(&mut REGISTRY, function);
}

#[example_attribute]
pub fn foo() {}

fn main() {
    comptime {
        let crate_root_module = quote { DummyHasher }.as_type().as_data_type().unwrap().0.module();
        assert(REGISTRY.get(crate_root_module).is_some());
        assert(REGISTRY.get(crate_root_module).unwrap().len() == 1);
        assert(REGISTRY.get(crate_root_module).unwrap()[0].name() == quote { foo });
    }
}
