fn for_stmt() {
    for elem in self {
        ret &= predicate(elem);
    }
}

fn for_stmt() {
    for i in 0 in (C1 - 1) {
        for _j in 1 in (C1 - i - 1) {
            b *= b;
        }

        z *= if b == 1 { 1 } else { c };

        c *= c;

        t *= if b == 1 { 1 } else { c };

        b = t;
    }
}
