# Using cvc5 and smt_verification module

Just build with `smt-verification` preset

Now you can import cvc5 using <cvc5/cvc5.h>

# How to use smt_circuit library

## 1. Setting variable names during circuit creation and exporting the circuit.

### There're five new methods inside (for now standard) circuit_builder

- ```set_variable_name(u32 index, str name)``` - assignes a name to a variable. Specifically, binds a name with the first index of an equivalence class.

- ```update_variable_names(u32 idx)``` - in case you've called ```assert_equal``` and ```update_real_variable_indices``` somewhere and you know that two or more variables from the equivalence class have separate names, call this method. Idx is the index of one of the variables of this class. The name of the first variable in class will remain.

- ```finalize_variable_names()``` - in case you don't want to mess with previous method, this one finds all the collisions and removes them.

- ```export_circuit()``` - exports all variables, gates, and assigned names to an msgpack-compatible buffer namely `msgpack::sbuffer`.

To store it on the disk just do 

```cpp
    msgpack::sbuffer buffer = circuit.export_circuit();
    
    std::fstream myfile;
    myfile.open("fname.pack", std::ios::out | std::ios::trunc | std::ios::binary);

    myfile.write(buffer.data(), static_cast<long>(buffer.size()));
    myfile.close();
```

## 2. Symbolic Circuit initialization and term creation

1. First you need to import the circuit from the saved file or from the buffer:

	- `smt_circuit::CircuitSchema c_info = smt_circuit::unpack_from_file(str fname);`

  	- `smt_circuit::CircuitSchema c_info = smt_circuit::unpack_from_buffer(msgpack::sbuffer buf);`


2. Initialize the Solver:

    There's an `smt_solver::SolverConfiguration` structure:
    
    ```cpp
    struct SolverConfiguration {
        bool produce_models;
        uint64_t timeout;
        uint32_t debug;

        bool ff_disjunctive_bit;
        std::string ff_solver;
    };
    ```

    - `produce_models` - should be initialized as `true` if you want to check the values obtained using the solver when the result of the check does not meet your expectations. **All the public variables will be constrained to be equal their real value**.
    - `timeout` - solver timeout in milliseconds
    - `debug` - 0, 1, 2 - defines verbosity level of cvc5
    - `ff_disjunctive_bit` - **Advanced**. Should be used to transform assertions like `(x == 0) | (x == 1)` to `x^2 - x = 0`
    - `ff_solver` - "gb" or "split-gb". **Advanced**. Change the solver approach to solving systems over finite fields.

    There's a `default_solver_config = { true, 0, 0, false, ""}`

    More info on `SolverConfiguration` can be found [here](solver/solver.hpp)

    Now we can initialize the solver

	`smt_solver::Solver s(str modulus, config=default_solver_config, u32 base=16, u32 bvsize=254)`

	- `base` can be any positive integer, it will mostly be 10 or 16, I guess. Default is 16.
    - `bvsize` defines BitVector size in bits, when you use `BVTerm`. Default is 254.

	**!Note that there should be no "0x" part in the modulus hex representation if you put it manually. Otherwise you can use `CircuitSchema.modulus` member that is exported directly from circuit.**

    To verify that the system has solution, just run `Solver::check` method. It will return the boolean.

    `Solver` instance has useful method `print_assertions` that will output all the assertions in kind of human readable format(not SMT2 lang).

    There's also a function `smt_timer(Solver& s, bool mins)` in `barretenberg/smt_verification/util/smt_util.hpp` that will run the `check`, measure the time in minutes/seconds and send it to stdout.
	
3. Initialize the Circuit 

	From now on we will use `smt_terms::STerm` and `smt_terms::Bool` types to operate inside the solver. 

    You can choose the behaviour of symbolic variables by providing the specific type to `STerm` or `Circuit` constructor:

    - `smt_terms::TermType::FFTerm` - symbolic variables that simulate finite field arithmetic. 
    - `smt_terms::TermType::FFITerm` - symbolic variables that simulate integer elements which behave like finite field ones. Useful, when you want to create range constraints. Bad when you try multiplication.
    - `smt_terms::TermType::BVTerm` - symbolic variables that simulate $\pmod{2^n}$ arithmetic. Useful, when you test uint circuits. Supports range constraints and bitwise operations. Doesn't behave like finite field element.

    All these types use different solver engines. The most general one is `FFTerm`.

    `Bool` - simulates the boolean values and mostly will be useful to simulate complex `if` statements if needed.

    Now we can create symbolic circuit
	
    ```smt_circuit::Circuit circuit(CircuitSchema c_info, Solver* s, TermType type, str tag="")```
	
	It will generate all the symbolic values of the circuit wires, add all the gate constrains, create a map `term_name->STerm` and the inverse of it. Where `term_name` is the the name you provided earlier.

    In case you want to create two similar circuits with the same `solver` and `schema`, then you should specify the tag(name) of a circuit. 

	Then you can get the previously named variables via `circuit[name]` or any other variable by `circuit[idx]`.

    There is a method `Circuit::simulate_circuit_eval(vector<fr> w)`. It checks that the evaluation process is correct for this particular witness.

4. Terms creation

    You can initialize symbolic variable via `STerm::Var(str name, &solver, TermType type)` or `STerm::Const(str val, &solver, TermType type, u32 base=16)`

    But also you can use `FFVar(str name, &Solver)` or equivalently via `FFIVar` and `BVVar` so you don't have to mess with types.

    Use `FFConst(str value, &Solver, u32 base=16)`/`FFIConst`/`BVConst` to create constants. However `STerm` is fully arithmetically compatible with `bb::fr` so you can avoid doing this.

    **!Note STerms of distinct types can't be mixed**

	You can add, subtract and multiply these variables(including `+=`, `-=`, etc);
	Also there are two functions:
	- `batch_add(std::vector<STerm>& terms)`
	- `batch_mul(std::vector<STerm>& terms)` 

	to create an addition/multiplication Term in one call

    `FFITerm` also can be used to create range constraints. e.g. `x <= bb::fr(2).pow(10);`

    `BVTerm` can be used to create bitwise constraints. e.g. `STerm y = x^z` or `STerm y = x.rotr(10)`. And range constraints too.

	You can create a constraint `==` or `!=` that will be included directly into solver. e.g. `x == y;` 

    **!Note: In this case it's not comparison operators**

	There is a Bool type:
	- `Bool Bool(STerm t)` or `Bool Bool(bool b, Solver* s)`

	You can `|, &, ==, !=, !` these variables and also `batch_or`, `batch_and` them.
	To create a constraint you should call `Bool::assert_term()` method.
	
	The way I see the use of Bool types is to create terms like `(a == b && c == 1) || (a != b && c == 0)`, `(a!=1)||(b!=2)|(c!=3)` and of course more sophisticated ones.

    **!Note that constraint like `(Bool(STerm a) == Bool(STerm b)).assert_term()`, where a has `FFTerm` type and b has `FFITerm` type, won't work, since their types differ.**
    **!Note `Bool(a == b)` won't work since `a==b` will create an equality constraint as I mentioned earlier and the return type of this operation is `void`.**
5. Post model checking

	After generating all the constrains you should call `bool res = solver.check()` and depending on your goal it could be `true` or `false`.
	
	In case you expected `false` but `true` was returned you can then check what went wrong.
	You should generate an unordered map with `str->term` values and ask the solver to obtain `unoredered_map<str, str> res = solver.model(unordered_map<str, FFTerm> terms)`.
    Or you can provide a vector of terms that you want to check and the return map will contain their symbolic names that are given during initialization. Specifically either it's the name that you set or `var_{i}`.
    
	Now you have the values of the specified terms, which resulted into `true` result. 
    **!Note that the return values are decimal strings/binary strings**, so if you want to use them later you should use `FFConst`, etc.

    Also, there is a header file "barretenberg/common/smt_model.hpp" with two functions:
    - `default_model(verctor<str> special_names, circuit1, circuit2, *solver, fname="witness.out")`
    - `default_model_single(vector<str> special_names, circuit, *solver, fname="witness.out)`

    These functions will write witness variables in c-like array format into file named `fname`.
    The vector of special names is the values that you want ot see in stdout.
 
6. Automated verification of a unique witness

    There's a function `pair<Circuit, Circuit> unique_wintes(CircuitSchema circuit_info, Solver*, TermType type, vector<str> equal)`
    It will create two separate circuits, constrain variables with names from `equal` to be equal acrosss the circuits, and set all the other variables to be not equal at the same time.

    Another one is `pair<Circuit, Circuit> unique_witness_ext(CircuitSchema circuit_info, Solver* s, TermType type, vector<str> equal_variables, vector<str> nequal_variables, vector<str> at_least_one_equal_variable, vector<str> at_least_one_nequal_variable)` that does the same but provides you with more flexible settings.

    The return circuits can be useful, if you want to define some additional constraints, that are not covered by the the above functions.
    You can call `s.check`, `s.model`, `smt_timer` or `default_model` further.

## 3. Simple examples

### Function Equality
```cpp
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(witness_t(&builder, fr::random_element()));
    field_t b(witness_t(&builder, fr::random_element()));
    field_t c = (a + a) / (b + b + b);

    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    builder.set_variable_name(c.witness_index, "c");
    ASSERT_TRUE(CircuitChecker::check(builder));

    auto buf = builder.export_circuit();

    smt_circuit::CircuitSchema circuit_info = smt_circuit::unpack_from_buffer(buf);
    smt_solver::Solver s(circuit_info.modulus);
    smt_circuit::Circuit circuit(circuit_info, &s, smt_terms::TermType::FFTerm);
    smt_terms::STerm a1 = circuit["a"];
    smt_terms::STerm b1 = circuit["b"];
    smt_terms::STerm c1 = circuit["c"];
    smt_terms::STerm two = smt_terms::FFConst("2", &s, 10);
    smt_terms::STerm thr = smt_terms::FFConst("3", &s, 10);
    smt_terms::STerm cr = smt_terms::FFVar("cr", &s);
    cr = (two * a1) / (thr * b1);
    c1 != cr;

    bool res = s.check();
    ASSERT_FALSE(res);
```
### Function Equality with mistake
```cpp
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(witness_t(&builder, fr::random_element()));
    field_t b(witness_t(&builder, fr::random_element()));
    field_t c = (a) / (b + b + b); // mistake was here

    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    builder.set_variable_name(c.witness_index, "c");
    ASSERT_TRUE(CircuitChecker::check(builder));

    auto buf = builder.export_circuit();

    smt_circuit::CircuitSchema circuit_info = smt_circuit::unpack_from_buffer(buf);
    smt_solver::Solver s(circuit_info.modulus);
    smt_circuit::Circuit circuit(circuit_info, &s, smt_terms::TermType::FFTerm);

    smt_terms::STerm a1 = circuit["a"];
    smt_terms::STerm b1 = circuit["b"];
    smt_terms::STerm c1 = circuit["c"];

    smt_terms::STerm two = smt_terms::FFConst("2", &s, 10);
    smt_terms::STerm thr = smt_terms::FFConst("3", &s, 10);
    smt_terms::STerm cr = smt_terms::FFVar("cr", &s);
    cr = (two * a1) / (thr * b1);
    c1 != cr;

    bool res = s.check();
    ASSERT_TRUE(res);

    std::unordered_map<std::string, cvc5::Term> terms({ { "a", a1 }, { "b", b1 }, { "c", c1 }, { "cr", cr } });

    std::unordered_map<std::string, std::string> vals = s.model(terms);

    info("a = ", vals["a"]);
    info("b = ", vals["b"]);
    info("c = ", vals["c"]);
    info("c_res = ", vals["cr"]);
```
### Unique Witness
```cpp
    // Make sure that quadratic polynomial evaluation doesn't have unique
    // witness using unique_witness_ext function
    // Find both roots of a quadratic equation x^2 + a * x + b = s

    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(pub_witness_t(&builder, fr::random_element()));
    field_t b(pub_witness_t(&builder, fr::random_element()));
    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    field_t z(witness_t(&builder, fr::random_element()));
    field_t ev = z * z + a * z + b;
    builder.set_variable_name(z.witness_index, "z");
    builder.set_variable_name(ev.witness_index, "ev");

    auto buf = builder.export_circuit();

    smt_circuit::CircuitSchema circuit_info = smt_circuit::unpack_from_buffer(buf);
    smt_solver::Solver s(circuit_info.modulus);

    std::pair<smt_circuit::Circuit, smt_circuit::Circuit> cirs =
        smt_circuit::unique_witness_ext(circuit_info, &s, smt_terms::TermType::FFTerm, { "ev" }, { "z" });

    bool res = s.check();
    ASSERT_TRUE(res);

    std::unordered_map<std::string, cvc5::Term> terms = { { "z_c1", cirs.first["z"] }, { "z_c2", cirs.second["z"] } };
    std::unordered_map<std::string, std::string> vals = s.model(terms);
    ASSERT_NE(vals["z_c1"], vals["z_c2"]);
```

### Custom model function

```cpp
void model_variables(Circuit<smt_terms::STerm>& c, Solver* s, FFTerm& evaluation)
{
    std::unordered_map<std::string, cvc5::Term> terms;
    terms.insert({ "point", c["point"] });
    terms.insert({ "result", c["result"] });
    terms.insert({ "evaluation", evaluation });

    auto values = s->model(terms);

    info("point = ", values["point"]);
    info("circuit_result = ", values["result"]);
    info("function_evaluation = ", values["evaluation"]);
}
```

More examples can be found in [terms/ffterm.test.cpp](terms/ffterm.test.cpp), [circuit/circuit.test.cpp](circuit/circuit.test.cpp) and [smt_polynomials.test.cpp](smt_polynomials.test.cpp).