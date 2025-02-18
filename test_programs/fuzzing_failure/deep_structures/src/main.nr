struct R { }
struct Q {
    q: bool,
    r: R
}
struct P {
    p: bool,
    q:Q,
}
struct O {
    o: bool,
    p: P,
}
struct N {
    n: bool,
    o: O,
}
struct M {
    m: bool,
    n: N,
}
struct L {
    l: bool,
    m: M,
}
struct K {
    k: bool,
    l: L,
}
struct J {
    j: bool,
    k: K,
}
struct I {
    i: bool,
    j: J,
}
struct H {
    h: bool,
    i: I,
}
struct G {
    g: bool,
    h: H,
}
struct F {
    f: bool,
    g: G,
}
struct E {
    e: bool,
    f: F,
}
struct D {
    d: bool,
    e: E,
}
struct C {
    c: bool,
    d: D,
}
struct B {
    b: bool,
    c: C,
}
struct A {
    a: bool,
    b: B,
}
// This program will fail when all the members of the struct except for a.b.c.d.e.f.g.h.i.j.j are true
#[fuzz]
fn depth(a: A) {
    if a.a {
        if a.b.b {
            if a.b.c.c {
                if a.b.c.d.d {
                    if a.b.c.d.e.e {
                        if a.b.c.d.e.f.f {
                            if a.b.c.d.e.f.g.g {
                                if a.b.c.d.e.f.g.h.h {
                                    if a.b.c.d.e.f.g.h.i.i {
                                        if !a.b.c.d.e.f.g.h.i.j.j {
                                            if a.b.c.d.e.f.g.h.i.j.k.k {
                                                if a.b.c.d.e.f.g.h.i.j.k.l.l {
                                                    if a.b.c.d.e.f.g.h.i.j.k.l.m.m {
                                                        if a.b.c.d.e.f.g.h.i.j.k.l.m.n.n {
                                                            if a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.o {
                                                                if a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.p
                                                                     {
                                                                        assert(false);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }