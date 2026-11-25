/-
REVIEWED: trusted entry point. Writes `renderAll` (`AcirLean/Spec/Pin.lean`)
to `templates.golden`.
-/

import AcirLean.Spec.Pin

def main (args : List String) : IO Unit := do
  match args with
  | [path] => IO.FS.writeFile path AcirLean.renderAll
  | _ => IO.print AcirLean.renderAll
