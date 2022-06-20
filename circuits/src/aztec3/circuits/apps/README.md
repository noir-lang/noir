# aztec3::circuits::apps

All code in this dir is for:
- Demonstrating how our users' (developers) apps will need to be written, in terms of:
    - Public inputs
    - Commitment/nullifier creation (recommendations)
    - Function calls
    - Everything from the 'Noir Examples' section of the Aztec3 book.
- To help illustrate ideas to the Noir team
- To allow test app circuits to be mocked-up as quickly and easily as possible

## Private State Code

This is some complex code, which attempts to abstract-away stuff to do with creating commitments, nullifiers, and Aztec3 public input ABIs. All that, so that test app circuits can be mocked-up without too much faff. Some explanation is needed:

```
contract_factory
   |___private_state_factory 
            |___private_state_vars
            |          |___private_state_var (fr state)
            |          |___private_state_var (mapping state)
            |                     |___private_state_var (fr state)
            |                     |...
            |                     |___private_state_var (fr state)
            |             ___________________|
            |            |                  fr states can creates notes
            |            v
            |___private_state_notes
            |            |___private_state_note
            |                        |___private_state_note_preimage
            |                                       |
            |                                       |___ preimage members are std::optional
            |                                            so that partial commitments can be
            |                                            created in one tx, and completed in
            |                                            a later tx.
            |
            |___commitments
            |___nullifiers      once all notes have been created by the circuit, we can
                                `finalise()` the state_factory. This will figure out whether
                                we need more:
                                  - dummy nullifiers (to use as input_nullifiers to
                                    commitments).
                                Only at this stage are the commitments (and partial commitments)
                                and nullifiers computed.
```

ContractFactory<Composer>
- Composer& composer;
- OracleWrapperInterface<Composer>& oracle;
- const std::string contract_name;
- PrivateStateFactory<Composer> private_state_factory;
- std::map<std::string, FunctionSignature<CT>> function_signatures;
- std::map<std::string, L1FunctionInterface<Composer>> l1_functions;

PrivateStateFactory<Composer>
- Composer& composer;
- OracleWrapperInterface<Composer>& oracle;
- const std::string contract_name;
- fr private_state_counter = 0;
- std::map<std::string, PrivateStateVar<Composer>> private_state_vars;
- std::vector<PrivateStateNote<Composer>> private_state_notes;
- std::vector<fr> commitments;
- std::vector<fr> nullifiers;

PrivateStateVar<Composer>
- PrivateStateFactory<Composer>* state_factory;
- PrivateStateType private_state_type;
- std::string name;
- fr start_slot;
- grumpkin_point slot_point;
- std::map<NativeTypes::fr, PrivateStateVar<Composer>> private_states;
- bool is_mapping = false;
- std::optional<std::vector<std::string>> mapping_key_names = std::nullopt;
- bool is_partial_slot = false;

PrivateStateNote<Composer>
- PrivateStateVar<Composer>& private_state_var;
- PrivateStateNotePreimage<CT> preimage;