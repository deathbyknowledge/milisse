use crate::primitives::*;

const SUBADDRESS_MODE_CODE_0: u8 = 0b00000; // Subaddress for mode code
const SUBADDRESS_MODE_CODE_1: u8 = 0b11111; // Subaddress for mode code
const BROADCAST_ADDR: u8 = 0b11111; // Address for Brodcast mode.

#[derive(Debug, Copy, Clone)]
pub enum WordFormat {
    CommandWord,
    DataWord,
    StatusWord,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RTAddr {
    Single(BitField<5>),
    Broadcast,
}

// Not mutually exclusive yet. Probably need to model this better.
impl From<RTAddr> for BitField<5> {
    fn from(addr: RTAddr) -> Self {
        match addr {
            RTAddr::Single(val) => val,
            RTAddr::Broadcast => Self::new(BROADCAST_ADDR),
        }
    }
}

impl From<BitField<5>> for RTAddr {
    fn from(bitfield: BitField<5>) -> Self {
        if BROADCAST_ADDR == bitfield.into() {
            RTAddr::Broadcast
        } else {
            RTAddr::Single(bitfield.into())
        }
    }
}

impl AlignableBitField<5, 11> for RTAddr {}

#[derive(Debug, Copy, Clone, PartialEq)]
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

impl From<BitField<1>> for RTAction {
    fn from(bitfield: BitField<1>) -> Self {
        if 1u8 == bitfield.into() {
            RTAction::Transmit
        } else {
            RTAction::Receive
        }
    }
}

impl AlignableBitField<1, 10> for RTAction {}

/*
// Command Words.
*/

#[derive(Debug, Clone, PartialEq)]
pub enum CommandWordData {
    DataTransfer {
        subaddress: BitField<5>,
        word_count: BitField<5>,
    },
    ModeCode(ModeCode),
}

impl From<CommandWordData> for ComplexBitField<10> {
    fn from(data: CommandWordData) -> Self {
        match data {
            CommandWordData::DataTransfer {
                subaddress,
                word_count,
            } => {
                let subaddress: u16 = subaddress.into();
                let word_count: u16 = word_count.into();
                ((subaddress << 5) + word_count).into()
            }
            CommandWordData::ModeCode(mode_code) => {
                let subaddr = (SUBADDRESS_MODE_CODE_1 as u16) << 5;
                let code = u8::from(mode_code) as u16;
                (subaddr + code).into()
            }
        }
    }
}

impl From<ComplexBitField<10>> for CommandWordData {
    fn from(bitfield: ComplexBitField<10>) -> Self {
        let subaddr = (bitfield.value() >> 5) as u8;
        let wdc = (bitfield.value() & 0b11111) as u8;
        if subaddr == SUBADDRESS_MODE_CODE_0 || subaddr == SUBADDRESS_MODE_CODE_1 {
            CommandWordData::ModeCode(ModeCode::from(wdc))
        } else {
            CommandWordData::DataTransfer {
                subaddress: subaddr.into(),
                word_count: wdc.into(),
            }
        }
    }
}

impl AlignableComplexBitField<10, 0> for CommandWordData {}

#[derive(Debug, Clone)]
pub struct CommandWord {
    raw_value: Word,
    // Remote Terminal address 5 bit field.
    // Transmit/Receive bit .
}

impl CommandWord {
    pub fn new_mode_command(rt_addr: RTAddr, tr: RTAction, code: ModeCode) -> Self {
        let mut raw_value = rt_addr.align_to_word();
        raw_value += tr.align_to_word();
        raw_value += CommandWordData::ModeCode(code).align_to_word();
        Self { raw_value }
    }

    pub fn from_u16(value: Word) -> Self {
        Self { raw_value: value }
    }

    // Return the Word as a u16 (omitting sync waves and parity bit).
    pub fn value(&self) -> Word {
        self.raw_value
    }

    pub fn new_data_transfer(
        rt_addr: RTAddr,
        tr: RTAction,
        subaddress: BitField<5>,
        word_count: BitField<5>,
    ) -> Self {
        let mut raw_value = rt_addr.align_to_word();
        raw_value += tr.align_to_word();
        raw_value += CommandWordData::DataTransfer {
            subaddress,
            word_count,
        }
        .align_to_word();
        Self { raw_value }
    }

    pub fn get_rt_addr(&self) -> RTAddr {
        return RTAddr::read(self.raw_value);
    }

    pub fn set_rt_addr(&mut self, addr: RTAddr) {
        self.raw_value = addr.set_in(self.raw_value)
    }

    pub fn get_tr_bit(&self) -> RTAction {
        return RTAction::read(self.raw_value);
    }

    pub fn set_tr_bit(&mut self, addr: RTAction) {
        self.raw_value = addr.set_in(self.raw_value)
    }

    pub fn get_command_data(&self) -> CommandWordData {
        return CommandWordData::read(self.raw_value);
    }

    /// Sets the Subaddress field to the Mode Code value
    /// and Word Data Count field to the provided code. It also
    /// sets the T/R bit for those codes that required a fixed 1.
    pub fn set_command_mode(&mut self, code: ModeCode) {
        // Depending on the mode code, the T/R bit must be
        // set to a fixed 1.
        self.raw_value = code.tr_bit().set_in(self.raw_value);
        self.raw_value = CommandWordData::ModeCode(code).set_in(self.raw_value);
    }

    pub fn set_data_transfer(&mut self, subaddress: BitField<5>, word_count: BitField<5>) {
        let data = CommandWordData::DataTransfer {
            subaddress,
            word_count,
        };
        self.raw_value = data.set_in(self.raw_value)
    }
}

impl From<u16> for CommandWord {
    fn from(value: u16) -> Self {
        CommandWord::from_u16(value)
    }
}

/*
// Data Words.
*/

#[derive(Debug, Clone)]
struct DataWord {
    data: u16, // Data 16 bit field.
}

/*
// Status Words.
*/

#[derive(Debug, Clone)]
pub struct StatusWord {
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

// TODO: Implement From u8
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    Invalid, // out of bounds OR a Reserved value
}

impl ModeCode {
    fn tr_bit(&self) -> RTAction {
        // Each mode code requires a different T/R bit setting,
        // this is a quick mapping to get the desired value.
        match Into::<u8>::into(*self) {
            0..=0b10000 => RTAction::Transmit,
            0b10001 => RTAction::Receive,
            0b10010..=0b10011 => RTAction::Transmit,
            0b10100..=0b10101 => RTAction::Receive,
            0b10110..=u8::MAX => unimplemented!(), // RESERVED values.
        }
    }
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
            ModeCode::Invalid => 0b11111, // one of many possible values
        }
    }
}

impl From<u8> for ModeCode {
    fn from(value: u8) -> Self {
        match value {
            0b00000 => ModeCode::DynamicBusControl,
            0b00001 => ModeCode::Synchronize,
            0b00010 => ModeCode::TransmitStatusWord,
            0b00011 => ModeCode::InitiateSelfTest,
            0b00100 => ModeCode::TransmitterShutdown,
            0b00101 => ModeCode::OverrideTransmitter,
            0b00110 => ModeCode::InhibitTerminalFlagBit,
            0b00111 => ModeCode::OverrideInhibitTerminalFlagBit,
            0b01000 => ModeCode::ResetRT,
            0b10000 => ModeCode::TransmitVectorWord,
            0b10001 => ModeCode::SyncronizeWithDataWord,
            0b10010 => ModeCode::TransmitLastCommand,
            0b10011 => ModeCode::TransmitBITWord,
            0b10100 => ModeCode::SelectedTransmitter,
            0b10101 => ModeCode::OverrideSelectedTransmitter,
            9_u8..=15_u8 | 22_u8..=u8::MAX => ModeCode::Invalid,
        }
    }
}

impl From<ModeCode> for BitField<5> {
    fn from(code: ModeCode) -> Self {
        BitField::new(code.into())
    }
}

#[cfg(test)]
mod tests {

    use crate::words::*;
    #[test]
    fn command_mode_word() {
        let mut cmd = CommandWord::new_mode_command(
            RTAddr::Single(23.into()),
            RTAction::Transmit,
            ModeCode::TransmitLastCommand,
        );
        assert_eq!(cmd.value(), 0b1011111111110010);
        assert_eq!(cmd.get_rt_addr(), RTAddr::Single(23.into()));
        assert_eq!(cmd.get_tr_bit(), RTAction::Transmit);
        assert_eq!(
            cmd.get_command_data(),
            CommandWordData::ModeCode(ModeCode::TransmitLastCommand)
        );

        cmd.set_rt_addr(RTAddr::Single(11.into()));
        assert_eq!(cmd.get_rt_addr(), RTAddr::Single(11.into()));
        cmd.set_tr_bit(RTAction::Receive);
        assert_eq!(cmd.get_tr_bit(), RTAction::Receive);
        cmd.set_command_mode(ModeCode::ResetRT);
        assert_eq!(
            cmd.get_command_data(),
            CommandWordData::ModeCode(ModeCode::ResetRT)
        );
    }

    #[test]
    fn command_mode_to_data_transfer() {
        let mut word = CommandWord::new_mode_command(
            RTAddr::Single(23.into()),
            RTAction::Transmit,
            ModeCode::TransmitLastCommand,
        );
        word.set_data_transfer(12.into(), 1.into());
        assert_eq!(
            word.get_command_data(),
            CommandWordData::DataTransfer {
                subaddress: 12.into(),
                word_count: 1.into()
            }
        );
    }

    #[test]
    fn command_data_transfer_word() {
        let dt = CommandWord::new_data_transfer(
            RTAddr::Single(27.into()),
            RTAction::Receive,
            1.into(),
            2.into(),
        );
        assert_eq!(dt.value(), 0b1101100000100010);
        assert_eq!(dt.get_rt_addr(), RTAddr::Single(27.into()));
        assert_eq!(dt.get_tr_bit(), RTAction::Receive);
        assert_eq!(
            dt.get_command_data(),
            CommandWordData::DataTransfer {
                subaddress: 1.into(),
                word_count: 2.into()
            }
        );
    }
    #[test]
    fn command_code_proper_tr_bit() {
        todo!();
    }
}
