#[macro_export]
macro_rules! deref {
    (
        $(#[$meta:meta])*
        $e_vis:vis struct $ty:ident($data:path);
    ) => {
        $(#[$meta])*
        $e_vis struct $ty($data);

        impl $ty {
            pub fn new(items: $data) -> Self {
                Self(items)
            }

            pub fn value(self) -> $data {
                self.0
            }
        }

        impl From<$data> for $ty {
            fn from(items: $data) -> Self {
                Self::new(items)
            }
        }

        impl std::ops::Deref for $ty {
            type Target = $data;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl std::ops::DerefMut for $ty {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };

    ($T:ty, $child:ty, $field:ident) => {
        impl std::ops::Deref for $T {
            type Target = $child;
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }
        impl std::ops::DerefMut for $T {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    }
}
