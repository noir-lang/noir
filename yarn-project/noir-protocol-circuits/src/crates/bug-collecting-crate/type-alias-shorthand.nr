use X as Y;

struct X{
    x :u32
}

fn main() {
    let z= Y{20};
}

error: cannot find `Y` in this scope 
  ┌─ main.nr:8:12
  │
8 │     let z= Y{20};
  │            - not found in this scope
  │

error: Expected a ; separating these two statements
  ┌─ main.nr:8:13
  │
8 │     let z= Y{20};
  │             -
  │

// Note not using shorthand and doing Y{x : 20} will fix this.