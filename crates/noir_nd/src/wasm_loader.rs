use std::path::Path;
use wasmer::{imports, Cranelift, Instance, Value};
use wasmer::{Module, Store};
use wasmer_engine_jit::JIT;

// When allocating and freeing memory, we set the alignment to always be 1 for our purpose
const ALIGNMENT: i32 = 1;

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
        value_to_i32(self.0.unwrap())
    }
}

fn value_to_i32(value: Value) -> i32 {
    i32::try_from(value).expect("expected an i32 value")
}

pub struct Pointer {
    addr: Value,
    size: usize,
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

    /// write to allocated memory
    pub fn write_memory(&mut self, arr: &[u8], ptr: &Pointer) {
        let ptr = value_to_i32(ptr.addr.clone()) as usize;
        let offset = arr.len();

        let memory = self.instance.exports.get_memory("memory").unwrap();
        for (byte_id, cell) in memory.view::<u8>()[ptr..(ptr + offset)].iter().enumerate() {
            cell.set(arr[byte_id]);
        }
    }

    // Note on 64 bit systems, we usize is a u64 which may cause problems if you underflow and
    // try to address WASM32 memory
    pub fn read_memory(&self, ptr: &Pointer, offset: usize) -> Vec<u8> {
        let memory = self.instance.exports.get_memory("memory").unwrap();

        let start = value_to_i32(ptr.addr.clone()) as usize;
        let end = start + offset;
        let mut result = Vec::with_capacity(offset as usize);

        for cell in memory.view()[start..end].iter() {
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
    // malloc takes two arguments ; size and alignment
    pub fn allocate(&mut self, bytes: &[u8]) -> Pointer {
        let addr = self
            .call_multiple(
                "malloc",
                vec![&Value::I32(bytes.len() as i32), &Value::I32(ALIGNMENT)],
            )
            .value();

        // Create a pointer abstraction storing the length
        let ptr = Pointer {
            addr,
            size: bytes.len(),
        };

        self.write_memory(bytes, &ptr);
        ptr
    }

    /// Frees a pointer.
    /// Notice we consume the Value, if you clone the value before passing it to free
    /// It most likely is a bug as you will be using a dangling pointer/ use-after-free
    // Free takes three arguments; address, size and alignment
    pub fn free(&mut self, ptr: Pointer) {
        self.call_multiple(
            "free",
            vec![
                &ptr.addr,
                &Value::I32(ptr.size as i32),
                &Value::I32(ALIGNMENT),
            ],
        );
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

#[test]
fn malloc_alloc_example() {
    let wasm_bytes = include_bytes!("malloc_alloc.wasm");
    let tmp_dir = tempdir::TempDir::new("temp_directory").unwrap();
    let mut compiled_mod = CompiledModule::new(wasm_bytes, tmp_dir.path());

    let bytes_to_alloc = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let ptr = compiled_mod.allocate(&bytes_to_alloc);

    // Read the first 5 bytes of the freshly allocated memory
    let offset = 5;
    let mem = compiled_mod.read_memory(&ptr, offset);
    assert_eq!(mem, vec![0; offset]);

    // Now write to the first five bytes of the freshly allocated memory
    compiled_mod.write_memory(&[1, 2, 3, 4, 5], &ptr);

    // Reading again should show that the memory has changed
    let mem = compiled_mod.read_memory(&ptr, offset);
    assert_eq!(mem, vec![1, 2, 3, 4, 5]);

    // The tests below may fail, depending on the allocator being used
    // We want to check if `free` is working as expected.
    // With the allocator being used, if memory has been freed
    // then when we call `free` again, it will return the same address.
    let old_addr = ptr.addr.clone();
    compiled_mod.free(ptr);

    let ptr = compiled_mod.allocate(&bytes_to_alloc);
    let new_addr = ptr.addr;
    assert_eq!(old_addr, new_addr);
}
