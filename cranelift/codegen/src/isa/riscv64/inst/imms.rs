//! Riscv64 ISA definitions: immediate constants.

// Some variants are never constructed, but we still want them as options in the future.
use super::Inst;
#[allow(dead_code)]
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Copy, Clone, Debug, Default)]
pub struct Imm12 {
    /// 16-bit container where the low 12 bits are the data payload.
    ///
    /// Acquiring the underlying value requires sign-extending the 12th bit.
    bits: u16,
}

impl Imm12 {
    pub(crate) const ZERO: Self = Self { bits: 0 };
    pub(crate) const ONE: Self = Self { bits: 1 };

    pub fn maybe_from_u64(val: u64) -> Option<Imm12> {
        Self::maybe_from_i64(val as i64)
    }

    pub fn maybe_from_i64(val: i64) -> Option<Imm12> {
        if val >= -2048 && val <= 2047 {
            Some(Imm12 {
                bits: val as u16 & 0xfff,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn from_i16(bits: i16) -> Self {
        assert!(bits >= -2048 && bits <= 2047);
        Self {
            bits: (bits & 0xfff) as u16,
        }
    }

    #[inline]
    pub fn as_i16(self) -> i16 {
        (self.bits << 4) as i16 >> 4
    }

    #[inline]
    pub fn bits(&self) -> u32 {
        self.bits.into()
    }
}

impl Into<i64> for Imm12 {
    fn into(self) -> i64 {
        self.as_i16().into()
    }
}

impl Display for Imm12 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:+}", self.as_i16())
    }
}

// singed
#[derive(Clone, Copy, Default)]
pub struct Imm20 {
    /// 32-bit container where the low 20 bits are the data payload.
    ///
    /// Acquiring the underlying value requires sign-extending the 20th bit.
    bits: u32,
}

impl Imm20 {
    pub(crate) const ZERO: Self = Self { bits: 0 };

    #[inline]
    pub fn from_i32(bits: i32) -> Self {
        assert!(bits >= -(0x7_ffff + 1) && bits <= 0x7_ffff);
        Self {
            bits: (bits as u32) & 0xf_ffff,
        }
    }

    #[inline]
    pub fn as_i32(&self) -> i32 {
        ((self.bits << 12) as i32) >> 12
    }

    #[inline]
    pub fn bits(&self) -> u32 {
        self.bits
    }
}

impl Debug for Imm20 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.as_i32())
    }
}

impl Display for Imm20 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.bits)
    }
}

/// An unsigned 5-bit immediate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UImm5 {
    value: u8,
}

impl UImm5 {
    /// Create an unsigned 5-bit immediate from u8.
    pub fn maybe_from_u8(value: u8) -> Option<UImm5> {
        if value < 32 {
            Some(UImm5 { value })
        } else {
            None
        }
    }

    /// Bits for encoding.
    pub fn bits(&self) -> u32 {
        u32::from(self.value)
    }
}

impl Display for UImm5 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value)
    }
}

/// A Signed 5-bit immediate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Imm5 {
    value: i8,
}

impl Imm5 {
    /// Create an signed 5-bit immediate from an i8.
    pub fn maybe_from_i8(value: i8) -> Option<Imm5> {
        if value >= -16 && value <= 15 {
            Some(Imm5 { value })
        } else {
            None
        }
    }

    pub fn from_bits(value: u8) -> Imm5 {
        assert_eq!(value & 0x1f, value);
        let signed = ((value << 3) as i8) >> 3;
        Imm5 { value: signed }
    }

    /// Bits for encoding.
    pub fn bits(&self) -> u8 {
        self.value as u8 & 0x1f
    }
}

impl Display for Imm5 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value)
    }
}

impl Inst {
    pub(crate) fn imm_min() -> i64 {
        let imm20_max: i64 = (1 << 19) << 12;
        let imm12_max = 1 << 11;
        -imm20_max - imm12_max
    }
    pub(crate) fn imm_max() -> i64 {
        let imm20_max: i64 = ((1 << 19) - 1) << 12;
        let imm12_max = (1 << 11) - 1;
        imm20_max + imm12_max
    }

    /// An imm20 immediate and an Imm12 immediate can generate a 32-bit immediate.
    /// This helper produces an imm12, imm20, or both to generate the value.
    ///
    /// `value` must be between `imm_min()` and `imm_max()`, or else
    /// this helper returns `None`.
    pub(crate) fn generate_imm(value: u64) -> Option<(Imm20, Imm12)> {
        if let Some(imm12) = Imm12::maybe_from_u64(value) {
            // can be load using single imm12.
            return Some((Imm20::ZERO, imm12));
        }
        let value = value as i64;
        if !(value >= Self::imm_min() && value <= Self::imm_max()) {
            // not in range, return None.
            return None;
        }
        const MOD_NUM: i64 = 4096;
        let (imm20, imm12) = if value > 0 {
            let mut imm20 = value / MOD_NUM;
            let mut imm12 = value % MOD_NUM;
            if imm12 >= 2048 {
                imm12 -= MOD_NUM;
                imm20 += 1;
            }
            assert!(imm12 >= -2048 && imm12 <= 2047);
            (imm20, imm12)
        } else {
            // this is the abs value.
            let value_abs = value.abs();
            let imm20 = value_abs / MOD_NUM;
            let imm12 = value_abs % MOD_NUM;
            let mut imm20 = -imm20;
            let mut imm12 = -imm12;
            if imm12 < -2048 {
                imm12 += MOD_NUM;
                imm20 -= 1;
            }
            (imm20, imm12)
        };
        assert!(imm20 != 0 || imm12 != 0);
        let imm20 = i32::try_from(imm20).unwrap();
        let imm12 = i16::try_from(imm12).unwrap();
        Some((Imm20::from_i32(imm20), Imm12::from_i16(imm12)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_imm12() {
        let x = Imm12::ZERO;
        assert_eq!(0, x.bits());
        Imm12::maybe_from_u64(0xffff_ffff_ffff_ffff).unwrap();
    }

    #[test]
    fn imm20_and_imm12() {
        assert!(Inst::imm_max() == (i32::MAX - 2048) as i64);
        assert!(Inst::imm_min() == i32::MIN as i64 - 2048);
    }
}
