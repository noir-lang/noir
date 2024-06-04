fn recfunc5(a: Field, b: Field) -> Field {
    a * b
}

fn recfunc4(a: Field, b: Field) -> Field {
    if a == b {
        recfunc5(a, b + 1) * recfunc5(a, b + 2)
    } else {
        recfunc5(a, b + 2) * recfunc5(a, b + 3)
    }
}

fn recfunc3(a: Field, b: Field) -> Field {
    if a == b {
        recfunc5(a, b + 1) * recfunc5(a, b + 2)
    } else {
        recfunc5(a, b + 2) * recfunc5(a, b + 3)
    }
}

fn recfunc2(a: Field, b: Field) -> Field {
    if a == b {
        recfunc3(a, b + 1) * recfunc3(a, b + 2)
    } else {
        recfunc3(a, b + 2) * recfunc3(a, b + 3)
    }
}

fn recfunc1(a: Field, b: Field) -> Field {
    if a == b {
        recfunc2(a, b + 1) * recfunc2(a, b + 2)
    } else {
        recfunc2(a, b + 2) * recfunc2(a, b + 3)
    }
}

fn main(x: Field, y: pub Field) -> pub Field {
    let mut a = x;
    let mut b = y;
    let mut c= 0;
    for _ in 0..1 {
        for _ in 0..1 {
            for _ in 0..1 {
                for _ in 0..1 {
                    for _ in 0..1 {
                        for _ in 0..1 {
                            for _ in 0..1 {
                                for _ in 0..1 {
                                    for _ in 0..1 {
                                        for _ in 0..1 {
                                            for _ in 0..1 {
                                                for _ in 0..1 {
                                                    for _ in 0..1 {
                                                        for _ in 0..1 {
                                                            for _ in 0..1 {
                                                                for _ in 0..1 {
                                                                    for _ in 0..1 {
                                                                        for _ in 0..1 {
                                                                            for _ in 0..1 {
                                                                                for _ in 0..1 {
                                                                                    for _ in 0..1 {
                                                                                        for _ in 0..1 {
                                                                                            for _ in 0..1 {
                                                                                                for _ in 0..1 {
                                                                                                    for _ in 0..1 {
                                                                                                        for _ in 0..1 {
                                                                                                            for _ in 0..1 {
                                                                                                                for _ in 0..1 {
                                                                                                                    for _ in 0..1 {
                                                                                                                        for _ in 0..1 {
                                                                                                                            for _ in 0..1 {
                                                                                                                                for _ in 0..1 {
                                                                                                                                    for _ in 0..1 {
                                                                                                                                        for _ in 0..1 {
                                                                                                                                            for _ in 0..1 {
                                                                                                                                                for _ in 0..1 {
                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                        for _ in 0..1 {
                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                for _ in 0..1 {
                                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                                        for _ in 0..1 {
                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                for _ in 0..1 {
                                                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                                                        for _ in 0..1 {
                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                for _ in 0..1 {
                                                                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                                                                        for _ in 0..1 {
                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                for _ in 0..1 {
                                                                                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                                                                                        for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                                for _ in 0..1 {
                                                                                                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                                                                                                        for _ in 0..1 {
                                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                                                for _ in 0..1 {
                                                                                                                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                                                                                                                        for _ in 0..1 {
                                                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                                                                for _ in 0..1 {
                                                                                                                                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                                                                                                                                        for _ in 0..1 {
                                                                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                                                                                for _ in 0..1 {
                                                                                                                                                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                                                                                                                                                        for _ in 0..1 {
                                                                                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                                                                                                for _ in 0..1 {
                                                                                                                                                                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                                                                                                                                                                        for _ in 0..1 {
                                                                                                                                                                                                                                                                                                            for _ in 0..1 {
                                                                                                                                                                                                                                                                                                                for _ in 0..1 {
                                                                                                                                                                                                                                                                                                                    for _ in 0..1 {
                                                                                                                                                                                                                                                                                                                        a+=1;
                                                                                                                                                                                                                                                                                                                        c+=recfunc1(a,b);
                                                                                                                                                                                                                                                                                                                    }}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}
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
                                                                                                    }
                                                                                                }
    }
    c
}

#[test]
fn test_main() {
    main(1, 2);
}
