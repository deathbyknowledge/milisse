pub type Word = u16;

// Trait used for encoding words as u16 (omitting sync waves and parity bits).
pub trait Encode {
    fn encode(&self) -> Word;
}

/*
// Primitive Types for bit sized fields.
*/

#[derive(Debug, Copy, Clone)]
pub struct BitField<const SIZE: usize> {
    pub value: u8,
}

impl<const SIZE: usize> BitField<SIZE> {
    pub fn new(value: u8) -> Self {
        if SIZE > 8 {
            panic!("SIZE is too large for u8.");
        }
        assert!(value < (1 << SIZE), "Value exceeds the bitfield size");
        Self { value }
    }
}

// FIELD_SIZE in bits. LSB_IDX the position of the LSB of the BitField, which
// translates to the amount of bits to shift left.
pub trait AlignableBitField<const FIELD_SIZE: usize, const LSB_IDX: u8>:
    Into<BitField<FIELD_SIZE>>
where
    Self: Clone,
{
    fn align_to_word(&self) -> u16 {
        let bit_field: BitField<FIELD_SIZE> = (self.clone()).into();
        (bit_field.value as u16) << LSB_IDX
    }
}

// For types that want to encapsulate adjacent fields
// resulting in more than 8 bits when added.
#[derive(Debug, Copy, Clone)]
pub struct ComplexBitField<const SIZE: usize> {
    pub value: u16,
}

impl<const SIZE: usize> ComplexBitField<SIZE> {
    pub fn new(value: Word) -> Self {
        if SIZE > 16 {
            panic!("SIZE is too large for u16.");
        }
        assert!(value < (1 << SIZE), "Value exceeds the bitfield size");
        Self { value }
    }
}

// FIELD_SIZE in bits. LSB_IDX the position of the LSB of the BitField, which
// translates to the amount of bits to shift left.
pub trait AlignableComplexBitField<const FIELD_SIZE: usize, const LSB_IDX: u8>:
    Into<ComplexBitField<FIELD_SIZE>>
where
    Self: Clone,
{
    fn align_to_word(&self) -> u16 {
        let bit_field: ComplexBitField<FIELD_SIZE> = (self.clone()).into();
        bit_field.value << LSB_IDX
    }
}
