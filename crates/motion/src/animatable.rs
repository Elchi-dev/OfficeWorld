//! What a value needs so that it can be animated.

/// A value that a spring can move and a tween can blend.
///
/// Anything that behaves like a vector fits: a number, a point, a size, a
/// colour in a linear space. Springs and tweens only ever add values and
/// stretch them, so every part of a value follows the same curve and a
/// diagonal move stays a straight line.
pub trait Animatable: Copy {
    /// The value that changes nothing when it is added.
    fn zero() -> Self;

    /// The sum of two values.
    #[must_use]
    fn plus(self, other: Self) -> Self;

    /// The value stretched by a factor.
    #[must_use]
    fn scaled(self, factor: f64) -> Self;

    /// The squared length of the value, in its own units.
    fn length_squared(self) -> f64;

    /// The difference of two values.
    #[must_use]
    fn minus(self, other: Self) -> Self {
        self.plus(other.scaled(-1.0))
    }
}

impl Animatable for f64 {
    fn zero() -> Self {
        0.0
    }

    fn plus(self, other: Self) -> Self {
        self + other
    }

    fn scaled(self, factor: f64) -> Self {
        self * factor
    }

    fn length_squared(self) -> f64 {
        self * self
    }
}

impl Animatable for f32 {
    fn zero() -> Self {
        0.0
    }

    fn plus(self, other: Self) -> Self {
        self + other
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "the product is computed in f64 and rounded back to the f32 it is stored as"
    )]
    fn scaled(self, factor: f64) -> Self {
        (f64::from(self) * factor) as f32
    }

    fn length_squared(self) -> f64 {
        f64::from(self) * f64::from(self)
    }
}

impl<T: Animatable, const N: usize> Animatable for [T; N] {
    fn zero() -> Self {
        [T::zero(); N]
    }

    fn plus(self, other: Self) -> Self {
        std::array::from_fn(|index| self[index].plus(other[index]))
    }

    fn scaled(self, factor: f64) -> Self {
        self.map(|part| part.scaled(factor))
    }

    fn length_squared(self) -> f64 {
        self.iter().map(|part| part.length_squared()).sum()
    }
}
