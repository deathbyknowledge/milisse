const WORD_SIZE: u8 = 16;
pub type Word = u16;

// Trait used for encoding words as u16 (omitting sync waves and parity bits).
pub trait Encode {
    fn encode(&self) -> Word;
}

/*
// Primitive Types for bit sized fields.
*/

#[derive(Debug, Copy, Clone)]
pub struct BitField<const SIZE: u8> {
    pub value: u8,
}

impl<const SIZE: u8> BitField<SIZE> {
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
pub trait AlignableBitField<const FIELD_SIZE: u8, const LSB_IDX: u8>:
    Into<BitField<FIELD_SIZE>>
where
    Self: Clone,
{
    // Align self to its desired bit position by shifting left.
    fn align_to_word(&self) -> Word {
        let bit_field: BitField<FIELD_SIZE> = (self.clone()).into();
        (bit_field.value as Word) << LSB_IDX
    }

    // Set position of self in the value word, without changing any other bits.
    fn set(&self, value: Word) -> Word {
        let shft_left_amount = WORD_SIZE - FIELD_SIZE + LSB_IDX;
        let mask = 0xFF >> LSB_IDX << LSB_IDX << shft_left_amount >> shft_left_amount;
        let result = value & !mask;
        result | self.align_to_word()
    }
}

// For types that want to encapsulate adjacent fields
// resulting in more than 8 bits when added.
#[derive(Debug, Copy, Clone)]
pub struct ComplexBitField<const SIZE: u8> {
    pub value: Word,
}

impl<const SIZE: u8> ComplexBitField<SIZE> {
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
pub trait AlignableComplexBitField<const FIELD_SIZE: u8, const LSB_IDX: u8>:
    Into<ComplexBitField<FIELD_SIZE>>
where
    Self: Clone,
{
    fn align_to_word(&self) -> Word {
        let bit_field: ComplexBitField<FIELD_SIZE> = (self.clone()).into();
        bit_field.value << LSB_IDX
    }

    // Set position of self in the value word, without changing any other bits.
    fn set(&self, value: Word) -> Word {
        let shft_left_amount = WORD_SIZE - FIELD_SIZE + LSB_IDX;
        let mask = 0xFF >> LSB_IDX << LSB_IDX << shft_left_amount >> shft_left_amount;
        let result = value & !mask;
        result | self.align_to_word()
    }
}
