//! ACVM execution is independent of the proving backend against which the ACIR code is being proven.
//! However there are currently a few opcodes for which there is currently no rust implementation so we must
//! use the C++ implementations included in Aztec Lab's Barretenberg library.
//!
//! As [`acvm`] includes rust implementations for these opcodes, this module can be removed.

mod barretenberg_structures;
mod pedersen;
mod scalar_mul;
mod schnorr;

use barretenberg_structures::Assignments;

pub(crate) use pedersen::Pedersen;
pub(crate) use scalar_mul::ScalarMul;
pub(crate) use schnorr::SchnorrSig;

/// The number of bytes necessary to store a `FieldElement`.
const FIELD_BYTES: usize = 32;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    FromFeature(#[from] FeatureError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum FeatureError {
    #[error("Trying to call {name} resulted in an error")]
    FunctionCallFailed { name: String, source: wasmer::RuntimeError },
    #[error("Could not find function export named {name}")]
    InvalidExport { name: String, source: wasmer::ExportError },
    #[error("No value available when value was expected")]
    NoValue,
    #[error("Value expected to be i32")]
    InvalidI32,
    #[error("Value {scalar_as_hex} is not a valid grumpkin scalar")]
    InvalidGrumpkinScalar { scalar_as_hex: String },
    #[error("Limb {limb_as_hex} is not less than 2^128")]
    InvalidGrumpkinScalarLimb { limb_as_hex: String },
    #[error("Could not convert value {value} from i32 to u32")]
    InvalidU32 { value: i32, source: std::num::TryFromIntError },
    #[error("Could not convert value {value} from i32 to usize")]
    InvalidUsize { value: i32, source: std::num::TryFromIntError },
    #[error("Value expected to be 0 or 1 representing a boolean")]
    InvalidBool,
}
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub(crate) struct BackendError(#[from] Error);

impl From<FeatureError> for BackendError {
    fn from(value: FeatureError) -> Self {
        value.into()
    }
}

#[derive(Debug)]
pub(crate) struct Barretenberg {
    store: std::cell::RefCell<wasmer::Store>,
    memory: wasmer::Memory,
    instance: wasmer::Instance,
}

use std::cell::RefCell;

use wasmer::{
    imports, Function, FunctionEnv, FunctionEnvMut, Imports, Instance, Memory, MemoryType, Store,
    Value, WasmPtr,
};

/// The number of bytes necessary to represent a pointer to memory inside the wasm.
// pub(super) const POINTER_BYTES: usize = 4;

/// The Barretenberg WASM gives us 1024 bytes of scratch space which we can use without
/// needing to allocate/free it ourselves. This can be useful for when we need to pass in several small variables
/// when calling functions on the wasm, however it's important to not overrun this scratch space as otherwise
/// the written data will begin to corrupt the stack.
///
/// Using this scratch space isn't particularly safe if we have multiple threads interacting with the wasm however,
/// each thread could write to the same pointer address simultaneously.
pub(super) const WASM_SCRATCH_BYTES: usize = 1024;

/// Embed the Barretenberg WASM file
#[derive(rust_embed::RustEmbed)]
#[folder = "$BARRETENBERG_BIN_DIR"]
#[include = "acvm_backend.wasm"]
struct Wasm;

impl Barretenberg {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn new() -> Barretenberg {
        let (instance, memory, store) = instance_load();
        let barretenberg = Barretenberg { memory, instance, store: RefCell::new(store) };
        barretenberg.call_wasi_initialize();
        barretenberg
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) async fn initialize() -> Barretenberg {
        let (instance, memory, store) = instance_load().await;
        let barretenberg = Barretenberg { memory, instance, store: RefCell::new(store) };
        barretenberg.call_wasi_initialize();
        barretenberg
    }
    /// Call initialization function for WASI, to initialize all of the appropriate
    /// globals.
    fn call_wasi_initialize(&self) {
        self.call_multiple("_initialize", vec![])
            .expect("expected call to WASI initialization function to not fail");
    }
}

/// A wrapper around the arguments or return value from a WASM call.
/// Notice, `Option<Value>` is used because not every call returns a value,
/// some calls are simply made to free a pointer or manipulate the heap.
#[derive(Debug, Clone)]
pub(crate) struct WASMValue(Option<Value>);

impl From<usize> for WASMValue {
    fn from(value: usize) -> Self {
        WASMValue(Some(Value::I32(value as i32)))
    }
}

impl From<u32> for WASMValue {
    fn from(value: u32) -> Self {
        WASMValue(Some(Value::I32(value as i32)))
    }
}

impl From<i32> for WASMValue {
    fn from(value: i32) -> Self {
        WASMValue(Some(Value::I32(value)))
    }
}

impl From<Value> for WASMValue {
    fn from(value: Value) -> Self {
        WASMValue(Some(value))
    }
}

impl TryFrom<WASMValue> for bool {
    type Error = FeatureError;

    fn try_from(value: WASMValue) -> Result<Self, Self::Error> {
        match value.try_into()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(FeatureError::InvalidBool),
        }
    }
}

impl TryFrom<WASMValue> for usize {
    type Error = FeatureError;

    fn try_from(value: WASMValue) -> Result<Self, Self::Error> {
        let value: i32 = value.try_into()?;
        value.try_into().map_err(|source| FeatureError::InvalidUsize { value, source })
    }
}

impl TryFrom<WASMValue> for u32 {
    type Error = FeatureError;

    fn try_from(value: WASMValue) -> Result<Self, Self::Error> {
        let value = value.try_into()?;
        u32::try_from(value).map_err(|source| FeatureError::InvalidU32 { value, source })
    }
}

impl TryFrom<WASMValue> for i32 {
    type Error = FeatureError;

    fn try_from(value: WASMValue) -> Result<Self, Self::Error> {
        value.0.map_or(Err(FeatureError::NoValue), |val| val.i32().ok_or(FeatureError::InvalidI32))
    }
}

impl TryFrom<WASMValue> for Value {
    type Error = FeatureError;

    fn try_from(value: WASMValue) -> Result<Self, Self::Error> {
        value.0.ok_or(FeatureError::NoValue)
    }
}

impl Barretenberg {
    /// Transfer bytes to WASM heap
    // TODO: Consider making this Result-returning
    pub(crate) fn transfer_to_heap(&self, data: &[u8], offset: usize) {
        let memory = &self.memory;
        let store = self.store.borrow();
        let memory_view = memory.view(&store);

        memory_view.write(offset as u64, data).unwrap();
    }

    // TODO: Consider making this Result-returning
    pub(crate) fn read_memory<const SIZE: usize>(&self, start: usize) -> [u8; SIZE] {
        self.read_memory_variable_length(start, SIZE)
            .try_into()
            .expect("Read memory should be of the specified length")
    }

    // TODO: Consider making this Result-returning
    pub(crate) fn read_memory_variable_length(&self, offset: usize, length: usize) -> Vec<u8> {
        let memory = &self.memory;
        let store = &self.store.borrow();
        let memory_view = memory.view(&store);

        let mut buf = vec![0; length];

        memory_view.read(offset as u64, &mut buf).unwrap();
        buf
    }

    pub(crate) fn call(&self, name: &str, param: &WASMValue) -> Result<WASMValue, Error> {
        self.call_multiple(name, vec![param])
    }

    pub(crate) fn call_multiple(
        &self,
        name: &str,
        params: Vec<&WASMValue>,
    ) -> Result<WASMValue, Error> {
        // We take in a reference to values, since they do not implement Copy.
        // We then clone them inside of this function, so that the API does not have a bunch of Clones everywhere

        let mut args: Vec<Value> = vec![];
        for param in params.into_iter().cloned() {
            args.push(param.try_into()?);
        }
        let func = self
            .instance
            .exports
            .get_function(name)
            .map_err(|source| FeatureError::InvalidExport { name: name.to_string(), source })?;
        let boxed_value = func.call(&mut self.store.borrow_mut(), &args).map_err(|source| {
            FeatureError::FunctionCallFailed { name: name.to_string(), source }
        })?;
        let option_value = boxed_value.first().cloned();

        Ok(WASMValue(option_value))
    }

    /// Creates a pointer and allocates the bytes that the pointer references to, to the heap
    pub(crate) fn allocate(&self, bytes: &[u8]) -> Result<WASMValue, Error> {
        let ptr: i32 = self.call("bbmalloc", &bytes.len().into())?.try_into()?;

        let i32_bytes = ptr.to_be_bytes();
        let u32_bytes = u32::from_be_bytes(i32_bytes);

        self.transfer_to_heap(bytes, u32_bytes as usize);
        Ok(ptr.into())
    }
}

fn init_memory_and_state() -> (Memory, Store, Imports) {
    let mut store = Store::default();

    let mem_type = MemoryType::new(18, Some(65536), false);
    let memory = Memory::new(&mut store, mem_type).unwrap();

    let function_env = FunctionEnv::new(&mut store, memory.clone());
    let custom_imports = imports! {
        "env" => {
            "logstr" => Function::new_typed_with_env(
                &mut store,
                &function_env,
                logstr,
            ),
            "memory" => memory.clone(),
        },
        "wasi_snapshot_preview1" => {
            "proc_exit" =>  Function::new_typed(&mut store, proc_exit),
            "random_get" => Function::new_typed_with_env(
                &mut store,
                &function_env,
                random_get
            ),
        },
    };

    (memory, store, custom_imports)
}

#[cfg(not(target_arch = "wasm32"))]
fn instance_load() -> (Instance, Memory, Store) {
    use wasmer::Module;

    let (memory, mut store, custom_imports) = init_memory_and_state();

    let module = Module::new(&store, Wasm::get("acvm_backend.wasm").unwrap().data).unwrap();

    (Instance::new(&mut store, &module, &custom_imports).unwrap(), memory, store)
}

#[cfg(target_arch = "wasm32")]
async fn instance_load() -> (Instance, Memory, Store) {
    use js_sys::WebAssembly::{self};
    use wasmer::AsJs;

    let (memory, mut store, custom_imports) = init_memory_and_state();

    let wasm_binary = Wasm::get("acvm_backend.wasm").unwrap().data;

    let js_bytes = unsafe { js_sys::Uint8Array::view(&wasm_binary) };
    let js_module_promise = WebAssembly::compile(&js_bytes);
    let js_module: js_sys::WebAssembly::Module =
        wasm_bindgen_futures::JsFuture::from(js_module_promise).await.unwrap().into();

    let js_instance_promise =
        WebAssembly::instantiate_module(&js_module, &custom_imports.as_jsvalue(&store).into());
    let js_instance = wasm_bindgen_futures::JsFuture::from(js_instance_promise).await.unwrap();
    let module: wasmer::Module = (js_module, wasm_binary).into();
    let instance: wasmer::Instance = Instance::from_jsvalue(&mut store, &module, &js_instance)
        .map_err(|_| "Error while creating BlackBox Functions vendor instance")
        .unwrap();

    (instance, memory, store)
}

fn logstr(mut env: FunctionEnvMut<Memory>, ptr: i32) {
    let (memory, store) = env.data_and_store_mut();
    let memory_view = memory.view(&store);

    let log_str_wasm_ptr: WasmPtr<u8, wasmer::Memory32> = WasmPtr::new(ptr as u32);

    match log_str_wasm_ptr.read_utf8_string_with_nul(&memory_view) {
        Ok(log_string) => println!("{log_string}"),
        Err(err) => println!("Error while reading log string from memory: {err}"),
    };
}

// Based on https://github.com/wasmerio/wasmer/blob/2.3.0/lib/wasi/src/syscalls/mod.rs#L2537
fn random_get(mut env: FunctionEnvMut<Memory>, buf_ptr: i32, buf_len: i32) -> i32 {
    let mut u8_buffer = vec![0; buf_len as usize];
    let res = getrandom::getrandom(&mut u8_buffer);
    match res {
        Ok(()) => {
            let (memory, store) = env.data_and_store_mut();
            let memory_view = memory.view(&store);
            match memory_view.write(buf_ptr as u64, u8_buffer.as_mut_slice()) {
                Ok(_) => {
                    0_i32 // __WASI_ESUCCESS
                }
                Err(_) => {
                    29_i32 // __WASI_EIO
                }
            }
        }
        Err(_) => {
            29_i32 // __WASI_EIO
        }
    }
}

fn proc_exit(_: i32) {
    unimplemented!("proc_exit is not implemented")
}
