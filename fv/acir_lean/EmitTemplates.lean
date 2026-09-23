import AcirLean.Template

def main (args : List String) : IO Unit := do
  match args with
  | [path] => IO.FS.writeFile path AcirLean.renderAll
  | _ => IO.print AcirLean.renderAll
