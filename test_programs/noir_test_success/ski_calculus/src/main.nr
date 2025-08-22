// SKI combinator calculus test

#[derive(Eq)]
pub struct Id {
    inner: u32,
}

// Using the following lambda-calculus to SKI conversions:
// - \x. x = I
// - \x. c = K c (where c does not depend on x)
// - \x. f x = f
// - \x. y z = S (\x. y) (\x. z)
pub enum Node {
    S,
    K,
    I,
    App(Id, Id),

    // NOTE: (+1) and literal u32's are included to easily check the values of
    // lambda-encoded natural numbers. E.g. if THREE is `3` lambda-encoded, then
    // (THREE (+1) 0) will evalaute to the literal `3`.
    // (+1)
    Succ,
    // Literal u32
    Const(u32),
}

impl Eq for Node {
    fn eq(self, other: Node) -> bool {
        match (self, other) {
            (Node::S, Node::S) => true,
            (Node::K, Node::K) => true,
            (Node::I, Node::I) => true,
            (Node::Succ, Node::Succ) => true,
            (Node::Const(x), Node::Const(y)) => x == y,
            (Node::App(f, x), Node::App(g, y)) => { (f == g) & (x == y) },
            _ => false,
        }
    }
}

#[derive(Eq)]
pub struct Arena<let N: u32> {
    inner: BoundedVec<Node, N>,
}

impl<let N: u32> Default for Arena<N> {
    fn default() -> Arena<N> {
        Arena { inner: BoundedVec::new() }
    }
}

struct ShowState<let N: u32> {
    output: BoundedVec<str<1>, N>,
    done: bool,
    needs_open_parens: bool,
    needs_lhs: Option<Id>,
    needs_space: bool,
    needs_rhs: Option<Id>,
    needs_close_parens: bool,
}

impl<let N: u32> From<Node> for ShowState<N> {
    fn from(input: Node) -> ShowState<N> {
        let mut output = ShowState {
            output: BoundedVec::<str<1>, N>::new(),
            done: true,
            needs_open_parens: false,
            needs_lhs: Option::none(),
            needs_space: false,
            needs_rhs: Option::none(),
            needs_close_parens: false,
        };

        match input {
            Node::S => output.output.push("S"),
            Node::K => output.output.push("K"),
            Node::I => output.output.push("I"),
            Node::Succ => {
                output.output.push("(");
                output.output.push("+");
                output.output.push("1");
                output.output.push(")");
            },
            Node::Const(x) => {
                let mut current_num = x;
                if current_num == 0 {
                    output.output.push("0");
                }
                for _ in 0..MAX_NUM_DIGITS {
                    if 0 < current_num {
                        let current_mod10 = current_num % 10;
                        output.output.push(BASE10_DIGITS[current_mod10]);
                        current_num /= 10;
                    }
                }
                if 0 < current_num {
                    let output_strs = output.output;
                    panic(
                        f"Arena::show: {current_num} remaining after {MAX_NUM_DIGITS}: {output_strs}",
                    );
                }
            },
            Node::App(f, x) => {
                output.done = false;
                output.needs_open_parens = true;
                output.needs_lhs = Option::some(f);
                output.needs_space = true;
                output.needs_rhs = Option::some(x);
                output.needs_close_parens = true;
            },
        }
        output
    }
}

impl<let N: u32> ShowState<N> {
    pub fn step(&mut self) -> Option<Id> {
        if self.done {
            Option::none()
        } else {
            if self.needs_open_parens {
                self.output.push("(");
                self.needs_open_parens = false;
                Option::none()
            } else if self.needs_lhs.is_some() {
                let lhs = self.needs_lhs.unwrap();
                self.needs_lhs = Option::none();
                Option::some(lhs)
            } else if self.needs_space {
                self.output.push(" ");
                self.needs_space = false;
                Option::none()
            } else if self.needs_rhs.is_some() {
                let rhs = self.needs_rhs.unwrap();
                self.needs_rhs = Option::none();
                Option::some(rhs)
            } else if self.needs_close_parens {
                self.output.push(")");
                self.needs_close_parens = false;
                Option::none()
            } else {
                self.done = true;
                Option::none()
            }
        }
    }
}

impl<let N: u32> Arena<N> {
    // get Node from Arena or panic if it's missing
    fn get(self, id: Id) -> Node {
        self.inner.get(id.inner)
    }

    // set Node in Arena or panic if it's missing
    fn set(&mut self, id: Id, x: Node) {
        self.inner.set(id.inner, x);
        assert_eq(self.get(id), x, "Arena::set consistency check failed!");
    }

    // push Node onto Arena and return its Id
    fn push(&mut self, x: Node) -> Id {
        let result_id = Id { inner: self.inner.len() };
        self.inner.push(x);
        assert_eq(self.get(result_id), x, "Arena::push consistency check failed!");
        result_id
    }

    // returns "did_step"
    //
    // step : &mut Arena -> Node -> bool
    // step id (S|K|I) = true
    // step id (App f x) = match get f {
    //   I => { set id (get x); true }
    //   App g y => match get g {
    //     K => { set id (get y); true }
    //     App h z => match get h {
    //       S => { set id (App ..); true }
    //       _ => false,
    //     }
    //   }
    //   _ => false,
    // }
    pub fn step(&mut self, id: Id) -> bool {
        match self.get(id) {
            Node::App(f, x) => {
                match self.get(f) {
                    Node::I => {
                        self.set(id, self.get(x));
                        true
                    },
                    Node::Succ => {
                        match self.get(x) {
                            Node::Const(x_value) => {
                                self.set(id, Node::Const(x_value + 1));
                                true
                            },
                            _ => false,
                        }
                    },
                    Node::Const(_x) => false,
                    Node::App(g, y) => {
                        match self.get(g) {
                            Node::K => {
                                self.set(id, self.get(y));
                                true
                            },
                            Node::App(h, z) => {
                                match self.get(h) {
                                    // (((S z) y) x) -> (z x) (y x)
                                    Node::S => {
                                        let fs = self.push(Node::App(z, x));
                                        let xs = self.push(Node::App(y, x));
                                        self.set(id, Node::App(fs, xs));
                                        true
                                    },
                                    _ => false,
                                }
                            },
                            _ => false,
                        }
                    },
                    _ => false,
                }
            },
            _ => false,
        }
    }

    fn eval_once(&mut self) -> u32 {
        let mut new_steps = 0;
        // looping method from stdlib any/map functions
        if std::runtime::is_unconstrained() {
            for i in 0..self.inner.len() {
                let id = Id { inner: i };
                if self.step(id) {
                    new_steps += 1;
                }
            }
        } else {
            for i in 0..N {
                if i < self.inner.len() {
                    let id = Id { inner: i };
                    if self.step(id) {
                        new_steps += 1;
                    }
                }
            }
        }
        new_steps
    }

    // returns (number of steps performed, fully_reduced)
    pub fn eval_n(&mut self, max_steps: u32) -> (u32, bool) {
        let mut current_steps = 0;
        let mut fully_reduced = false;
        for _ in 0..max_steps {
            if !fully_reduced {
                // attempt to evaluate each node once
                let new_steps = self.eval_once();
                // if no new steps, we "exit early"
                fully_reduced |= (new_steps == 0);
                // accumulate new steps
                current_steps += new_steps;
            }
        }
        (current_steps, fully_reduced)
    }

    // TODO: blocked by inability to pass `&mut T` to an unconstrained function taking `T`, when `T` contains enum's
    // https://github.com/noir-lang/noir/issues/7558
    // unconstrained fn find_eval<let M: u32>(self) -> BoundedVec<Id, M> {
    //     let mut output = BoundedVec::<Id, M>::new();
    // }
    //
    // // unconstrained search for reductions to run then assert !self.step(id) for all nodes
    // pub fn eval<let M: u32>(&mut self) -> u32 {
    //     // Safety: all steps are explicitly checked and the check below ensures zero steps remain
    //     let target_ids: BoundedVec<Id, M> = unsafe {
    //         self.find_eval::<M>()
    //     };
    //
    //     // run all of the steps
    //     // looping method from stdlib any/map functions
    //     if std::runtime::is_unconstrained() {
    //         for i in 0..target_ids.len() {
    //             let id = target_ids.get(i);
    //             let _ = self.step(id);
    //         }
    //     } else {
    //         for i in 0..N {
    //             if i < target_ids.len() {
    //                 let id = target_ids.get(i);
    //                 let _ = self.step(id);
    //             }
    //         }
    //     }
    //
    //     // assert that zero steps remain
    //     // looping method from stdlib any/map functions
    //     if std::runtime::is_unconstrained() {
    //         for i in 0..self.inner.len() {
    //             let id = Id { inner: i };
    //             assert(!self.step(id));
    //         }
    //     } else {
    //         for i in 0..N {
    //             if i < self.inner.len() {
    //                 let id = Id { inner: i };
    //                 assert(!self.step(id));
    //             }
    //         }
    //     }
    //
    //     target_ids.len()
    // }

    // eval_n for `expected_steps` and assert that `expected_steps` have
    // resulted in a fully-reduced term
    pub fn assert_eval_steps(&mut self, max_steps: u32, expected_steps: u32) {
        let (num_steps, fully_reduced) = self.eval_n(max_steps);
        assert_eq(num_steps, expected_steps);
        assert(fully_reduced);
    }

    pub unconstrained fn print<let M: u32>(self, id: Id) {
        println("");
        let mut output_strs: BoundedVec<str<1>, M> = BoundedVec::new();
        let mut sub_str = " ";
        self.show(id, &mut output_strs);

        // looping method from stdlib any/map functions
        if std::runtime::is_unconstrained() {
            for i in 0..output_strs.len() {
                sub_str = output_strs.get_unchecked(i);
                print(f"{sub_str}");
            }
        } else {
            for i in 0..N {
                if i < output_strs.len() {
                    sub_str = output_strs.get_unchecked(i);
                    print(f"{sub_str}");
                }
            }
        }
        println("");
    }

    pub unconstrained fn show<let M: u32>(self, id: Id, output: &mut BoundedVec<str<1>, M>) {
        let mut show_states = BoundedVec::<ShowState<M>, M>::new();
        let initial_show_state: ShowState<M> = self.get(id).into();
        show_states.push(initial_show_state);
        for _ in 0..M {
            if show_states.len() != 0 {
                let mut current_show_state = show_states.pop();
                if current_show_state.done {
                    output.extend_from_bounded_vec(current_show_state.output);
                } else {
                    let step_result = current_show_state.step();
                    if step_result.is_some() {
                        let next_show_state: ShowState<M> = self.get(step_result.unwrap()).into();

                        // append output from current_show_state and reset it
                        output.extend_from_bounded_vec(current_show_state.output);
                        current_show_state.output = BoundedVec::<str<1>, M>::new();

                        show_states.push(current_show_state);
                        show_states.push(next_show_state);
                    } else {
                        show_states.push(current_show_state);
                    }
                }
            }
        }
    }

    pub unconstrained fn assert_show<let M: u32, let P: u32>(
        self,
        id: Id,
        expected_output: [str<1>; P],
    ) {
        let mut output = BoundedVec::<str<1>, M>::new();
        self.show(id, &mut output);
        let expected_output = BoundedVec::<str<1>, M>::from(expected_output);
        assert_eq(output, expected_output);
    }

    // I (I (.. I))
    // 1  2  .. n
    fn push_I_n(&mut self, n: u32) -> Id {
        let I_id = self.push(Node::I);
        let mut next_id = I_id;
        for _ in 0..n {
            next_id = self.push(Node::App(I_id, next_id));
        }
        next_id
    }

    // (T & F | (NOT F | F) & T) & T
    fn push_logic_example(&mut self) -> Id {
        let S_id = self.push(Node::S);
        let K_id = self.push(Node::K);
        let I_id = self.push(Node::I);
        // T = K
        let TRUE = K_id;
        // F = SK
        let FALSE = self.push(Node::App(S_id, K_id));
        let KT = self.push(Node::App(K_id, TRUE));
        let KF = self.push(Node::App(K_id, FALSE));
        let SI = self.push(Node::App(S_id, I_id));
        let SS = self.push(Node::App(S_id, S_id));
        let SI_KF = self.push(Node::App(SI, KF));
        let K_KF = self.push(Node::App(K_id, KF));
        let S_SI_KF = self.push(Node::App(S_id, SI_KF));
        // OR = SI(KT)
        let OR = self.push(Node::App(SI, KT));
        // AND = SS(K(KF))
        let AND = self.push(Node::App(SS, K_KF));
        // NOT = S(SI(KF))(KT)
        let NOT = self.push(Node::App(S_SI_KF, KT));
        // AND T
        let AND_T = self.push(Node::App(AND, TRUE));
        // AND T F
        let AND_T_F = self.push(Node::App(AND_T, FALSE));
        // NOT F
        let NOT_F = self.push(Node::App(NOT, FALSE));
        // OR (NOT F)
        let OR_NOT_F = self.push(Node::App(OR, NOT_F));
        // OR (NOT F) F
        let OR_NOT_F_F = self.push(Node::App(OR_NOT_F, FALSE));
        // OR (AND T F)
        let OR_AND_T_F = self.push(Node::App(OR, AND_T_F));
        // (OR (AND T F)) (OR (NOT F) F)
        let OR_AND_T_F_OR_NOT_F_F = self.push(Node::App(OR_AND_T_F, OR_NOT_F_F));
        // AND ((OR (AND T F)) (OR (NOT F) F))
        let AND_OR_AND_T_F_OR_NOT_F_F = self.push(Node::App(AND, OR_AND_T_F_OR_NOT_F_F));
        // AND (OR (AND T F)) (OR (NOT F) F) T
        let AND_OR_AND_T_F_OR_NOT_F_F_T = self.push(Node::App(AND_OR_AND_T_F_OR_NOT_F_F, TRUE));
        AND_OR_AND_T_F_OR_NOT_F_F_T
    }

    // 3
    fn push_numeric_example(&mut self) -> Id {
        let S_id = self.push(Node::S);
        let K_id = self.push(Node::K);
        let I_id = self.push(Node::I);
        // 0 := \f.\x.x
        // 0 := K (\x.x)
        // 0 := K I
        let ZERO = self.push(Node::App(K_id, I_id));

        // SUCC := S (S (K S) K)
        let KS = self.push(Node::App(K_id, S_id));
        let S_KS = self.push(Node::App(S_id, KS));
        let S_KS_K = self.push(Node::App(S_KS, K_id));
        let SUCC = self.push(Node::App(S_id, S_KS_K));
        // ONE := SUCC ZERO
        let ONE = self.push(Node::App(SUCC, ZERO));
        // TWO := SUCC (SUCC ZERO)
        let TWO = self.push(Node::App(SUCC, ONE));
        // THREE := SUCC (SUCC (SUCC ZERO))
        let THREE = self.push(Node::App(SUCC, TWO));
        THREE
    }

    // 2^3
    fn push_pow_example(&mut self) -> Id {
        let S_id = self.push(Node::S);
        let K_id = self.push(Node::K);
        let I_id = self.push(Node::I);
        let SI = self.push(Node::App(S_id, I_id));

        // 0 := \f.\x.x
        // 0 := K (\x.x)
        // 0 := K I
        let ZERO = self.push(Node::App(K_id, I_id));
        // SUCC := S (S (K S) K)
        let KS = self.push(Node::App(K_id, S_id));
        let S_KS = self.push(Node::App(S_id, KS));
        let S_KS_K = self.push(Node::App(S_KS, K_id));
        let SUCC = self.push(Node::App(S_id, S_KS_K));
        // K (S I)
        let K_SI = self.push(Node::App(K_id, SI));
        // S (K (S I))
        let S_K_SI = self.push(Node::App(S_id, K_SI));

        // ONE := SUCC ZERO
        let ONE = self.push(Node::App(SUCC, ZERO));
        // TWO := SUCC (SUCC ZERO)
        let TWO = self.push(Node::App(SUCC, ONE));
        // THREE := SUCC (SUCC (SUCC ZERO))
        let THREE = self.push(Node::App(SUCC, TWO));

        // POW := S (K (S I)) K
        let POW = self.push(Node::App(S_K_SI, K_id));

        let POW_TWO = self.push(Node::App(POW, TWO));
        let POW_TWO_THREE = self.push(Node::App(POW_TWO, THREE));
        POW_TWO_THREE
    }

    // 2+3
    fn push_add_example(&mut self) -> Id {
        let S_id = self.push(Node::S);
        let K_id = self.push(Node::K);
        let I_id = self.push(Node::I);

        // 0 := \f.\x.x
        // 0 := K (\x.x)
        // 0 := K I
        let ZERO = self.push(Node::App(K_id, I_id));
        // SUCC := S (S (K S) K)
        let KS = self.push(Node::App(K_id, S_id));
        let S_KS = self.push(Node::App(S_id, KS));
        let S_KS_K = self.push(Node::App(S_KS, K_id));
        let SUCC = self.push(Node::App(S_id, S_KS_K));

        // ONE := SUCC ZERO
        let ONE = self.push(Node::App(SUCC, ZERO));
        // TWO := SUCC (SUCC ZERO)
        let TWO = self.push(Node::App(SUCC, ONE));
        // THREE := SUCC (SUCC (SUCC ZERO))
        let THREE = self.push(Node::App(SUCC, TWO));

        // PLUS := S (S (K S) (S (K K) (S (K S) (S (K (S (K S))) (S (K K)))))) (K I)
        // K K
        let KK = self.push(Node::App(K_id, K_id));
        // K I
        let KI = self.push(Node::App(K_id, I_id));
        // S (K K)
        let S_KK = self.push(Node::App(S_id, KK));
        // K (S (K S))
        let PLUS_7 = self.push(Node::App(K_id, S_KS));
        // S (K (S (K S)))
        let PLUS_6 = self.push(Node::App(S_id, PLUS_7));
        // (S (K (S (K S)))) (S (K K))
        let PLUS_5 = self.push(Node::App(PLUS_6, S_KK));
        // (S (K S)) (S (K (S (K S))) (S (K K)))
        let PLUS_4 = self.push(Node::App(S_KS, PLUS_5));
        // (S (K K)) (S (K S) (S (K (S (K S))) (S (K K))))
        let PLUS_3 = self.push(Node::App(S_KK, PLUS_4));
        // (S (K S)) (S (K K) (S (K S) (S (K (S (K S))) (S (K K)))))
        let PLUS_2 = self.push(Node::App(S_KS, PLUS_3));
        // S (S (K S) (S (K K) (S (K S) (S (K (S (K S))) (S (K K))))))
        let PLUS_1 = self.push(Node::App(S_id, PLUS_2));
        // PLUS := (S (S (K S) (S (K K) (S (K S) (S (K (S (K S))) (S (K K))))))) (K I)
        let PLUS = self.push(Node::App(PLUS_1, KI));

        // 2 + 3
        let PLUS_TWO = self.push(Node::App(PLUS, TWO));
        let PLUS_TWO_THREE = self.push(Node::App(PLUS_TWO, THREE));
        PLUS_TWO_THREE
    }
}

pub global ARENA_SIZE: u32 = 256;

// "(" + char + " " + ")" + overhead = 5
pub global PRINT_SIZE: u32 = 5 * ARENA_SIZE;

pub global MAX_NUM_DIGITS: u32 = 64;
pub global BASE10_DIGITS: [str<1>; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

fn main() {}

#[test]
fn test_identity() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // I (I (.. I))
    // 1  2  .. 10
    let target_formula = arena.push_I_n(10);

    // normalize arena
    arena.assert_eval_steps(10, 10);

    // ensure it reduced to a single identity function
    assert_eq(arena.get(target_formula), Node::I);

    // check final length
    assert_eq(arena.inner.len(), 11);
}

#[test]
unconstrained fn test_identity_and_show() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // I (I (.. I))
    // 1  2  .. 10
    let target_formula = arena.push_I_n(10);

    // print AST
    arena.assert_show::<ARENA_SIZE, _>(
        target_formula,
        [
            "(", "I", " ", "(", "I", " ", "(", "I", " ", "(", "I", " ", "(", "I", " ", "(", "I",
            " ", "(", "I", " ", "(", "I", " ", "(", "I", " ", "(", "I", " ", "I", ")", ")", ")",
            ")", ")", ")", ")", ")", ")", ")",
        ],
    );

    // normalize arena
    arena.assert_eval_steps(10, 10);

    // print normalized AST
    arena.assert_show::<ARENA_SIZE, _>(target_formula, ["I"]);

    // ensure it reduced to a single identity function
    assert_eq(arena.get(target_formula), Node::I);

    // check final length
    assert_eq(arena.inner.len(), 11);
}

#[test]
fn test_logic() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // (T & F | (NOT F | F) & T) & T
    let target_formula = arena.push_logic_example();

    // normalize arena
    arena.assert_eval_steps(20, 29);

    // ensure result is TRUE
    assert_eq(arena.get(target_formula), Node::K);

    // check final length
    assert_eq(arena.inner.len(), 45);
}

#[test]
unconstrained fn test_logic_and_show() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // (T & F | (NOT F | F) & T) & T
    let target_formula = arena.push_logic_example();

    // print AST
    arena.assert_show::<ARENA_SIZE, _>(
        target_formula,
        [
            "(", "(", "(", "(", "S", " ", "S", ")", " ", "(", "K", " ", "(", "K", " ", "(", "S",
            " ", "K", ")", ")", ")", ")", " ", "(", "(", "(", "(", "S", " ", "I", ")", " ", "(",
            "K", " ", "K", ")", ")", " ", "(", "(", "(", "(", "S", " ", "S", ")", " ", "(", "K",
            " ", "(", "K", " ", "(", "S", " ", "K", ")", ")", ")", ")", " ", "K", ")", " ", "(",
            "S", " ", "K", ")", ")", ")", " ", "(", "(", "(", "(", "S", " ", "I", ")", " ", "(",
            "K", " ", "K", ")", ")", " ", "(", "(", "(", "S", " ", "(", "(", "S", " ", "I", ")",
            " ", "(", "K", " ", "(", "S", " ", "K", ")", ")", ")", ")", " ", "(", "K", " ", "K",
            ")", ")", " ", "(", "S", " ", "K", ")", ")", ")",
        ],
    );

    // normalize arena
    arena.assert_eval_steps(20, 29);

    // print AST
    arena.assert_show::<ARENA_SIZE, _>(target_formula, ["K"]);

    // ensure result is TRUE
    assert_eq(arena.get(target_formula), Node::K);

    // check final length
    assert_eq(arena.inner.len(), 45);
}

#[test]
fn test_numeric() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // 3
    let target_formula = arena.push_numeric_example();

    // normalize arena
    arena.assert_eval_steps(1, 0);
    assert_eq(arena.get(target_formula), arena.get(target_formula));

    let SUCC = arena.push(Node::Succ);
    let CONST_0 = arena.push(Node::Const(0));
    let app_succ = arena.push(Node::App(target_formula, SUCC));
    let app_succ_0 = arena.push(Node::App(app_succ, CONST_0));

    // target_formula (+1) 0
    let target_formula = app_succ_0;

    // normalize arena
    arena.assert_eval_steps(20, 20);
    assert_eq(arena.get(target_formula), Node::Const(3));

    // check final length
    assert_eq(arena.inner.len(), 33);
}

#[test]
unconstrained fn test_numeric_and_show() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // 3
    let target_formula = arena.push_numeric_example();

    // print AST
    arena.assert_show::<PRINT_SIZE, _>(
        target_formula,
        [
            "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")", ")", " ", "K", ")",
            ")", " ", "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")", ")", " ",
            "K", ")", ")", " ", "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")",
            ")", " ", "K", ")", ")", " ", "(", "K", " ", "I", ")", ")", ")", ")",
        ],
    );

    // normalize arena
    arena.assert_eval_steps(1, 0);
    assert_eq(arena.get(target_formula), arena.get(target_formula));

    let SUCC = arena.push(Node::Succ);
    let CONST_0 = arena.push(Node::Const(0));
    let app_succ = arena.push(Node::App(target_formula, SUCC));
    let app_succ_0 = arena.push(Node::App(app_succ, CONST_0));

    // target_formula (+1) 0
    let target_formula = app_succ_0;

    // normalize arena
    arena.assert_eval_steps(20, 20);
    assert_eq(arena.get(target_formula), Node::Const(3));

    // check final length
    assert_eq(arena.inner.len(), 33);
}

#[test]
fn test_pow() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // 2^3
    let target_formula = arena.push_pow_example();

    // normalize arena
    arena.assert_eval_steps(10, 15);

    let SUCC = arena.push(Node::Succ);
    let CONST_0 = arena.push(Node::Const(0));
    let app_succ = arena.push(Node::App(target_formula, SUCC));
    let app_succ_0 = arena.push(Node::App(app_succ, CONST_0));

    // target_formula (+1) 0
    let target_formula = app_succ_0;

    // normalize arena
    arena.assert_eval_steps(20, 82);
    assert_eq(arena.get(target_formula), Node::Const(8));

    // check final length
    assert_eq(arena.inner.len(), 99);
}

#[test]
unconstrained fn test_pow_and_show() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // 2^3
    let target_formula = arena.push_pow_example();

    // print AST
    arena.assert_show::<PRINT_SIZE, _>(
        target_formula,
        [
            "(", "(", "(", "(", "S", " ", "(", "K", " ", "(", "S", " ", "I", ")", ")", ")", " ",
            "K", ")", " ", "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")", ")",
            " ", "K", ")", ")", " ", "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S",
            ")", ")", " ", "K", ")", ")", " ", "(", "K", " ", "I", ")", ")", ")", ")", " ", "(",
            "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")", ")", " ", "K", ")", ")",
            " ", "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")", ")", " ", "K",
            ")", ")", " ", "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")", ")",
            " ", "K", ")", ")", " ", "(", "K", " ", "I", ")", ")", ")", ")", ")",
        ],
    );

    // normalize arena
    arena.assert_eval_steps(10, 15);

    let SUCC = arena.push(Node::Succ);
    let CONST_0 = arena.push(Node::Const(0));
    let app_succ = arena.push(Node::App(target_formula, SUCC));
    let app_succ_0 = arena.push(Node::App(app_succ, CONST_0));

    // target_formula (+1) 0
    let target_formula = app_succ_0;

    // normalize arena
    arena.assert_eval_steps(20, 82);
    assert_eq(arena.get(target_formula), Node::Const(8));

    // print AST
    arena.assert_show::<ARENA_SIZE, _>(target_formula, ["8"]);

    // check final length
    assert_eq(arena.inner.len(), 99);
}

#[test]
fn test_add() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    let target_formula = arena.push_add_example();

    // normalize arena
    arena.assert_eval_steps(10, 13);

    let SUCC = arena.push(Node::Succ);
    let CONST_0 = arena.push(Node::Const(0));
    let app_succ = arena.push(Node::App(target_formula, SUCC));
    let app_succ_0 = arena.push(Node::App(app_succ, CONST_0));

    // target_formula (+1) 0
    let target_formula = app_succ_0;

    // normalize arena
    arena.assert_eval_steps(20, 42);
    assert_eq(arena.get(target_formula), Node::Const(5));

    // check final length
    assert_eq(arena.inner.len(), 78);
}

#[test]
unconstrained fn test_add_and_show() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    let target_formula = arena.push_add_example();

    // print AST
    arena.assert_show::<PRINT_SIZE, _>(
        target_formula,
        [
            "(", "(", "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")", ")", " ",
            "(", "(", "S", " ", "(", "K", " ", "K", ")", ")", " ", "(", "(", "S", " ", "(", "K",
            " ", "S", ")", ")", " ", "(", "(", "S", " ", "(", "K", " ", "(", "S", " ", "(", "K",
            " ", "S", ")", ")", ")", ")", " ", "(", "S", " ", "(", "K", " ", "K", ")", ")", ")",
            ")", ")", ")", ")", " ", "(", "K", " ", "I", ")", ")", " ", "(", "(", "S", " ", "(",
            "(", "S", " ", "(", "K", " ", "S", ")", ")", " ", "K", ")", ")", " ", "(", "(", "S",
            " ", "(", "(", "S", " ", "(", "K", " ", "S", ")", ")", " ", "K", ")", ")", " ", "(",
            "K", " ", "I", ")", ")", ")", ")", " ", "(", "(", "S", " ", "(", "(", "S", " ", "(",
            "K", " ", "S", ")", ")", " ", "K", ")", ")", " ", "(", "(", "S", " ", "(", "(", "S",
            " ", "(", "K", " ", "S", ")", ")", " ", "K", ")", ")", " ", "(", "(", "S", " ", "(",
            "(", "S", " ", "(", "K", " ", "S", ")", ")", " ", "K", ")", ")", " ", "(", "K", " ",
            "I", ")", ")", ")", ")", ")",
        ],
    );

    // normalize arena
    arena.assert_eval_steps(10, 13);

    let SUCC = arena.push(Node::Succ);
    let CONST_0 = arena.push(Node::Const(0));
    let app_succ = arena.push(Node::App(target_formula, SUCC));
    let app_succ_0 = arena.push(Node::App(app_succ, CONST_0));

    // target_formula (+1) 0
    let target_formula = app_succ_0;

    // normalize arena
    arena.assert_eval_steps(20, 42);
    assert_eq(arena.get(target_formula), Node::Const(5));

    // print AST
    arena.assert_show::<ARENA_SIZE, _>(target_formula, ["5"]);

    // check final length
    assert_eq(arena.inner.len(), 78);
}

// Docs

// SUCC := \n.\f.\x.f (n f x)
//
// Given:
// S x y z = (x z) (y z)
// S x y = \z. (x z) (y z)
//
// SUCC := \n.\f.\x.f ((n f) x)
// SUCC := \n.\f. \x. f ((n f) x)
//  \x. f ((n f) x)
//  S (\x.f) (\x.((n f) x))
//  S (K f) (\x. (n f) x)
//  S (K f) (n f)
//  (S (K f)) (n f)
// SUCC := \n.\f. (S (K f)) (n f)
// SUCC := \n. \f. (S (K f)) (n f)
//  \f. (S (K f)) (n f)
//  S (\f. S (K f)) (\f. n f)
//  S (\f. S (K f)) n
//  S (S (\f. S) (\f. K f)) n
//  S (S (K S) K) n
// SUCC := \n. S (S (K S) K) n
// SUCC := S (S (K S) K)
//
// Testing in Haskell:
// ghci> let ss = \x y z -> (x z) (y z)
// ghci> let kk = const
// ghci> let ii = id
// ghci> :t kk ii
// kk ii :: b -> a -> a
// ghci> :t ss (ss (kk ss) kk)
// ss (ss (kk ss) kk)
//   :: ((t2 -> t3) -> t1 -> t2) -> (t2 -> t3) -> t1 -> t3
// ghci> :t (ss (ss (kk ss) kk)) (kk ii)
// (ss (ss (kk ss) kk)) (kk ii) :: (t2 -> t3) -> t2 -> t3
// ghci> :t (ss (ss (kk ss) kk)) (kk ii) (+1)
// (ss (ss (kk ss) kk)) (kk ii) (+1) :: Num t3 => t3 -> t3
// ghci> :t (ss (ss (kk ss) kk)) (kk ii) (+1) 0
// (ss (ss (kk ss) kk)) (kk ii) (+1) 0 :: Num t => t
// ghci> (ss (ss (kk ss) kk)) (kk ii) (+1) 0
// 1

// PLUS := \m.\n.\f.\x.m f (n f x)
// PLUS := \m.\n.\f. \x.m f (n f x)
//  \x. m f (n f x)
//  S (\x. m f) (\x. n f x)
//  S (K (m f)) (n f)
// PLUS := \m.\n.\f. (S (K (m f))) (n f)
// PLUS := \m.\n. \f. (S (K (m f))) (n f)
// PLUS := \m.\n.
//  \f. (S (K (m f))) (n f)
//  S (\f. S (K (m f))) (\f. n f)
//  S (\f. S (K (m f))) n
//    (\f. S (K (m f)))
//    S (\f. S) (\f. K (m f))
//    S (K S) (S (\f. K) (\f. m f))
//    S (K S) (S (K K) m)
//  S (S (K S) (S (K K) m)) n
// PLUS := \m.\n. S (S (K S) (S (K K) m)) n
// PLUS := \m.\n. (S (S (K S) (S (K K) m))) n
// PLUS := \m. \n. (S (S (K S) (S (K K) m))) n
// PLUS := \m. S (\n. S (S (K S) (S (K K) m))) (\n. n)
// PLUS := \m. S (K (S (S (K S) (S (K K) m)))) I
// PLUS := \m. (S (K (S (S (K S) (S (K K) m))))) I
// PLUS := S (\m. S (K (S (S (K S) (S (K K) m))))) (\m. I)
// PLUS := S (\m. S (K (S (S (K S) (S (K K) m))))) (K I)
//   (\m. S (K (S (S (K S) (S (K K) m)))))
//   \m. S (K (S (S (K S) (S (K K) m))))
//   S (\m. S) (\m. K (S (S (K S) (S (K K) m))))
//   S (K S) (\m. K (S (S (K S) (S (K K) m))))
//     (\m. K (S (S (K S) (S (K K) m))))
//     \m. K (S (S (K S) (S (K K) m)))
//     S (\m. K) (\m. S (S (K S) (S (K K) m)))
//     S (K K) (\m. S (S (K S) (S (K K) m)))
//       (\m. S (S (K S) (S (K K) m)))
//       \m. S (S (K S) (S (K K) m))
//       S (\m. S) (\m. (S (K S)) (S (K K) m))
//       S (K S) (\m. (S (K S)) (S (K K) m))
//         (\m. (S (K S)) (S (K K) m))
//         \m. (S (K S)) (S (K K) m)
//         S (\m. S (K S)) (\m. S (K K) m)
//         S (K (S (K S))) (S (K K))
//       S (K S) (S (K (S (K S))) (S (K K)))
//     S (K K) (S (K S) (S (K (S (K S))) (S (K K))))
//   S (K S) (S (K K) (S (K S) (S (K (S (K S))) (S (K K)))))
// PLUS := S (S (K S) (S (K K) (S (K S) (S (K (S (K S))) (S (K K)))))) (K I)
//
// Testing in Haskell:
// ghci> let zero = kk ii
// ghci> let one = (ss (ss (kk ss) kk)) zero
// ghci> let two = (ss (ss (kk ss) kk)) one
// ghci> let three = (ss (ss (kk ss) kk)) two
//
// ghci> let plus = ss (ss (kk ss) (ss (kk kk) (ss (kk ss) (ss (kk (ss (kk ss))) (ss (kk kk)))))) (kk ii)
// ghci> :t plus
// plus :: (t2 -> t3 -> t4) -> (t2 -> t5 -> t3) -> t2 -> t5 -> t4
// ghci> plus one two (+1) 0
// 3

// POW := \b.\e.e b
// POW := \b. \e. e b
// POW := \b. S (\e. e) (\e. b)
// POW := \b. S I (K b)
// POW := \b. (S I) (K b)
// POW := S (\b. S I) (\b. K b)
// POW := S (K (S I)) K
//
// Testing in Haskell:
// ghci> let pow = ss (kk (ss ii)) kk
// ghci> :t pow two three
// pow two three :: (t1 -> t1) -> t1 -> t1
// ghci> pow two three (+1) 0
// 8
