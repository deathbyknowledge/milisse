use crate::primitives::*;

const SUBADDRESS_MODE_CODE_0: u8 = 0b00000; // Subaddress for mode code
const SUBADDRESS_MODE_CODE_1: u8 = 0b11111; // Subaddress for mode code
pub const BROADCAST_ADDR: u8 = 0b11111; // Address for Brodcast mode.

#[derive(Debug, Clone)]
pub enum Word {
    Command(CommandWord),
    Data(DataWord),
    Status(StatusWord),
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

// TODO: Some values dont' allow address to be Broadcast
#[derive(Debug, Copy, Clone, PartialEq)]
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
    SynchronizeWithDataWord,
    TransmitLastCommand,
    TransmitBITWord,
    SelectedTransmitter,
    OverrideSelectedTransmitter,
    Invalid, // out of bounds OR a Reserved value
}

/// Each Mode Code has different requirements for the T/R bit,
/// if there's an associated data word and if a Broadcast address is allowed.
pub struct ModeCodeOptions {
    pub tr: RTAction,
    pub requires_data_word: bool,
    pub broadcast_allowed: bool,
}
impl ModeCode {
    pub fn associated_options(&self) -> ModeCodeOptions {
        match *self {
            ModeCode::DynamicBusControl => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: false,
                broadcast_allowed: false,
            },
            ModeCode::Synchronize => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: false,
                broadcast_allowed: true,
            },
            ModeCode::TransmitStatusWord => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: false,
                broadcast_allowed: false,
            },
            ModeCode::InitiateSelfTest => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: false,
                broadcast_allowed: true,
            },
            ModeCode::TransmitterShutdown => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: false,
                broadcast_allowed: true,
            },
            ModeCode::OverrideTransmitter => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: false,
                broadcast_allowed: true,
            },
            ModeCode::InhibitTerminalFlagBit => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: false,
                broadcast_allowed: true,
            },
            ModeCode::OverrideInhibitTerminalFlagBit => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: false,
                broadcast_allowed: true,
            },
            ModeCode::ResetRT => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: false,
                broadcast_allowed: true,
            },
            ModeCode::TransmitVectorWord => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: true,
                broadcast_allowed: false,
            },
            ModeCode::SynchronizeWithDataWord => ModeCodeOptions {
                tr: RTAction::Receive,
                requires_data_word: true,
                broadcast_allowed: true,
            },
            ModeCode::TransmitLastCommand => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: true,
                broadcast_allowed: false,
            },
            ModeCode::TransmitBITWord => ModeCodeOptions {
                tr: RTAction::Transmit,
                requires_data_word: true,
                broadcast_allowed: false,
            },
            ModeCode::SelectedTransmitter => ModeCodeOptions {
                tr: RTAction::Receive,
                requires_data_word: true,
                broadcast_allowed: true,
            },
            ModeCode::OverrideSelectedTransmitter => ModeCodeOptions {
                tr: RTAction::Receive,
                requires_data_word: true,
                broadcast_allowed: true,
            },
            ModeCode::Invalid => unreachable!(),
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
            ModeCode::SynchronizeWithDataWord => 0b10001,
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
            0b10001 => ModeCode::SynchronizeWithDataWord,
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

/** Command Word structure (16 bits)
 *  Bits [15:11]: Remote Terminal (RT) Address.
 *  Bits [11:10]: Transmit/Receive bit.
 *  Bits [9:5]: Subaddress Mode.
 *  Bits [4:0]: Data Word Count / Mode Code.
**/
#[derive(Debug, Clone, Copy)]
pub struct CommandWord {
    raw_value: u16,
}

impl CommandWord {
    /// Initialize a new CommandWord as Mode Command. T/R bit is set
    /// by the ModeCode selected.
    pub fn new_mode_command(rt_addr: RTAddr, code: ModeCode) -> Self {
        let mut raw_value = rt_addr.align_to_word();
        raw_value += code.associated_options().tr.align_to_word();
        raw_value += CommandWordData::ModeCode(code).align_to_word();
        Self { raw_value }
    }

    /// Initialize a CommandWord from a u16.
    pub fn from_u16(value: u16) -> Self {
        Self { raw_value: value }
    }

    /// Return the CommandWord as a u16 (without sync waves and parity bit).
    pub fn value(&self) -> u16 {
        self.raw_value
    }

    /// Initialize a new CommandWord in Data Transfer mode.
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

    /// Set the T/R bit. If the CommandWord is a Mode Code Command,
    /// the provided T/R bit is ignored since the value is based on
    /// the Code value.
    pub fn set_tr_bit(&mut self, addr: RTAction) {
        if let CommandWordData::DataTransfer { .. } = self.get_command_data() {
            self.raw_value = addr.set_in(self.raw_value);
        }
    }

    pub fn get_command_data(&self) -> CommandWordData {
        return CommandWordData::read(self.raw_value);
    }

    /// Sets the Subaddress field to the Mode Code value
    /// and Word Data Count field to the provided code. It also
    /// sets the T/R bit for those codes that required a fixed 1.
    pub fn set_command_mode(&mut self, code: ModeCode) {
        let options = code.associated_options();

        // The Mode Code dictates the T/R bit value, so it must
        // be updated it. (Set manually)
        self.raw_value = options.tr.set_in(self.raw_value);

        // TODO: revisit this.
        // If the Mode Code does not allow a Broadcast address, validate
        // it's not being used. If it is, reset to 0b1 for now.
        if !options.broadcast_allowed && (self.get_rt_addr() == RTAddr::Broadcast) {
            self.set_rt_addr(RTAddr::Single(1.into()));
        }
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
// Status Words.
*/

// All status Flags

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageError(bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrumentation(bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRequest(bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroadcastCommand(bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Busy(bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subsystem(bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicBusControl(bool);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalFlag(bool);

// Allows to easily create Flag types as `struct MyFlag(bool)`
macro_rules! impl_flag_traits {
  ($($t:ty),*) => {
      $(
          impl From<$t> for BitField<1> {
              fn from(new_val: $t) -> Self {
                  new_val.0.into()
              }
          }

          impl From<BitField<1>> for $t {
              fn from(bitfield: BitField<1>) -> Self {
                  Self(bitfield.into())
              }
          }

          impl From<$t> for bool {
              fn from(new_val: $t) -> Self {
                  new_val.0
              }
          }

          impl From<bool> for $t {
              fn from(bitfield: bool) -> Self {
                  Self(bitfield)
              }
          }
      )*
  };
}
impl_flag_traits!(
    MessageError,
    Instrumentation,
    ServiceRequest,
    BroadcastCommand,
    Busy,
    Subsystem,
    DynamicBusControl,
    TerminalFlag
);

impl AlignableBitField<1, 10> for MessageError {}
impl AlignableBitField<1, 9> for Instrumentation {}
impl AlignableBitField<1, 8> for ServiceRequest {}
impl AlignableBitField<1, 4> for BroadcastCommand {}
impl AlignableBitField<1, 3> for Busy {}
impl AlignableBitField<1, 2> for Subsystem {}
impl AlignableBitField<1, 1> for DynamicBusControl {}
impl AlignableBitField<1, 0> for TerminalFlag {}

/** Status Word structure (16 bits)
 *  Bits [15:11]: Remote Terminal (RT) Address.
 *  Bits [11:10]: Message Error bit.
 *  Bits [10:9]: Instrumentation bit.
 *  Bits [9:8]: Service request bit.
 *  Bits [8:5]: Reserved.
 *  Bits [5:4]: Broadcast Command bit.
 *  Bits [4:3]: Busy bit.
 *  Bits [3:2]: Subsystem bit.
 *  Bits [2:1]: Dynamic Bus Control bit.
 *  Bits [1:0]: Terminal flag bit.
**/
#[derive(Debug, Clone)]
pub struct StatusWord {
    raw_value: u16,
}

impl StatusWord {
    pub fn new(
        addr: RTAddr,
        msg_err: MessageError,
        inst: Instrumentation,
        svc_req: ServiceRequest,
        bc_cmd: BroadcastCommand,
        busy: Busy,
        subsys: Subsystem,
        dbc: DynamicBusControl,
        terminal: TerminalFlag,
    ) -> Self {
        let value = addr.align_to_word()
            + msg_err.align_to_word()
            + inst.align_to_word()
            + svc_req.align_to_word()
            + bc_cmd.align_to_word()
            + busy.align_to_word()
            + subsys.align_to_word()
            + dbc.align_to_word()
            + terminal.align_to_word();
        Self { raw_value: value }
    }

    pub fn value(&self) -> u16 {
        self.raw_value
    }

    pub fn from_u16(value: u16) -> Self {
        Self { raw_value: value }
    }

    pub fn get_rt_addr(&self) -> RTAddr {
        return RTAddr::read(self.raw_value);
    }

    pub fn set_rt_addr(&mut self, addr: RTAddr) {
        self.raw_value = addr.set_in(self.raw_value)
    }

    pub fn get_message_error(&self) -> MessageError {
        return MessageError::read(self.raw_value);
    }

    pub fn set_message_error(&mut self, flag: MessageError) {
        self.raw_value = flag.set_in(self.raw_value)
    }

    pub fn get_instrumentation(&self) -> Instrumentation {
        return Instrumentation::read(self.raw_value);
    }

    pub fn set_instrumentation(&mut self, flag: Instrumentation) {
        self.raw_value = flag.set_in(self.raw_value)
    }

    pub fn get_service_request(&self) -> ServiceRequest {
        return ServiceRequest::read(self.raw_value);
    }

    pub fn set_service_request(&mut self, flag: Instrumentation) {
        self.raw_value = flag.set_in(self.raw_value)
    }

    pub fn get_broadcast_command(&self) -> BroadcastCommand {
        return BroadcastCommand::read(self.raw_value);
    }

    pub fn set_broadcast_command(&mut self, flag: BroadcastCommand) {
        self.raw_value = flag.set_in(self.raw_value)
    }

    pub fn get_busy(&self) -> Busy {
        return Busy::read(self.raw_value);
    }

    pub fn set_busy(&mut self, flag: Busy) {
        self.raw_value = flag.set_in(self.raw_value)
    }

    pub fn get_subsystem(&self) -> Subsystem {
        return Subsystem::read(self.raw_value);
    }

    pub fn set_subsystem(&mut self, flag: Subsystem) {
        self.raw_value = flag.set_in(self.raw_value)
    }

    pub fn get_dynamic_bus_control(&self) -> DynamicBusControl {
        return DynamicBusControl::read(self.raw_value);
    }

    pub fn set_dynamic_bus_control(&mut self, flag: DynamicBusControl) {
        self.raw_value = flag.set_in(self.raw_value)
    }

    pub fn get_terminal_flag(&self) -> TerminalFlag {
        return TerminalFlag::read(self.raw_value);
    }

    pub fn set_terminal_flag(&mut self, flag: TerminalFlag) {
        self.raw_value = flag.set_in(self.raw_value)
    }
}

/*
// Data Words.
*/

#[derive(Debug, Clone)]
pub struct DataWord {
    raw_value: u16, // Data 16 bit field.
}

impl DataWord {
    pub fn from_u16(value: u16) -> Self {
        Self { raw_value: value }
    }

    pub fn value(&self) -> u16 {
        self.raw_value
    }

    pub fn set_value(&mut self, value: u16) {
        self.raw_value = value
    }
}

#[cfg(test)]
mod tests {

    use crate::words::*;
    #[test]
    fn command_mode_word() {
        let mut cmd =
            CommandWord::new_mode_command(RTAddr::Single(23.into()), ModeCode::TransmitLastCommand);
        assert_eq!(cmd.value(), 0b1011111111110010);
        assert_eq!(cmd.get_rt_addr(), RTAddr::Single(23.into()));
        assert_eq!(cmd.get_tr_bit(), RTAction::Transmit);
        assert_eq!(
            cmd.get_command_data(),
            CommandWordData::ModeCode(ModeCode::TransmitLastCommand)
        );

        cmd.set_rt_addr(RTAddr::Single(11.into()));
        assert_eq!(cmd.get_rt_addr(), RTAddr::Single(11.into()));
        cmd.set_command_mode(ModeCode::ResetRT);
        assert_eq!(
            cmd.get_command_data(),
            CommandWordData::ModeCode(ModeCode::ResetRT)
        );
    }

    #[test]
    fn command_mode_to_data_transfer() {
        let mut word =
            CommandWord::new_mode_command(RTAddr::Single(23.into()), ModeCode::TransmitLastCommand);
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
        let mut cmd =
            CommandWord::new_mode_command(RTAddr::Single(23.into()), ModeCode::TransmitLastCommand);

        assert_eq!(cmd.get_tr_bit(), RTAction::Transmit);
        cmd.set_command_mode(ModeCode::SynchronizeWithDataWord);
        assert_eq!(cmd.get_tr_bit(), RTAction::Receive);
    }

    #[test]
    fn status_word_works() {
        let word = StatusWord::new(
            RTAddr::Single(21.into()),
            false.into(),
            true.into(),
            false.into(),
            true.into(),
            false.into(),
            true.into(),
            false.into(),
            true.into(),
        );
        assert_eq!(word.value(), 0b1010101000010101);
    }
}
