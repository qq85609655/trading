pub use color::ShowColor;
pub use percent::Percent;

pub mod percent {
    use crate::Bar;

    pub trait Percent<T> {
        fn percent(&self, other: T) -> f64;
    }

    impl Percent<f64> for f64 {
        fn percent(&self, other: f64) -> f64 {
            if other == 0.0 {
                return 0.0;
            }
            (self - other) / other * 100.0
        }
    }

    impl Percent<f32> for f64 {
        fn percent(&self, other: f32) -> f64 {
            let other = other as f64;
            ((self - other) / other) * 100.0
        }
    }

    impl Percent<&Bar> for Bar {
        fn percent(&self, other: &Bar) -> f64 {
            self.close.percent(other.close)
        }
    }
}

pub mod color {
    pub trait ShowColor<T>
    where
        T: PartialOrd<Self> + Default,
    {
        type Output;
        fn color(&self) -> Self::Output {
            self.colorw(&T::default())
        }
        fn colorw(&self, other: &T) -> Self::Output;
    }

    #[macro_export]
    macro_rules! impl_color {
        ($A:ty, $B:ty, $color:ty, $red:expr, $grey:expr, $green:expr) => {
            impl ShowColor<$B> for $A {
                type Output = $color;
                fn colorw(&self, other: &$B) -> Self::Output {
                    match self.partial_cmp(other) {
                        None => $grey,
                        Some(order) => match order {
                            std::cmp::Ordering::Greater => $red,
                            std::cmp::Ordering::Equal => $grey,
                            std::cmp::Ordering::Less => $green,
                        },
                    }
                }
            }
        };
        ($T:ty, $color:ty, $red:expr, $grey:expr, $green:expr) => {
            impl_color!($T, $T, $color, $red, $grey, $green)
        };
    }

    #[cfg(feature = "iced")]
    impl_color!(
        f64,
        f64,
        iced::Color,
        iced::Color::from_rgb8(0xFF, 0, 0),
        iced::Color::WHITE,
        iced::Color::from_rgb8(0, 0xFF, 0)
    );
}
