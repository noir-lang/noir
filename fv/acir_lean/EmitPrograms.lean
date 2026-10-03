/-
REVIEWED: trusted entry point. Writes `renderTestPrograms`
(`AcirLean/Spec/Pin.lean`) to `test_programs.golden`.
-/

import AcirLean.Spec.Pin

def main (args : List String) : IO Unit := do
  match args with
  | [path] => IO.FS.writeFile path AcirLean.renderTestPrograms
  | _ => IO.print AcirLean.renderTestPrograms
