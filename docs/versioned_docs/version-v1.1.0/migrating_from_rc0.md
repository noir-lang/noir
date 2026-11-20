---
title: Migrating from v1.0.0-rc.0
description: How to update a Noir project from v1.0.0-rc.0 to the latest Noir 1.0 release, covering installation changes, new compile errors and how to fix them, and tooling changes.
keywords: [Noir, migration, upgrade, nargo, noirup, 1.0, breaking changes]
---

These notes cover what needs to change when moving a project from Noir `v1.0.0-rc.0` to the latest Noir 1.0 toolchain.

The language itself is the same. Most of the work comes from the type checker, which now enforces trait and impl rules more strictly, so some code that compiled on rc.0 is rejected. A smaller number of changes affect how the toolchain is installed, and what some code and tools produce.

To upgrade:

1. **Update `noirup` and install the new toolchain** (P0). An out-of-date `noirup` can fail to install the new releases.
2. **Run `nargo check` on every package**, including your own dependencies, and fix each error using P1. Every item there gives the error you'll see and how to resolve it.
3. **Review the code listed in P2**, where the same source now means something different, and adjust it if you depended on the old behavior.
4. **Update formatting and tests for the tooling changes** in P3: run `nargo fmt`, and update tests that match on exact error messages.
5. **Run your full test suite** on the new toolchain before shipping.

Only changes that need action are listed. Bug fixes that need nothing from you are left out. Sections are ordered by priority.

---

## P0 — Must act on: the upgrade fails without this

### 1. Update `noirup` before installing the new toolchain

Releases from rc.2 onwards ship **only** the `noir-<target>.tar.gz` bundle. The standalone `nargo-<target>.tar.gz` is no longer published ([#13625](https://github.com/noir-lang/noir/pull/13625)). An out-of-date `noirup` can fail to install them.

Update `noirup` itself first, then install:

```sh
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup --version <version>
nargo --version
```

`noirup` received three fixes on 2026-09-17 for this release layout:

- **Bundle detection** ([noirup#69](https://github.com/noir-lang/noirup/pull/69)). Older `noirup` used a `HEAD` probe to check for the `noir-` bundle and fell back to the legacy `nargo-` tarball on *any* probe failure. Where `HEAD` is rejected (seen as a `401` from GitHub's asset CDN on some networks), it falls back to a file that no longer exists and the install fails. The current script downloads the bundle and falls back only on a real `404`.
- **Missing bin directory** ([noirup#63](https://github.com/noir-lang/noirup/pull/63)). Older `noirup` failed to unpack a prebuilt release when `$NARGO_HOME/bin` didn't exist, e.g. a fresh CI runner or container. It still printed `done` and exited 0, so CI could carry on with no `nargo` (or a stale one) on `PATH`.
- **`noir-execute`** ([noirup#65](https://github.com/noir-lang/noirup/pull/65)). Older `noirup` doesn't install `noir-execute` for source builds (`--branch`, `-C`, `--pr`, `--path`), and doesn't remove a stale one, so it can end up mismatched with `nargo`.

Also:

- Pin the `noirup` install script (or its commit) in CI and Docker images, so runners don't keep using a cached old copy. Make CI fail if `nargo --version` doesn't report the expected version.
- Anything that downloads `nargo-<target>.tar.gz` directly, without `noirup`, will 404. Switch it to `noir-<target>.tar.gz`. That bundle also contains `noir-execute`, `noir-inspector` and `noir-profiler`.
- The `acvm` binary (`acvm_cli`) was **deleted** ([#13632](https://github.com/noir-lang/noir/pull/13632)). Use `noir-execute` instead. It can now also take a witness file as input with `--witness-file` ([#13592](https://github.com/noir-lang/noir/pull/13592)).

---

## P1 — New compile errors: code that compiled on rc.0 may be rejected

The trait solver was largely reworked between rc.4 and `master`. The changes below **reject** code that rc.0 accepted. Run `nargo check` on every package (including dependencies) first, and work through the errors using the notes below.

1. **Overlapping impls are detected as rustc does it** ([#14085](https://github.com/noir-lang/noir/pull/14085), [#14099](https://github.com/noir-lang/noir/pull/14099)).

   Two impls of the same trait now conflict if *any* type could satisfy both, even if no type in the program currently does. This covers:
   - two blanket impls whose `where` clauses could both hold for one type;
   - a generic impl for `Wrap<U>` alongside a blanket impl that some `Wrap<_>` satisfies.

   ```noir
   impl<T> Validate for T where T: Bounded { ... }
   impl<T> Validate for T where T: Audited { ... }   // error: overlaps (a type could be both)

   impl Small for Wrap<u8> {}
   impl<T> Checked for T where T: Small { ... }
   impl<U> Checked for Wrap<U> { ... }                // error: overlaps for Wrap<u8>
   ```

   **Fix:** Noir has no negative bounds, so the impls have to be restructured so they can't apply to the same type:
   - Replace one blanket impl with concrete impls for the types that actually need it, e.g. `impl Validate for Amount { ... }`.
   - Or merge the two blanket impls into one, behind a single trait that captures both behaviors.
   - For the generic-wrapper case, implement `Checked` for the specific `Wrap<...>` types that don't already get it from the blanket impl, instead of for every `Wrap<U>`.

2. **`where` bounds that mention no generics are checked** ([#14029](https://github.com/noir-lang/noir/pull/14029), [#14039](https://github.com/noir-lang/noir/pull/14039), [#14037](https://github.com/noir-lang/noir/pull/14037)).

   A bound such as `where Field: Marker` or `where S: Marker` doesn't depend on any of the function's generics, so it is now checked at the declaration. If no impl satisfies it, it is an error, even if the function is never called. This applies to free functions, `main`, and inherent and trait methods.

   **Fix:** If the bound is meant to hold, add the missing impl (`impl Marker for S {}`). If it isn't needed, remove it: a bound with no generics never constrains callers, so removing it doesn't change the function's signature for anyone calling it.

3. **Method calls that several inherent impls could serve are ambiguous** ([#14015](https://github.com/noir-lang/noir/pull/14015), [#14036](https://github.com/noir-lang/noir/pull/14036)).

   When a struct has inherent methods with the same name in impls for different instantiations, and the receiver's generics aren't known yet at the call, the call is now rejected with "Multiple applicable methods named `m` in scope". rc.0 silently picked one of the impls.

   ```noir
   struct S<T> { v: T }
   impl S<u8>  { fn m(self) -> Field { 8 } }
   impl S<u16> { fn m(self) -> Field { 16 } }

   let s = S { v: 5 };   // T not known yet
   s.m();                 // error
   S::<_>::m(s);          // error
   ```

   **Fix:** Make the receiver's type known before the call, e.g. `let s: S<u8> = S { v: 5 };`, `S::<u8> { v: 5 }`, or `S::<u8>::m(s)`. Check which impl rc.0 was picking: it may not be the one you intended.

4. **Impl methods may not require more than the trait method** ([#13655](https://github.com/noir-lang/noir/pull/13655)).

   An impl method whose `where` clause equates two bounds' associated items, when the trait method doesn't, is rejected as "impl has stricter requirements than trait":

   ```noir
   trait Foo { fn foo<B, C>() where B: Bar, C: Bar; }
   impl Foo for Field {
       fn foo<B, C>() where B: Bar, C: Bar<N = <B as Bar>::N> {}   // error
   }
   ```

   **Fix:** Either add the same requirement to the trait method's declaration (this then applies to every impl and every caller), or remove it from the impl. For associated constants, the relationship can be enforced in the body instead, e.g. `assert_eq(<B as Bar>::N, <C as Bar>::N)`.

5. **Numeric generics must divide exactly when solved** ([#13861](https://github.com/noir-lang/noir/pull/13861)).

   rc.0 solved `A * 2 = 5` as `A = 2` by truncating the division. Now there is no solution, so the call fails with "Type annotation needed":

   ```noir
   fn make<let A: u32>() -> [Field; A * 2] { [0; A * 2] }
   let x: [Field; 5] = make();   // error: no u32 A satisfies A * 2 = 5
   ```

   **Fix:** Use a length the expression can actually produce (`[Field; 4]` or `[Field; 6]` here), or pass the generic explicitly (`make::<2>()`) and update the annotated type to match. Any code that relied on the truncated solution had a length mismatch, so check what it was doing.

6. **A trait listed as its own parent is a dependency cycle** ([#13934](https://github.com/noir-lang/noir/pull/13934)).

   `trait A: A {}` or `trait A: A + Bar {}` now reports "Dependency cycle found".

   **Fix:** Remove `A` from its own supertrait list. If the intent was to require another trait, name that trait instead.

7. **Stricter rules for unconstrained functions inside unconstrained-call arguments** ([#13705](https://github.com/noir-lang/noir/pull/13705), [#13785](https://github.com/noir-lang/noir/pull/13785)).

   - An unconstrained function can be passed where a constrained `fn` is expected only when it is **directly** an argument of an unconstrained call. Assigning it to a constrained `fn` variable or struct field somewhere inside the argument expression is now an error ("Converting an unconstrained fn to a non-unconstrained fn is unsafe"):

     ```noir
     let mut g: fn() -> () = foo;
     unsafe { sink({ g = bar; 0 }) };   // error: `bar` is unconstrained
     ```

     **Fix:** If `g` should hold unconstrained functions, declare it with the type `unconstrained fn() -> ()`. Otherwise, assign a constrained function.

   - Only a lambda written **directly** as an argument of an unconstrained call is compiled as unconstrained. A lambda defined inside that lambda's body, or passed to a constrained method somewhere in the argument list, is constrained. Inside such a lambda, calls to unconstrained functions now report "Call to unconstrained function is unsafe".

     **Fix:** Wrap those calls in `unsafe { ... }` with a `// Safety:` comment, or move the logic into a named `unconstrained fn`.

8. **Binary operators are checked against the operands' resolved types** ([#14073](https://github.com/noir-lang/noir/pull/14073)).

   When one operand's type is inferred, e.g. from a generic function's return type, the operator is checked against the type it resolves to. `zero() + true`, where `zero()` resolves to `bool`, is now "Cannot add a `bool` to a `bool`".

   **Fix:** Use an operator the resolved type supports (`|`, `&` or `^` for `bool`), or annotate the call so it resolves to the type intended, e.g. `zero::<u8>()`.

9. **Impl `where` clauses are checked more thoroughly.** The solver now checks:
   - a trait impl's `where` clause even when the impl is used before it has been fully elaborated ([#13939](https://github.com/noir-lang/noir/pull/13939));
   - a trait's own `where` clause against the associated types an impl chooses ([#13933](https://github.com/noir-lang/noir/pull/13933)).

   Impls that didn't meet these requirements but weren't caught on rc.0 now produce "No matching impl found" errors.

   **Fix:** Add the impl the error names, or add the missing bound to the impl's `where` clause so the requirement is passed on to its users.

---

## P2 — Code whose meaning changed

- **Turbofish through a type alias** ([#13997](https://github.com/noir-lang/noir/pull/13997)). A struct literal or pattern's turbofish on a type alias is now resolved against the alias's own generics, not the aliased struct's. With `type Scaled<let SCALE: u32, let MAX: u32> = Capped<MAX, SCALE>;`, `Scaled::<1000, 8> { v }` now means `Capped<8, 1000>`. If you wrote turbofish arguments in the struct's order to match the old behavior, swap them into the alias's order.

---

## P3 — Tooling changes

- **`nargo fmt`** removes empty `use` lists (it used to turn them into imports) ([#13678](https://github.com/noir-lang/noir/pull/13678)). If CI runs `nargo fmt --check`, run `nargo fmt` and commit the result.
- **Out-of-bounds error messages** in ACIR now report the logical index, e.g. `Index out of bounds, array has size 3, but index was 9` ([#13770](https://github.com/noir-lang/noir/pull/13770), [#13776](https://github.com/noir-lang/noir/pull/13776)). Update any tests that match on exact error strings.
