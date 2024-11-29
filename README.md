# The Blocksense Noir compiler

An essential component of Blocksense are ZK proofs, the primary technology
which eliminates bad actors from manipulating truth. In order to make it easier
for our ZK engineers to develop this component, we built a PLONKY2 backend for
the Noir programming language. While this backend is not completely stable, it
already serves as a good proof-of-concept. Since our work is public and
open-source, anyone can download it, try it out, and submit feedback.

In addition to the PLONKY2 backend, we have also developed a proof-of-concept
formal verification system for the Noir language. Formal verification is able
to mathematically ensure that all possible scenarios are accounted for. In this
way it covers more possibilities than hand-made tests would and eliminates
entire categories of bugs, rather than addressing cases one by one.

## Formal Verification in Noir

Targeting constraint systems introduces some natural limitations to the
expressivity of Noir. The lack of pointers, random-access to memory, resource
management, multi-threading, global mutable state and other familiar, but
error-prone programming concepts makes the language much more amenable to
formal verification.

Furthermore, the target audience of the language is developers working on
crypto-economic protocols, based on smart contracts and zero-knowledge
circuits. These can be considered [high-integrity software
systems](https://en.wikipedia.org/wiki/High_integrity_software) as the cost of
discovering a code defect in production could be extremely high.

Terms such as [certified programming](http://adam.chlipala.net/cpdt/),
[proof-carrying code](https://en.wikipedia.org/wiki/Proof-carrying_code), and
[formal verification](https://en.wikipedia.org/wiki/Formal_verification) in
general still haven't entered the mainstream, but there are two main successful
schools of thought:

Proof assistants such as Coq and Lean and dependently-typed languages such as
Idris are quite powerful, but they require the programmer to learn a whole new
paradigm for expressing the properties of their algorithms and data structures.
The development cost for proving even basic properties of the certified
programs is quite high.

Languages such as [Ada
SPARK](https://learn.adacore.com/courses/intro-to-spark/chapters/01_Overview.html)
have demonstrated that you can add practical formal verification capabilities
to existing procedural programming languages by restricting the language
features available to the programmer (precisely by eliminating features such as
pointers and global mutable state that are naturally missing from Noir).
 
This is why we chose Verus as the back-end for our prototype when implementing
formal verification in Noir. Its architecture is well-suited for our needs.
This reduces the complexity of incorporating it into Noir while supporting
nearly all the features required. Influenced by languages like Dafny and Ada SPARK,
Verus integrates the Z3 SMT solver, enabling precise reasoning and verification
of logical constraints. By connecting the upstream Noir frontend to Verus via
our compiler backend, we managed to produce a proof-of-concept FV system for
Noir, that you can try today!

### How to install

0. Install dependencies, you'll only need two things: the [nix package manager](https://nixos.org/download/) and [direnv](https://direnv.net/docs/installation.html). They're compatible with most OSes and will **not** collide with your system.

> [!IMPORTANT]
> After installing `direnv` do not forget to [add the hook](https://direnv.net/docs/hook.html)!

1. Clone [our branch](https://github.com/blocksense-network/noir/tree/formal-verification) with SSH:

    ```bash
    git clone git@github.com:blocksense-network/noir.git -b formal-verification
    ```

2. Navigate to the folder `noir`.

    ```bash
    cd noir
    ```

3. Run direnv command:

    ```bash
    direnv allow
    ```

    This should result in a lot of things happening. If not, you haven't [added the direnv hook](https://direnv.net/docs/hook.html)!

> [!WARNING]
> Depending on your `nix` installation, you may get a `Permission denied` error. In that case, it's best to start a superuser shell and continue from there:
> 
> ```bash
> sudo su                      # Start superuser shell
> eval "$(direnv hook bash)"   # Setup the direnv hook
> direnv allow
> ```

4. Test if everything works:

    ```bash
    cargo test formal
    ```

    This will also take a little bit of time, until the project fully compiles.

### Example usage

> [!CAUTION]
> The Noir formal-verifications project is in alpha stage! Expect to find bugs and limitations!

1. Create a new project:

    ```bash
    nargo new my_program
    ```

2. Navigate to the folder:

    ```bash
    cd my_program
    ```

3. Update `src/main.nr` with your favorite text editor to:

    ```noir
    #[requires(x < 100 & 0 < y & y < 100)]
    #[ensures(result >= 5 + x)]
    fn main(x: u32, y: u32) -> pub u32 {
      x + y * 5
    }
    ```

4. Finally, verify the program:

    ```bash
    nargo formal-verify
    ```

### Leveraging the formal verification

We examine the following code snippet:
```noir
fn main(x: i32, y:i32, arr: [u32; 5]) -> pub u32 {
  let z = arithmetic_magic(x, y);
  arr[z]
}

fn arithmetic_magic(x: i32, y: i32) -> i32 {
  (x / 2) + (y / 2)
}
```
Formally verifying it produces an error.

This is due to us not ensuring that `z` stays in bounds of the `arr` array.

Adding an if statement which checks for the aforementioned scenario resolves the error.  
The following formally verifies successfully:
```noir
fn main(x: i32, y:i32, arr: [u32; 5]) -> pub u32 {
  let z = arithmetic_magic(x, y);
  if (z >= 0) & (z < 5) {
    arr[z]
  } else {
    0
  }
}

fn arithmetic_magic(x: i32, y: i32) -> i32 {
  (x / 2) + (y / 2)
}
```

## PLONKY2 backend for Noir

The only system which has been adapted for ACIR is barratenberg, also built by
Aztec Labs. While it is an impressive project, we wanted to experiment with
different proving systems in order to leverage the latest and greatest of ZK
research. This is why we built our PLONKY2 backend for the Noir programming
language.

### What is PLONKY2?

PLONKY2 is a zkSNARK built by [Polygon Labs](https://polygon.technology/), with
efficiency, decomposition and size in mind. Recursive proofs can be generated
faster than other systems. This enables proofs to be split into subproofs and distributed
across hundreds or thousands of machines, and it provides the ability to shrink
proofs down dramatically in seconds.

A simple programming language to write ZK programs, with fast-to-generate,
distributed and small in size proofs gives us the best of both worlds. The
consensus mechanism can be developed and maintained without much difficulty,
while it's execution can be distributed on the blockchain with vast assuredness
of the result's correctness, all for a small cost.

### Installing

0. Install dependencies, you'll only need two things: the [nix package manager](https://nixos.org/download/) and [direnv](https://direnv.net/docs/installation.html). They're compatible with most OSes and will **not** collide with your system.

    <Callout type="warning" emoji="⚠️">
      After installing `direnv` do not forget to [add the hook](https://direnv.net/docs/hook.html)!
    </Callout>

1. Clone [our repository](https://github.com/blocksense-network/noir/) with SSH:

    ```bash
    git clone git@github.com:blocksense-network/noir.git
    ```

2. Navigate to the folder `noir`.

    ```bash
    cd noir
    ```

3. Run direnv command:

    ```bash
    direnv allow
    ```

    <Callout type="warning" emoji="⚠️">
      Depending on your `nix` installation, you may get a `Permission denied` error. In that case, it's best to start a superuser shell and continue from there:
      ```bash
      sudo su                      # Start superuser shell
      eval "$(direnv hook bash)"   # Setup the direnv hook
      direnv allow
      ```
    </Callout>
    This should result in a plethora of things happening in the background and foreground. Sit back, relax, and wait it out. By the end you'll have everything ready to start work.

4. Test if everything works:

    ```bash
    cargo test zk_dungeon
    ```

    This will also take a little bit of time, until the project fully compiles.

### Using

<Callout type="error" emoji="🛑">
  As mentioned, the PLONKY2 Noir backend is still under active development and is **not** stable! Expect to find bugs and limitations!
</Callout>

We're now ready to create our first proof!
1. Create a new project:
    ```bash
    nargo new my_program
    ```
2. Navigate to the folder:
    ```bash
    cd my_program
    ```
3. Update `src/main.nr` with your favorite text editor to:
    ```rust
    fn main(x: pub u64, y: u64) {
        assert(x % y == 0);
    }
    ```
    This program allows one to prove that they know of a private factor `y` of a public integer `x`.
4. Run a small check to generate what you need:
    ```bash
    nargo check
    ```
5. We're almost there, change `Prover.toml` to:

    ```toml
    x = "4611686014132420609"
    y = "2147483647"
    ```

6. Finally, we're ready to start proving:
    ```bash
    nargo prove
    ```
    Congratulations 🎉, you've made your first proof! Now we can verify it:

    ```bash
    nargo verify
    ```

You've now successfully written and proven a Noir program! Feel free to play around, for example, if you change `y` to `3` in `Prover.toml`, you'll get a prove error.

Once you're done, head over to [noir-lang.org](https://noir-lang.org/) and start learning about the language.

## Conclusion

These are our current projects that aim to enhance the Noir programming
language. The formal verification framework aims to allow users to write safer
and more correct programs. The PLONKY2 backend aims at providing an alternative
for a ZK proving system that could have advantages over using barrentenberg.

Built with love by the blocksense.network team.
