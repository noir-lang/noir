---
title: Inner Workings of Functions and Macros
sidebar_position: 3
tags: [functions]
---

Below, we go more into depth of what is happening under the hood when you create a function in an Aztec contract and what the attributes are really doing.

If you are looking for a reference of function macros, go [here](../../../../reference/smart_contract_reference/macros.md).

## Private functions #[aztec(private)]

A private function operates on private information, and is executed by the user on their device. Annotate the function with the `#[aztec(private)]` attribute to tell the compiler it's a private function. This will make the [private context](./context.md#the-private-context) available within the function's execution scope. The compiler will create a circuit to define this function.

`#aztec(private)` is just syntactic sugar. At compile time, the Aztec.nr framework inserts code that allows the function to interact with the [kernel](../../circuits/kernels/private_kernel.md).

To help illustrate how this interacts with the internals of Aztec and its kernel circuits, we can take an example private function, and explore what it looks like after Aztec.nr's macro expansion.

#### Before expansion

#include_code simple_macro_example /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

#### After expansion

#include_code simple_macro_example_expanded /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

#### The expansion broken down

Viewing the expanded Aztec contract uncovers a lot about how Aztec contracts interact with the [kernel](../../circuits/kernels/private_kernel.md). To aid with developing intuition, we will break down each inserted line.

**Receiving context from the kernel.**
#include_code context-example-inputs /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

Private function calls are able to interact with each other through orchestration from within the [kernel circuit](../../circuits/kernels/private_kernel.md). The kernel circuit forwards information to each contract function (recall each contract function is a circuit). This information then becomes part of the private context.
For example, within each private function we can access some global variables. To access them we can call on the `context`, e.g. `context.chain_id()`. The value of the chain ID comes from the values passed into the circuit from the kernel.

The kernel checks that all of the values passed to each circuit in a function call are the same.

**Returning the context to the kernel.**
#include_code context-example-return /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

The contract function must return information about the execution back to the kernel. This is done through a rigid structure we call the `PrivateCircuitPublicInputs`.

> _Why is it called the `PrivateCircuitPublicInputs`?_
> When verifying zk programs, return values are not computed at verification runtime, rather expected return values are provided as inputs and checked for correctness. Hence, the return values are considered public inputs.

This structure contains a host of information about the executed program. It will contain any newly created nullifiers, any messages to be sent to l2 and most importantly it will contain the return values of the function.

**Hashing the function inputs.**
#include_code context-example-hasher /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

_What is the hasher and why is it needed?_

Inside the kernel circuits, the inputs to functions are reduced to a single value; the inputs hash. This prevents the need for multiple different kernel circuits; each supporting differing numbers of inputs. The hasher abstraction that allows us to create an array of all of the inputs that can be reduced to a single value.

**Creating the function's context.**
#include_code context-example-context /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

Each Aztec function has access to a [context](context) object. This object, although labelled a global variable, is created locally on a users' device. It is initialized from the inputs provided by the kernel, and a hash of the function's inputs.

#include_code context-example-context-return /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

We use the kernel to pass information between circuits. This means that the return values of functions must also be passed to the kernel (where they can be later passed on to another function).
We achieve this by pushing return values to the execution context, which we then pass to the kernel.

**Making the contract's storage available**
#include_code storage-example-context /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

When a [`Storage` struct](../../../../guides/smart_contracts/writing_contracts/storage) is declared within a contract, the `storage` keyword is made available. As shown in the macro expansion above, this calls the init function on the storage struct with the current function's context.

Any state variables declared in the `Storage` struct can now be accessed as normal struct members.

**Returning the function context to the kernel.**
#include_code context-example-finish /noir-projects/noir-contracts/contracts/docs_example_contract/src/main.nr rust

This function takes the application context, and converts it into the `PrivateCircuitPublicInputs` structure. This structure is then passed to the kernel circuit.

## Unconstrained functions

Unconstrained functions are an underlying part of Noir. In short, they are functions which are not directly constrained and therefore should be seen as un-trusted. That they are un-trusted means that the developer must make sure to constrain their return values when used. Note: Calling an unconstrained function from a private function means that you are injecting unconstrained values.

Defining a function as `unconstrained` tells Aztec to simulate it completely client-side in the [ACIR simulator](../../pxe/acir_simulator.md) without generating proofs. They are useful for extracting information from a user through an [oracle](../oracles).

When an unconstrained function is called, it prompts the ACIR simulator to

1. generate the execution environment
2. execute the function within this environment

To generate the environment, the simulator gets the blockheader from the [PXE database](../../pxe/index.md#database) and passes it along with the contract address to `ViewDataOracle`. This creates a context that simulates the state of the blockchain at a specific block, allowing the unconstrained function to access and interact with blockchain data as it would appear in that block, but without affecting the actual blockchain state.

Once the execution environment is created, `execute_unconstrained_function` is invoked:

#include_code execute_unconstrained_function yarn-project/simulator/src/client/unconstrained_execution.ts typescript

This:

1. Prepares the ACIR for execution
2. Converts `args` into a format suitable for the ACVM (Abstract Circuit Virtual Machine), creating an initial witness (witness = set of inputs required to compute the function). `args` might be an oracle to request a user's balance
3. Executes the function in the ACVM, which involves running the ACIR with the initial witness and the context. If requesting a user's balance, this would query the balance from the PXE database
4. Extracts the return values from the `partialWitness` and decodes them based on the artifact to get the final function output. The [artifact](../../../../reference/smart_contract_reference/contract_artifact.md) is the compiled output of the contract, and has information like the function signature, parameter types, and return types

Beyond using them inside your other functions, they are convenient for providing an interface that reads storage, applies logic and returns values to a UI or test. Below is a snippet from exposing the `balance_of_private` function from a token implementation, which allows a user to easily read their balance, similar to the `balanceOf` function in the ERC20 standard.

#include_code balance_of_private /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

:::info
Note, that unconstrained functions can have access to both public and private data when executed on the user's device. This is possible since it is not actually part of the circuits that are executed in contract execution.
:::

## `Public` Functions #[aztec(public)]

A public function is executed by the sequencer and has access to a state model that is very similar to that of the EVM and Ethereum. Even though they work in an EVM-like model for public transactions, they are able to write data into private storage that can be consumed later by a private function.

:::note
All data inserted into private storage from a public function will be publicly viewable (not private).
:::

To create a public function you can annotate it with the `#[aztec(public)]` attribute. This will make the [public context](./context.md) available within the function's execution scope.

#include_code set_minter /noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

Under the hood:

- Context Creation: The macro inserts code at the beginning of the function to create a`PublicContext` object:
```rust
let mut context = PublicContext::new(inputs);
```
This context provides access to public state and transaction information
- Function Signature Modification: The macro modifies the function signature to include a `PublicContextInputs` parameter:
```rust
fn function_name(inputs: PublicContextInputs, ...other_params) -> ReturnType
```
- Return Type Transformation: For functions that return a value, the macro wraps the return type in a `PublicCircuitPublicInputs` struct:
```rust 
-> protocol_types::abis::public_circuit_public_inputs::PublicCircuitPublicInputs
```
- Storage Access: If the contract has a storage struct defined, the macro inserts code to initialize the storage:
```rust
let storage = Storage::init(&mut context);
```
- Function Body Wrapping: The original function body is wrapped in a new scope that handles the context and return value
- Visibility Control: The function is marked as pub, making it accessible from outside the contract.
- Unconstrained Execution: Public functions are marked as unconstrained, meaning they don't generate proofs and are executed directly by the sequencer.

## Constrained `view` Functions #[aztec(view)]

The `#[aztec(view)]` attribute is used to define constrained view functions in Aztec contracts. These functions are similar to view functions in Solidity, in that they are read-only and do not modify the contract's state. They are similar to the [`unconstrained`](#unconstrained-functions-aztecunconstrained) keyword but are executed in a constrained environment. It is not possible to update state within an `#[aztec(view)]` function.

This means the results of these functions are verifiable and can be trusted, as they are part of the proof generation and verification process. This is unlike unconstrained functions, where results are provided by the PXE and are not verified.

This makes `#[aztec(view)]` functions suitable for critical read-only operations where the integrity of the result is crucial. Unconstrained functions, on the other hand, are executed entirely client-side without generating any proofs. It is better to use `#[aztec(view)]` if the result of the function will be used in another function that will affect state, and they can be used for cross-contract calls.

`#[aztec(view)]` functions can be combined with other Aztec attributes like `#[aztec(private)]` or `#[aztec(public)]`.

## `Initializer` Functions #[aztec(initializer)]

This is used to designate functions as initializers (or constructors) for an Aztec contract. These functions are responsible for setting up the initial state of the contract when it is first deployed. The macro does two important things:

- `assert_initialization_matches_address_preimage(context)`: This checks that the arguments and sender to the initializer match the commitments from the address preimage
- `mark_as_initialized(&mut context)`: This is called at the end of the function to emit the initialization nullifier, marking the contract as fully initialized and ensuring this function cannot be called again

Key things to keep in mind:

- A contract can have multiple initializer functions defined, but only one initializer function should be called for the lifetime of a contract instance
- Other functions in the contract will have an initialization check inserted, ie they cannot be called until the contract is initialized, unless they are marked with [`#[aztec(noinitcheck)])`](#aztecnoinitcheck)

## #[aztec(noinitcheck)]

In normal circumstances, all functions in an Aztec contract (except initializers) have an initialization check inserted at the beginning of the function body. This check ensures that the contract has been initialized before any other function can be called. However, there may be scenarios where you want a function to be callable regardless of the contract's initialization state. This is when you would use `#[aztec(noinitcheck)]`.

When a function is annotated with `#[aztec(noinitcheck)]`:

- The Aztec macro processor skips the [insertion of the initialization check](#initializer-functions-aztecinitializer) for this specific function
- The function can be called at any time, even if the contract hasn't been initialized yet

## `Internal` functions #[aztec(internal)]

This macro inserts a check at the beginning of the function to ensure that the caller is the contract itself. This is done by adding the following assertion:

```rust
assert(context.msg_sender() == context.this_address(), "Function can only be called internally");
```

## Custom notes #[aztec(note)]

The `#[aztec(note)]` attribute is used to define custom note types in Aztec contracts. Learn more about notes [here](../../../concepts/storage/index.md).

When a struct is annotated with `#[aztec(note)]`, the Aztec macro applies a series of transformations and generates implementations to turn it into a note that can be used in contracts to store private data.

1. **NoteInterface Implementation**: The macro automatically implements most methods of the `NoteInterface` trait for the annotated struct. This includes:

   - `serialize_content` and `deserialize_content`
   - `get_header` and `set_header`
   - `get_note_type_id`
   - `compute_note_hiding_point`
   - `to_be_bytes`
   - A `properties` method in the note's implementation

2. **Automatic Header Field**: If the struct doesn't already have a `header` field of type `NoteHeader`, one is automatically created

3. **Note Type ID Generation**: A unique `note_type_id` is automatically computed for the note type using a Keccak hash of the struct name

4. **Serialization and Deserialization**: Methods for converting the note to and from a series of `Field` elements are generated, assuming each field can be converted to/from a `Field`

5. **Property Metadata**: A separate struct is generated to describe the note's fields, which is used for efficient retrieval of note data

6. **Export Information**: The note type and its ID are automatically exported


### Before expansion

Here is how you could define a custom note:

```rust
#[aztec(note)]
struct CustomNote {
    data: Field,
    owner: Address,
}
```

### After expansion

```rust
impl CustomNote {
    fn serialize_content(self: CustomNote) -> [Field; NOTE_SERIALIZED_LEN] {
        [self.data, self.owner.to_field()]
    }

    fn deserialize_content(serialized_note: [Field; NOTE_SERIALIZED_LEN]) -> Self {
        CustomNote {
            data: serialized_note[0] as Field,
            owner: Address::from_field(serialized_note[1]),
            header: NoteHeader::empty()
        }
    }

    fn get_note_type_id() -> Field {
        // Automatically generated unique ID based on Keccak hash of the struct name
        0xd2de93eaab1d59abddf06134e737665f076f556feb7b6d3d72ca557b430b14d2
    }

    fn get_header(note: CustomNote) -> aztec::note::note_header::NoteHeader {
        note.header
    }

    fn set_header(self: &mut CustomNote, header: aztec::note::note_header::NoteHeader) {
        self.header = header;
    }

    fn compute_note_hiding_point(self: CustomNote) -> Point {
        aztec::hash::pedersen_commitment(
            self.serialize_content(), 
            aztec::protocol_types::constants::GENERATOR_INDEX__NOTE_HIDING_POINT
        )
    }

      fn to_be_bytes(self, storage_slot: Field) -> [u8; 128] {
            assert(128 == 2 * 32 + 64, "Note byte length must be equal to (serialized_length * 32) + 64 bytes");
            let serialized_note = self.serialize_content();

            let mut buffer: [u8; 128] = [0; 128];

            let storage_slot_bytes = storage_slot.to_be_bytes(32);
            let note_type_id_bytes = CustomNote::get_note_type_id().to_be_bytes(32);

            for i in 0..32 {
                buffer[i] = storage_slot_bytes[i];
                buffer[32 + i] = note_type_id_bytes[i];
            }

            for i in 0..serialized_note.len() {
                let bytes = serialized_note[i].to_be_bytes(32);
                for j in 0..32 {
                    buffer[64 + i * 32 + j] = bytes[j];
                }
            }
            buffer
        }

    pub fn properties() -> CustomNoteProperties {
        CustomNoteProperties {
            data: aztec::note::note_getter_options::PropertySelector { index: 0, offset: 0, length: 32 },
            owner: aztec::note::note_getter_options::PropertySelector { index: 1, offset: 0, length: 32 }
        }
    }
}

struct CustomNoteProperties {
    data: aztec::note::note_getter_options::PropertySelector,
    owner: aztec::note::note_getter_options::PropertySelector,
}
```
Key things to keep in mind:

- Developers can override any of the auto-generated methods by specifying a note interface
- The note's fields are automatically serialized and deserialized in the order they are defined in the struct

## Storage struct #[aztec(storage)]

The `#[aztec(storage)]` attribute is used to define the storage structure for an Aztec contract.

When a struct is annotated with `#[aztec(storage)]`, the macro does this under the hood:

1. **Context Injection**: injects a `Context` generic parameter into the storage struct and all its fields. This allows the storage to interact with the Aztec context, eg when using `context.msg_sender()`

2. **Storage Implementation Generation**: generates an `impl` block for the storage struct with an `init` function. The developer can override this by implementing a `impl` block themselves

3. **Storage Slot Assignment**: automatically assigns storage slots to each field in the struct based on their serialized length

4. **Storage Layout Generation**: a `StorageLayout` struct and a global variable are generated to export the storage layout information for use in the contract artifact

### Before expansion

```rust
#[aztec(storage)]
struct Storage {
    balance: PublicMutable<Field>,
    owner: PublicMutable<Address>,
    token_map: Map<Address, Field>,
}
```

### After expansion

```rust
struct Storage<Context> {
    balance: PublicMutable<Field, Context>,
    owner: PublicMutable<Address, Context>,
    token_map: Map<Address, Field, Context>,
}

impl<Context> Storage<Context> {
    fn init(context: Context) -> Self {
        Storage {
            balance: PublicMutable::new(context, 1),
            owner: PublicMutable::new(context, 2),
            token_map: Map::new(context, 3, |context, slot| Field::new(context, slot)),
        }
    }
}

struct StorageLayout {
    balance: dep::aztec::prelude::Storable,
    owner: dep::aztec::prelude::Storable,
    token_map: dep::aztec::prelude::Storable,
}

#[abi(storage)]
global CONTRACT_NAME_STORAGE_LAYOUT = StorageLayout {
    balance: dep::aztec::prelude::Storable { slot: 1 },
    owner: dep::aztec::prelude::Storable { slot: 2 },
    token_map: dep::aztec::prelude::Storable { slot: 3 },
};
```

Key things to keep in mind:

- Only one storage struct can be defined per contract
- `Map` types and private `Note` types always occupy a single storage slot

## Further reading
- [How do macros work](./inner_workings.md)
- [Macros reference](../../../../reference/smart_contract_reference/macros.md)


