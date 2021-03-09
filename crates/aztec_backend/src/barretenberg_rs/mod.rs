///  Import the barretenberg WASM file
pub static WASM: &'static [u8] = include_bytes!("barretenberg.wasm");

pub mod blake2s;
pub mod composer;
mod crs;
pub mod pedersen;
mod pippenger;
pub mod schnorr;

use wasmer::{imports, ChainableNamedResolver, Cranelift, Function, Instance, Value};
use wasmer::{Module, Store};
use wasmer_engine_jit::JIT;
use wasmer_wasi::WasiState;

/// Barretenberg is the low level struct which calls the WASM file
/// This is the bridge between Rust and the WASM which itself is a bridge to the C++ codebase.
pub struct Barretenberg {
    instance: Instance,
}

// XXX: It may be better to use this global mutex, since we do not need to
// keep state around. However, for this we need to make sure
// that mem_free is being called at appropriate times
use once_cell::sync::Lazy;
use std::sync::Mutex;
pub static BARRETENBERG: Lazy<Mutex<Barretenberg>> = Lazy::new(|| Mutex::new(Barretenberg::new()));

/// A wrapper around the return value from a WASM call
/// Notice, Option<> is used because not every call returns a value
/// Some calls are simply made to free a pointer for example
/// Or manipulate the heap
#[derive(Debug)]
pub struct WASMValue(Option<Value>);

impl WASMValue {
    pub fn value(self) -> Value {
        self.0.unwrap()
    }
    pub fn to_i32(self) -> i32 {
        use std::convert::TryFrom;
        i32::try_from(self.0.unwrap()).expect("expected an i32 value")
    }
}

impl Barretenberg {
    /// Transfer bytes to WASM heap
    pub fn transfer_to_heap(&mut self, arr: &[u8], offset: usize) {
        let memory = self.instance.exports.get_memory("memory").unwrap();

        for (byte_id, cell) in memory.view::<u8>()[offset as usize..(offset + arr.len())]
            .iter()
            .enumerate()
        {
            cell.set(arr[byte_id]);
        }
    }
    // XXX: change to read_mem
    pub fn slice_memory(&self, start: usize, end: usize) -> Vec<u8> {
        let memory = self.instance.exports.get_memory("memory").unwrap();

        let mut result = Vec::new();

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

        let params: Vec<_> = params.into_iter().map(|p| p.clone()).collect();
        let func = self.instance.exports.get_function(name).unwrap();
        let option_value = func.call(&params).unwrap().first().cloned();

        WASMValue(option_value)
    }

    /// Creates a pointer and allocates the bytes that the pointer references to, to the heap
    pub fn allocate(&mut self, bytes: &[u8]) -> Value {
        let ptr = self
            .call("bbmalloc", &Value::I32(bytes.len() as i32))
            .value();
        self.transfer_to_heap(bytes, ptr.unwrap_i32() as usize);
        ptr
    }

    /// Frees a pointer.
    /// Notice we consume the Value, if you clone the value before passing it to free
    /// It most likely is a bug
    pub fn free(&mut self, pointer: Value) {
        self.call("bbfree", &pointer);
    }
}

fn load_module() -> (Module, Store) {
    use wasmer_cache::{Cache, FileSystemCache, Hash};

    let compiler_config = Cranelift::default();
    let engine = JIT::new(compiler_config).engine();
    let compile_store = Store::new(&engine);

    let headless_engine = JIT::headless().engine();
    let headless_store = Store::new(&headless_engine);

    let cache_key = Hash::generate(WASM);

    // Load directory into cache
    let mut fs_cache = FileSystemCache::new(mod_cache_location()).unwrap();

    // Load module; check if it is in the cache
    // If it is not then compile
    // If it is return it
    let module = match unsafe { fs_cache.load(&headless_store, cache_key) } {
        Ok(module) => module,
        Err(_) => {
            println!("Compiling WASM... This will take ~3 minutes for the first time, and cached on subsequent runs.");

            let module = Module::new(&compile_store, &WASM).unwrap();

            // Store a module into the cache given a key
            fs_cache.store(cache_key, &module).unwrap();
            module
        }
    };

    (module, headless_store)
}

fn mod_cache_location() -> std::path::PathBuf {
    let mut mod_cache_dir = dirs::home_dir().unwrap();
    mod_cache_dir.push(std::path::Path::new("noir_cache"));
    mod_cache_dir.push(std::path::Path::new("barretenberg_module_cache"));
    mod_cache_dir
}

fn instance_load() -> Instance {
    let (module, store) = load_module();

    let mut wasi_env = WasiState::new("barretenberg").finalize().unwrap();
    let import_object = wasi_env.import_object(&module).unwrap();

    let log_env = Function::new_native_with_env(
        &store,
        Env {
            memory: wasmer::LazyInit::new(),
        },
        logstr,
    );

    let custom_imports = imports! {
        "env" => {
            "logstr" => log_env,
        },
    };

    let res_import = import_object.chain_back(custom_imports);
    Instance::new(&module, &res_import).unwrap()
}

impl Barretenberg {
    pub fn new() -> Barretenberg {
        Barretenberg {
            instance: instance_load(),
        }
    }
}

fn logstr(my_env: &Env, ptr: i32) {
    use std::cell::Cell;
    let memory = my_env.memory.get_ref().unwrap();

    let mut ptr_end = 0;
    for (i, cell) in memory.view::<u8>()[ptr as usize..].iter().enumerate() {
        if cell.get() != 0 {
            ptr_end = i;
        } else {
            break;
        }
    }

    let str_vec: Vec<_> = memory.view()[ptr as usize..=(ptr + ptr_end as i32) as usize]
        .iter()
        .map(|cell: &Cell<u8>| cell.get())
        .collect();

    // Convert the subslice to a `&str`.
    let string = std::str::from_utf8(&str_vec).unwrap();

    // Print it!
    println!("[WASM LOG] {}", string);
}

#[derive(Clone)]
pub struct Env {
    memory: wasmer::LazyInit<wasmer::Memory>,
}

impl wasmer::WasmerEnv for Env {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), wasmer::HostEnvInitError> {
        let memory = instance.exports.get_memory("memory").unwrap();
        self.memory.initialize(memory.clone());
        Ok(())
    }
}
