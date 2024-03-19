//#![no_std]
const WORD_SIZE: usize = 16; // Word size in bits.
const SUBADDRESS_MODE_CODE_0: u8 = 0b00000; // Subaddress for mode code
const SUBADDRESS_MODE_CODE_1: u8 = 0b11111; // Subaddress for mode code
const BROADCAST_ADDR: u8 = 0b11111; // Address for Brodcast mode.

// Trait used for encoding words as u16 (omitting sync waves and parity bits).
pub trait Encode {
  fn encode(&self) -> u16; 
}

/*
// Primitive Types for bit sized fields.
*/

#[derive(Debug,Copy,Clone)]
struct BitField<const SIZE: usize> {
    value: u8,
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
pub trait AlignableBitField<const FIELD_SIZE: usize, const LSB_IDX: u8>: Into<BitField<FIELD_SIZE>> where Self: Clone {
  fn align_to_word(&self) -> u16 {
    let bit_field: BitField<FIELD_SIZE> = (self.clone()).into();
    (bit_field.value as u16) << LSB_IDX
  }
}

// For types that want to encapsulate contingent fields
// resulting in more than 8 bits when added.
#[derive(Debug,Copy,Clone)]
struct ComplexBitField<const SIZE: usize> {
    value: u16,
}

impl<const SIZE: usize> ComplexBitField<SIZE> {
    pub fn new(value: u16) -> Self {
        if SIZE > 16 {
            panic!("SIZE is too large for u16.");
        }
        assert!(value < (1 << SIZE), "Value exceeds the bitfield size");
        Self { value }
    }
}

// FIELD_SIZE in bits. LSB_IDX the position of the LSB of the BitField, which
// translates to the amount of bits to shift left.
pub trait AlignableComplexBitField<const FIELD_SIZE: usize, const LSB_IDX: u8>: Into<ComplexBitField<FIELD_SIZE>> where Self: Clone {
  fn align_to_word(&self) -> u16 {
    let bit_field: ComplexBitField<FIELD_SIZE> = (self.clone()).into();
    bit_field.value << LSB_IDX
  }
}


#[derive(Debug,Copy,Clone)]
pub enum WordFormat {
    CommandWord,
    DataWord,
    StatusWord,
}

#[derive(Debug,Copy,Clone)]
pub enum RTAddr {
    Single(u8),
    Broadcast,
}

// Not mutually exclusive yet. Probably need to model this better.
impl From<RTAddr> for BitField<5> {
    fn from(addr: RTAddr) -> Self {
        match addr {
            RTAddr::Single(val) => BitField::<5>::new(val),
            RTAddr::Broadcast => BitField::<5>::new(BROADCAST_ADDR),
        }
    }
}

impl AlignableBitField<5, 11> for RTAddr {}

#[derive(Debug,Copy,Clone)]
pub enum RTAction {
    Transmit,
    Receive,
}

impl From<RTAction> for u8 {
    fn from(action: RTAction) -> Self {
        match action {
            RTAction::Receive => 0b0,
            RTAction::Transmit => 0b1,
        }
    }
}

impl From<RTAction> for BitField<1> {
    fn from(action: RTAction) -> Self {
        BitField::<1>::new(u8::from(action))
    }
}

impl AlignableBitField<1, 10> for RTAction {}


/*
// Command Words.
*/

#[derive(Debug,Clone)]
struct CommandWord {
    rt_addr: RTAddr,       // Remote Terminal address 5 bit field.
    tr: RTAction,          // Transmit/Receive bit .
    data: CommandWordData, // Subaddress Mode 5 bit field.
}

impl Encode for CommandWord {
  fn encode(&self) -> u16 {
    let addr = self.rt_addr.align_to_word();
    let tr = self.tr.align_to_word();
    addr + tr + self.data.align_to_word()
  }
}

#[derive(Debug,Clone)]
enum CommandWordData {
    DataTransfer {
        subaddress: BitField<5>,
        word_count: BitField<5>,
    },
    ModeCode(ModeCode),
}

impl From<CommandWordData> for ComplexBitField<10> {
    fn from(data: CommandWordData) -> Self {
        match data {
          CommandWordData::DataTransfer{subaddress, word_count} => {
            let subaddr = (BitField::<5>::from(subaddress).value as u16) << 5;
            let wc = BitField::<5>::from(word_count).value as u16;
            ComplexBitField::new(subaddr + wc)
          },
          CommandWordData::ModeCode(mode_code) => {
            let subaddr = (SUBADDRESS_MODE_CODE_1 as u16) << 5;
            let code = u8::from(mode_code) as u16;
            ComplexBitField::new(subaddr + code)
          }
        }
    }
}


impl AlignableComplexBitField<10, 0> for CommandWordData {}

impl CommandWord {
    pub fn new_mode_command(rt_addr: RTAddr, tr: RTAction, code: ModeCode) -> Self {
        Self {
            rt_addr,
            tr,
            data: CommandWordData::ModeCode(code),
        }
    }

    pub fn new_data_transfer(rt_addr: RTAddr, tr: RTAction, subaddress: u8, word_count: u8) -> Self {
        let subaddress = BitField::new(subaddress);
        let word_count = BitField::new(word_count);
        Self {
            rt_addr,
            tr,
            data: CommandWordData::DataTransfer{subaddress, word_count},
        }
    }
}

/*
// Data Words.
*/

#[derive(Debug,Clone)]
struct DataWord {
    data: u16, // Data 16 bit field.
}

/*
// Status Words.
*/

#[derive(Debug,Clone)]
struct StatusWord {
    rt_addr: RTAddr,       // Remote Terminal address 5 bit field.
    mesg_err: BitField<1>, // Message Error bit.
    inst: BitField<1>,     // Instrumentation bit.
    svc_req: BitField<1>,  // Service request bit.
    bc_comm: BitField<1>,  // Broadcast Command bit.
    busy: BitField<1>,     // Busy bit.
    subsys: BitField<1>,   // Subsystem Flag bit.
    dbc: BitField<1>,      // Dynamic Bus Control bit.
    term: BitField<1>,     // Terminal Flag bit.
}

// TODO: Some of these codes REQUIRE the T/R bit to be set to 1 regardless of the direction of data flow.
// TODO: Implement From u8
#[derive(Debug,Clone)]
pub enum ModeCode {
    DynamicBusControl,
    Synchronize,
    TransmitStatusWord,
    InitiateSelfTest,
    TransmitterShutdown,
    OverrideTransmitter,
    InhibitTerminalFlagBit,
    OverrideInhibitTerminalFlagBit,
    ResetRT,
    TransmitVectorWord,
    SyncronizeWithDataWord,
    TransmitLastCommand,
    TransmitBITWord,
    SelectedTransmitter,
    OverrideSelectedTransmitter,
}

impl From<ModeCode> for u8 {
    fn from(code: ModeCode) -> Self {
        match code {
            ModeCode::DynamicBusControl => 0b00000,
            ModeCode::Synchronize => 0b00001,
            ModeCode::TransmitStatusWord => 0b00010,
            ModeCode::InitiateSelfTest => 0b00011,
            ModeCode::TransmitterShutdown => 0b00100,
            ModeCode::OverrideTransmitter => 0b00101,
            ModeCode::InhibitTerminalFlagBit => 0b00110,
            ModeCode::OverrideInhibitTerminalFlagBit => 0b00111,
            ModeCode::ResetRT => 0b01000,
            ModeCode::TransmitVectorWord => 0b10000,
            ModeCode::SyncronizeWithDataWord => 0b10001,
            ModeCode::TransmitLastCommand => 0b10010,
            ModeCode::TransmitBITWord => 0b10011,
            ModeCode::SelectedTransmitter => 0b10100,
            ModeCode::OverrideSelectedTransmitter => 0b10101,
        }
    }
}

impl From<ModeCode> for BitField<5> {
    fn from(code: ModeCode) -> Self {
        BitField::new(u8::from(code))
    }
}

fn main() {
    println!("");
    let cmd = CommandWord::new_mode_command(
        RTAddr::Single(23),
        RTAction::Transmit,
        ModeCode::TransmitLastCommand,
    );
    println!("{:?}", cmd);
    println!("{:b}", cmd.encode());
    let dt = CommandWord::new_data_transfer(
        RTAddr::Single(27),
        RTAction::Transmit,
        1,
        2,
    );
    println!("{:?}", dt);
    println!("{:b}", dt.encode());
    //assert!(
}
