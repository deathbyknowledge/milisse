const WORD_SIZE: u8 = 16;
pub type Word = u16;

/*
// Primitive Types for bit sized fields.
*/

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BitField<const SIZE: u8> {
    raw_value: u8,
}

impl<const SIZE: u8> BitField<SIZE> {
    pub fn new(value: u8) -> Self {
        if SIZE > 8 {
            panic!("SIZE is too large for u8.");
        }
        assert!(value < (1 << SIZE), "Value exceeds the bitfield size");
        Self { raw_value: value }
    }

    pub fn value(&self) -> u8 {
        self.raw_value
    }
}

macro_rules! impl_from_for_bitfield {
  ($($size:expr),*) => {
      $(
          impl From<u8> for BitField<$size> {
              fn from(value: u8) -> Self {
                  BitField::new(value)
              }
          }

          impl From<BitField<$size>> for u8 {
              fn from(bitfield: BitField<$size>) -> Self {
                  bitfield.raw_value
              }
          }

          impl From<BitField<$size>> for u16 {
              fn from(bitfield: BitField<$size>) -> Self {
                  bitfield.raw_value as u16
              }
          }
      )*
  };
}

macro_rules! impl_from_for_complex_bitfield {
  ($($size:expr),*) => {
      $(
          impl From<u16> for ComplexBitField<$size> {
              fn from(value: u16) -> Self {
                  ComplexBitField::new(value)
              }
          }

          impl From<ComplexBitField<$size>> for u16 {
              fn from(bitfield: ComplexBitField<$size>) -> Self {
                  bitfield.raw_value
              }
          }
      )*
  };
}

// Implement From<u8> for all possible BitField lengths.
impl_from_for_bitfield!(1, 2, 3, 4, 5, 6, 7, 8);
impl_from_for_complex_bitfield!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

impl From<bool> for BitField<1> {
    fn from(value: bool) -> Self {
        BitField::new(value as u8)
    }
}

impl From<BitField<1>> for bool {
    fn from(bitfield: BitField<1>) -> Self {
        bitfield.into()
    }
}

// FIELD_SIZE in bits. LSB_IDX the position of the LSB of the BitField, which
// translates to the amount of bits to shift left.
pub trait AlignableBitField<const FIELD_SIZE: u8, const LSB_IDX: u8>:
    Into<BitField<FIELD_SIZE>> + From<BitField<FIELD_SIZE>>
where
    Self: Clone,
{
    // Align self to its desired bit position by shifting left.
    fn align_to_word(&self) -> Word {
        let bit_field: BitField<FIELD_SIZE> = (self.clone()).into();
        (bit_field.raw_value as Word) << LSB_IDX
    }

    // Set position of self in the value word, without changing any other bits.
    fn set_in(&self, value: Word) -> Word {
        let shift_left = WORD_SIZE - (FIELD_SIZE + LSB_IDX);
        let mask = 0xFFFF >> LSB_IDX << LSB_IDX << shift_left >> shift_left;
        let result = value & !mask;
        result | self.align_to_word()
    }

    fn read(value: Word) -> Self {
        let shift_left = WORD_SIZE - (FIELD_SIZE + LSB_IDX);
        let result = value << shift_left >> shift_left >> (LSB_IDX);
        let bitfield = BitField::new(result as u8);
        bitfield.into()
    }
}

// For types that want to encapsulate adjacent fields
// resulting in more than 8 bits when added.
#[derive(Debug, Copy, Clone)]
pub struct ComplexBitField<const SIZE: u8> {
    raw_value: Word,
}

impl<const SIZE: u8> ComplexBitField<SIZE> {
    pub fn new(value: Word) -> Self {
        if SIZE > 16 {
            panic!("SIZE is too large for u16.");
        }
        assert!(value < (1 << SIZE), "Value exceeds the bitfield size");
        Self { raw_value: value }
    }

    pub fn value(&self) -> Word {
        self.raw_value
    }
}

// FIELD_SIZE in bits. LSB_IDX the position of the LSB of the BitField, which
// translates to the amount of bits to shift left.
pub trait AlignableComplexBitField<const FIELD_SIZE: u8, const LSB_IDX: u8>:
    Into<ComplexBitField<FIELD_SIZE>> + From<ComplexBitField<FIELD_SIZE>>
where
    Self: Clone,
{
    fn align_to_word(&self) -> Word {
        let bit_field: ComplexBitField<FIELD_SIZE> = (self.clone()).into();
        bit_field.raw_value << LSB_IDX
    }

    // Set position of self in the value word, without changing any other bits.
    fn set_in(&self, value: Word) -> Word {
        let shft_left_amount = WORD_SIZE - FIELD_SIZE + LSB_IDX;
        let mask = 0xFFFF >> LSB_IDX << LSB_IDX << shft_left_amount >> shft_left_amount;
        let result = value & !mask;
        result | self.align_to_word()
    }

    fn read(value: Word) -> Self {
        let shift_left = WORD_SIZE - (FIELD_SIZE + LSB_IDX);
        let result = value << shift_left >> shift_left >> (LSB_IDX);
        let bitfield = ComplexBitField::new(result);
        bitfield.into()
    }
}
