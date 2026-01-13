# SSA CLI

The SSA CLI facilitates experimenting with SSA snippets outside the context of a Noir program. For example we can print the Initial SSA (or any intermediate step) of a Noir program from `nargo [compile|execute|interpret]` using the `--show-ssa-pass` option, and then use the SSA CLI to further `transform`, `interpret` or `visualize` it.


## Example

Take this simple Noir program:

```rust
fn main(c: i32, h: i32) {
    assert(c < h);
    for t in -10..40 {
        show(c, h, t);
    }
}

fn show(c: i32, h: i32, t: i32) {
    if t < 0 {
        println(f"{t}: freezing");
    } else if t < c {
        println(f"{t}: cold");
    } else if t < h {
        println(f"{t}: mild");
    } else {
        println(f"{t}: hot");
    }
}
```

Print the initial SSA and save it to a file:
```bash
cargo run -q -p nargo_cli -- compile --silence-warnings --show-ssa-pass Initial \
  | tail -n +2 \
  > example.ssa
```

The `tail` is just to get rid of the "After Initial SSA:" message.

For quick checks we can also just copy-paste some SSA snippet and use e.g. `pbpaste` to pipe it to the SSA CLI.

### Visualize

We can render the Control Flow Graph of the program to a Markdown file which we can preview in an editor (e.g. using the VS Code [Mermaid Markdown extension](https://marketplace.visualstudio.com/items?itemName=bierner.markdown-mermaid)):

```bash
cat example.ssa \
  | cargo run -q -p noir_ssa_cli -- visualize --markdown \
  > example.md
code example.md
```

Or we can go directly to the [Mermaid Live Editor](https:://mermaid.live):

```bash
open https://mermaid.live/view#$(cargo run -q -p noir_ssa_cli -- visualize --source-path example.ssa --url-encode)
```

The result should look like [this](https://mermaid.live/view#pako:eNqNlN2ymjAUhV-FyZXOeJj8kQAXvWj7CL2qdDooUZyjwYkwp63ju3cbAZMoZ2S8CGt9e0nI3pzRuqkUytFm33ys69K00Y-vhY7gOnWrrSmPdbTB0fJQ7jQsft2s67XB8QrPZis8nwdi9Pb2xa6IbxCgSUiTkaZTBvMNNoMcNg-DKKj0QQyfRemq0OH-SLQ81c0HLNz9kWf7I-P-iL8_z6C-8fhoZHw0WPEpI_GNBGKSMCYZaTllpL6RQkwaxqQjnfmGBFqGtJyiM6CzkM5GWvgGB5qHNJ-iBdAipMVIM994bDYy9tQjHfTU0y6h0fJodrrdwyBQt1GobRTbKZ_VM6eeufXstXru1HO3nr9Wn_T1vzu9bvSpNTDSqgLdzUpeyxITWcLNEq9lyYks6WbJz7L6SY9je7i9dmvqm8bvmhw0dtf4E40MWj_L9pQHrR81e3KD1nerPY1BgwFGC7Q1uwrlrenUAh2UgU8p3KLzlS9QW6uDKlAOy6o07wUq9AVqjqX-2TSHocw03bZG-abcn-CuO1Zlq77vSniDdwReiDLfmk63KCfMRqD8jP6gnCU8ZiTNBPwol3yB_gKCs5iljHNKGMkwSdhlgf7Z_8RxiomgLOOYC3llLv8BmsiLzg).

The same can be achieved for a Noir project with the following utility command:

```shell
just visualize-ssa-cfg Initial
```

### Transform

We can experiment with applying various SSA passes with the `transform` command, using the `--ssa-pass` option to specify which passes we want to run. The same pass can appear multiple times.

```bash
cargo run -q -p noir_ssa_cli -- transform --source-path example.ssa --ssa-pass Unrolling
```

Use the `list` command to see the available options for `--ssa-pass`.

### Interpret

The SSA can be evaluated with some inputs using the `interpret` command. The inputs can come from a JSON or TOML file, or CLI options. Notably the variable names have to match the SSA, not the original Noir code.

For example if we look at the `example.ssa` above, we see that the variables are called `v0` and `v1`:

```console
$ head example.ssa
acir(inline) fn main f0 {
  b0(v0: i32, v1: i32):
    v3 = lt v0, v1
    constrain v3 == u1 1
    jmp b1(i32 -10)
  b1(v2: i32):
    v7 = lt v2, i32 40
    jmpif v7 then: b2, else: b3
  b2():
    call f1(v0, v1, v2)
```

We can run the SSA for example by passing values encoded as TOML using the `--input-toml` option:

```console
$ cargo run -q -p noir_ssa_cli -- interpret --source-path example.ssa --input-toml "v0=10; v1=30"
-10: freezing
-9: freezing
...
39: hot
--- Interpreter result:
Ok()
---
```