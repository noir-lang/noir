# Lessons

## Proving code is unreachable

- **Failure mode:** Claimed a branch (`resolve_name_in_module`'s "type member followed by a
  trailing segment") was unreachable because a temporary `panic!` never fired across the existing
  `noirc_frontend` test suite.
- **Detection signal:** The maintainer asked "did you test with an actual program that's
  `Type::something::something_else`?" — and a type-position path (`fn f(_x: Foo::C::Bar)`) hit the
  branch immediately. The existing suite simply had no test of that shape.
- **Prevention rule:** "No existing test triggers it" is not a proof of unreachability — it only
  shows the suite lacks coverage. To claim a path is unreachable, construct and run an *actual*
  program that should exercise it (cover every entry point: expression position **and** type
  position, imports, etc.), and only then conclude. When in doubt, assume reachable and handle it.
