use std::path::Path;
use wasmer::{imports, ChainableNamedResolver, Cranelift, Function, Instance, Value};
use wasmer::{Module, Store};
use wasmer_engine_jit::JIT;
use wasmer_wasi::WasiState;

/// A wrapper around the return value from a WASM call
/// Notice, Option<> is used because not every call returns a value
/// For example, some calls are simply made to free a pointer
/// or manipulate the heap
#[derive(Debug)]
pub struct WASMValue(Option<Value>);

impl WASMValue {
    pub fn value(self) -> Value {
        self.0.expect("expected a `Value`")
    }
    pub fn into_i32(self) -> i32 {
        i32::try_from(self.0.unwrap()).expect("expected an i32 value")
    }
}

pub struct CompiledModule {
    instance: Instance,
}

impl CompiledModule {
    /// Creates an instance from a WASM Module allowing one to call methods on that WASM module
    // one must provide the byte code and the path to cache the module, so that subsequent loads
    // of the same module will be fast
    pub fn new(wasm_byte_code: &[u8], path_to_cache_module: &Path) -> CompiledModule {
        CompiledModule {
            instance: instance_load(wasm_byte_code, path_to_cache_module),
        }
    }

    /// Transfer bytes to WASM heap
    pub fn transfer_to_heap(&mut self, arr: &[u8], offset: usize) {
        let memory = self.instance.exports.get_memory("memory").unwrap();
        for (byte_id, cell) in memory.view::<u8>()[offset..(offset + arr.len())]
            .iter()
            .enumerate()
        {
            cell.set(arr[byte_id]);
        }
    }

    pub fn read_memory(&self, start: usize, end: usize) -> Vec<u8> {
        let memory = self.instance.exports.get_memory("memory").unwrap();

        let mut result = Vec::with_capacity(end - start);

        for cell in memory.view()[start as usize..end].iter() {
            result.push(cell.get());
        }

        result
    }

    pub fn call(&self, name: &str, param: &Value) -> WASMValue {
        self.call_multiple(name, vec![param])
    }
    pub fn call_multiple(&self, name: &str, params: Vec<&Value>) -> WASMValue {
        // We take in a reference to values, since they do not implement Copy.
        // We then clone them inside of this function, so that the API does not have a bunch of Clones everywhere

        let params: Vec<_> = params.into_iter().cloned().collect();
        let func = self.instance.exports.get_function(name).unwrap();
        let option_value = func.call(&params).unwrap().first().cloned();

        WASMValue(option_value)
    }

    /// Creates a pointer and allocates the bytes that the pointer references to, to the heap
    pub fn allocate(&mut self, bytes: &[u8]) -> Value {
        let ptr = self
            .call("wasm_malloc", &Value::I32(bytes.len() as i32))
            .value();

        let i32_bytes = ptr.unwrap_i32().to_be_bytes();
        let u32_bytes = u32::from_be_bytes(i32_bytes);

        self.transfer_to_heap(bytes, u32_bytes as usize);
        ptr
    }

    /// Frees a pointer.
    /// Notice we consume the Value, if you clone the value before passing it to free
    /// It most likely is a bug as you will be using a dangling pointer/ use-after-free
    pub fn free(&mut self, pointer: Value) {
        self.call("wasm_free", &pointer);
    }
}

// Below lies the code to load a module and create an instance
//
fn load_module(wasm_byte_code: &[u8], path_to_cache_module: &Path) -> (Module, Store) {
    use wasmer_cache::{Cache, FileSystemCache, Hash};

    let compiler_config = Cranelift::default();
    let engine = JIT::new(compiler_config).engine();
    let compile_store = Store::new(&engine);

    let headless_engine = JIT::headless().engine();
    let headless_store = Store::new(&headless_engine);

    let cache_key = Hash::generate(wasm_byte_code);

    // Load directory into cache
    let mut fs_cache = FileSystemCache::new(path_to_cache_module).unwrap();

    // Load module; check if it is in the cache
    // If it is not then compile
    // If it is return it
    let module = match unsafe { fs_cache.load(&headless_store, cache_key) } {
        Ok(module) => module,
        Err(_) => {
            println!("Compiling WASM");

            let module = Module::new(&compile_store, wasm_byte_code).unwrap();

            // Store a module into the cache given a key
            fs_cache.store(cache_key, &module).unwrap();
            module
        }
    };

    (module, headless_store)
}

fn instance_load(wasm_byte_code: &[u8], cache_location: &Path) -> Instance {
    let (module, _) = load_module(wasm_byte_code, cache_location);
    Instance::new(&module, &imports! {}).unwrap()
}

#[test]
fn load_simple_add_wasm_file() {
    // We load a wasm file which only contains the following method
    // fn add(a: i32, b: i32) -> i32 { a + b}
    let wasm_bytes = include_bytes!("simple_add.wasm");
    let tmp_dir = tempdir::TempDir::new("temp_directory").unwrap();

    let compiled_mod = CompiledModule::new(wasm_bytes, tmp_dir.path());

    let a = 2;
    let b = 4;
    let c = a + b;
    let result = compiled_mod.call_multiple("add", vec![&Value::I32(a), &Value::I32(b)]);
    assert_eq!(result.value(), Value::I32(c));
}
