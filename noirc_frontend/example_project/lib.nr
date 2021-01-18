mod foo;
mod world;

fn hello(x : Witness) {
    priv y = x + x;
    priv _z = y + x;
}