//! Note the documentation for the primitives **str** and **[T]** (also
//! called 'slice'). Many method calls on **String** and **Vec<T>** are actually
//! calls to methods on **str** and **[T]** respectively, via **deref
//! coercions**.



///doc comment
/*
  This is a block comment describing a complex function.
*/
fn main(x : Field, y : pub Field) {
    assert(x != y);
}

///qwe
///rty
fn simple_fn(){}

///qwe
///rty
pub fn pub_fn(){}

//! Data available to helper can be found in [Helper](struct.Helper.html). And there are more
//! examples in [HelperDef](trait.HelperDef.html) page.
//!
//! You can learn more about helpers by looking into source code of built-in helpers.
//!
//! Like our JavaScript counterparts, handlebars allows user to define simple helpers with
//! a scripting language, [rhai](https://docs.rs/crate/rhai/). This can be enabled by
//! turning on `script_helper` feature flag.

/// struct
struct MyStruct {
  mana: i32,
  name: String,
}

///a 
///lot 
///of 
/**
doc 
comments
*/
fn another_fn(){}

/// com 1
/// com 2
pub struct AnotherStruct {
  pub name: String,
  pub number: i32,
}

impl MyStruct {
  fn new() {}
}

impl<X: SampleUniform> MyStruct<X> {
    /// Create a new `Uniform` instance which samples uniformly from the half
    /// open range `[low, high)` (excluding `high`). Panics if `low >= high`.
    pub fn new<B1, B2>(low: B1, high: B2) -> Uniform<X>
    where
        B1: SampleBorrow<X> + Sized,
        B2: SampleBorrow<X> + Sized,
    {
        Uniform(X::Sampler::new(low, high))
    }

    /// Create a new `Uniform` instance which samples uniformly from the closed
    /// range `[low, high]` (inclusive). Panics if `low > high`.
    pub fn new_inclusive<B1, B2>(low: B1, high: B2) -> Uniform<X>
    where
        B1: SampleBorrow<X> + Sized,
        B2: SampleBorrow<X> + Sized,
    {
        Uniform(X::Sampler::new_inclusive(low, high))
    }
}

/// Range that supports generating a single sample efficiently.
///
/// Any type implementing this trait can be used to specify the sampled range
/// for `Rng::gen_range`.
pub trait SampleRange<T> {
    /// Generate a sample from the given range.
    fn sample_single<R: RngCore + ?Sized>(self, rng: &mut R) -> T;

    /// Check whether the range is empty.
    fn is_empty(&self) -> bool;
}

impl<T: SampleUniform + PartialOrd> SampleRange<T> for AnotherStruct<T> {
    #[inline]
    fn sample_single<R: RngCore + ?Sized>(self, rng: &mut R) -> T {
        T::Sampler::sample_single(self.start, self.end, rng)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        !(self.start < self.end)
    }
}

/// this is a trait =-=
//! Helper trait handling actual uniform sampling.
//!
//! See the [module documentation] on how to implement [`Uniform`] range
//! sampling for a custom type.
//!
//! Implementation of [`sample_single`] is optional, and is only useful when
//! the implementation can be faster than `Self::new(low, high).sample(rng)`.
//!
//! [module documentation]: crate::distributions::uniform
//! [`sample_single`]: UniformSampler::sample_single
pub trait UniformSampler: Sized {
    /// The type sampled by this implementation.
    type X;

    /// Construct self, with inclusive lower bound and exclusive upper bound
    /// `[low, high)`.
    ///
    /// Usually users should not call this directly but instead use
    /// `Uniform::new`, which asserts that `low < high` before calling this.
    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized;

    /// Construct self, with inclusive bounds `[low, high]`.
    ///
    /// Usually users should not call this directly but instead use
    /// `Uniform::new_inclusive`, which asserts that `low <= high` before
    /// calling this.
    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized;

    /// Sample a value.
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X;

    /// Sample a single value uniformly from a range with inclusive lower bound
    /// and exclusive upper bound `[low, high)`.
    ///
    /// By default this is implemented using
    /// `UniformSampler::new(low, high).sample(rng)`. However, for some types
    /// more optimal implementations for single usage may be provided via this
    /// method (which is the case for integers and floats).
    /// Results may not be identical.
    ///
    /// Note that to use this method in a generic context, the type needs to be
    /// retrieved via `SampleUniform::Sampler` as follows:
    /// ```
    /// use rand::{thread_rng, distributions::uniform::{SampleUniform, UniformSampler}};
    /// # #[allow(unused)]
    /// fn sample_from_range<T: SampleUniform>(lb: T, ub: T) -> T {
    ///     let mut rng = thread_rng();
    ///     <T as SampleUniform>::Sampler::sample_single(lb, ub, &mut rng)
    /// }
    /// ```
    fn sample_single<R: Rng + ?Sized, B1, B2>(low: B1, high: B2, rng: &mut R) -> Self::X
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let uniform: Self = UniformSampler::new(low, high);
        uniform.sample(rng)
    }

    fn new_fn() -> i32;

    /// Sample a single value uniformly from a range with inclusive lower bound
    /// and inclusive upper bound `[low, high]`.
    ///
    /// By default this is implemented using
    /// `UniformSampler::new_inclusive(low, high).sample(rng)`. However, for
    /// some types more optimal implementations for single usage may be provided
    /// via this method.
    /// Results may not be identical.
    fn sample_single_inclusive<R: Rng + ?Sized, B1, B2>(low: B1, high: B2, rng: &mut R)
        -> Self::X
        where B1: SampleBorrow<Self::X> + Sized,
              B2: SampleBorrow<Self::X> + Sized
    {
        let uniform: Self = UniformSampler::new_inclusive(low, high);
        uniform.sample(rng)
    }
}

impl UniformSampler for UniformChar {
    type X = char;

    #[inline] // if the range is constant, this helps LLVM to do the
              // calculations at compile-time.
    fn new<B1, B2>(low_b: B1, high_b: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let low = char_to_comp_u32(*low_b.borrow());
        let high = char_to_comp_u32(*high_b.borrow());
        let sampler = UniformInt::<u32>::new(low, high);
        UniformChar { sampler }
    }

    #[inline] // if the range is constant, this helps LLVM to do the
              // calculations at compile-time.
    fn new_inclusive<B1, B2>(low_b: B1, high_b: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let low = char_to_comp_u32(*low_b.borrow());
        let high = char_to_comp_u32(*high_b.borrow());
        let sampler = UniformInt::<u32>::new_inclusive(low, high);
        UniformChar { sampler }
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        let mut x = self.sampler.sample(rng);
        if x >= CHAR_SURROGATE_START {
            x += CHAR_SURROGATE_LEN;
        }
        // SAFETY: x must not be in surrogate range or greater than char::MAX.
        // This relies on range constructors which accept char arguments.
        // Validity of input char values is assumed.
        unsafe { core::char::from_u32_unchecked(x) }
    }
}

mod MyModule {
    ///module function
    fn module_fn() {
        let a = 100;
    }

    struct MyStruct {
        mana: i32,
        name: String,
    }
}

mod another_module;

fn yes () {}

struct GrumpkinScalar {
  low: Field,
  high: Field,
}

struct B<T_C> {}


fn sss(new_concrete_t_c_constructor: fn () -> T_C) -> B<T_C> {
    B { new_concrete_t_c_constructor }
}

impl<T_C> B<T_C> {
    fn new(new_concrete_t_c_constructor: fn () -> T_C) -> B<T_C> {
        B { new_concrete_t_c_constructor }
    }
}

fn abc(){}
