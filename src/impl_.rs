use crate::{Scan, ScanBinary, ScanError, ScanLowerHex, ScanOctal, ScanUpperHex};

impl Scan for String {
    fn scan(s: &str) -> Result<Self, ScanError> {
        Ok(s.to_owned())
    }

    fn is_valid_start(_: char) -> bool {
        true
    }
}

macro_rules! int_impl {
    ($($intTy:ident)+) => {$(
        impl Scan for $intTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                s.parse().map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // leading sign or a number is a valid start.
            fn is_valid_start(c: char) -> bool {
                c.is_digit(10) || c == '-' || c == '+'
            }
        }

        impl ScanBinary for $intTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                <$intTy>::from_str_radix(s, 2).map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // leading sign or a number is a valid start.
            fn is_valid_start(c: char) -> bool {
                matches!(c, '0' | '1' | '-' | '+')
            }
        }

        impl ScanOctal for $intTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                <$intTy>::from_str_radix(s, 8).map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // leading sign or a number is a valid start.
            fn is_valid_start(c: char) -> bool {
                c.is_digit(8) || c == '-' || c == '+'
            }
        }

        impl ScanLowerHex for $intTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                <$intTy>::from_str_radix(s, 16).map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // leading sign or a number is a valid start.
            fn is_valid_start(c: char) -> bool {
                c.is_digit(16) || c == '-' || c == '+'
            }
        }

        impl ScanUpperHex for $intTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                <$intTy>::from_str_radix(&s.to_ascii_lowercase(), 16).map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // leading sign or a number is a valid start.
            fn is_valid_start(c: char) -> bool {
                c.is_digit(10) || (c.is_ascii_uppercase() && c.to_ascii_lowercase().is_digit(16)) || c == '-' || c == '+'
            }
        }
    )+};
}

macro_rules! uint_impl {
    ($($uintTy:ident)+) => {$(
        impl Scan for $uintTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                s.parse().map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // leading sign or a number is a valid start.
            fn is_valid_start(c: char) -> bool {
                c.is_digit(10) || c == '+'
            }
        }

        impl ScanBinary for $uintTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                <$uintTy>::from_str_radix(s, 2).map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // leading sign or a number is a valid start.
            fn is_valid_start(c: char) -> bool {
                matches!(c, '0' | '1' | '+')
            }
        }

        impl ScanOctal for $uintTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                <$uintTy>::from_str_radix(s, 8).map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // leading sign or a number is a valid start.
            fn is_valid_start(c: char) -> bool {
                c.is_digit(8) || c == '+'
            }
        }

        impl ScanLowerHex for $uintTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                <$uintTy>::from_str_radix(s, 16).map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // leading sign or a number is a valid start.
            fn is_valid_start(c: char) -> bool {
                c.is_digit(16) || c == '+'
            }
        }

        impl ScanUpperHex for $uintTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                <$uintTy>::from_str_radix(&s.to_ascii_lowercase(), 16).map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // leading sign or a number is a valid start.
            fn is_valid_start(c: char) -> bool {
                c.is_digit(10) || (c.is_ascii_uppercase() && c.to_ascii_lowercase().is_digit(16)) || c == '+'
            }
        }
    )+};
}

int_impl!(isize i8 i16 i32 i64 i128);
uint_impl!(usize u8 u16 u32 u64 u128);

macro_rules! float_impl {
    ($($floatTy:ident)+) => {$(
        impl Scan for $floatTy {
            fn scan(s: &str) -> Result<Self, ScanError> {
                s.parse().map_err(|e| ScanError::Custom(Box::new(e)))
            }

            // number, inf, NaN
            fn is_valid_start(c: char) -> bool {
                c.is_digit(10) || matches!(c, 'i' | 'N' | '-' | '+')
            }
        }
    )+};
}

float_impl!(f32 f64);
