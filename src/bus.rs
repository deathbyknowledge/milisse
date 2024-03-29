use crate::{primitives::BitField, words::*};

// Should the timing be implemented here?
pub trait Bus {
    fn write_word(&mut self, value: Word);
    // TODO: this should probably not be blocking
    fn read_next(&self) -> Word;
}

pub struct BusController<'a> {
    bus: &'a mut dyn Bus,
}

impl BusController<'_> {
    pub fn send_broadcast_transfer(&mut self, data: &[DataWord]) -> Result<(), ()> {
        if data.len() > 31 {
            return Err(());
        }
        let rcv_cmd = CommandWord::new_data_transfer(
            RTAddr::Broadcast,
            RTAction::Receive,
            1.into(),
            (data.len() as u8).into(),
        );

        self.bus.write_word(Word::Command(rcv_cmd));
        for w in data {
            self.bus.write_word(Word::Data((*w).clone()));
        }
        Ok(())
    }

    pub fn send_transfer(
        &mut self,
        addr: RTAddr,
        subaddr: BitField<5>,
        data: &[DataWord],
    ) -> Result<(), ()> {
        // Broadcast transfer alias
        if addr == RTAddr::Single(BROADCAST_ADDR.into()) || addr == RTAddr::Broadcast {
            self.send_broadcast_transfer(data)?
        }

        if data.len() > 31 {
            return Err(());
        }
        let rcv_cmd = CommandWord::new_data_transfer(
            addr,
            RTAction::Receive,
            subaddr,
            (data.len() as u8).into(),
        );

        self.bus.write_word(Word::Command(rcv_cmd));
        for w in data {
            self.bus.write_word(Word::Data((*w).clone()));
        }
        Ok(())
    }

    pub fn send_mode_command(
        &mut self,
        addr: RTAddr,
        code: ModeCode,
        /*IN/OUT*/ data: Option<&mut DataWord>,
    ) -> Result<Option<StatusWord>, ()> {
        let mode_command = CommandWord::new_mode_command(addr, code);
        let options = code.associated_options();
        if !options.broadcast_allowed && addr == RTAddr::Broadcast {
            // The selected mode code does not allow a Broadcast address.
            return Err(());
        }

        if let (true, true, RTAction::Receive) = (data.is_none(), options.requires_data_word, options.tr) {
            // The selected mode requires the Bus Controller to transmit a data word
            // but none was provided.
            return Err(());
        }

        self.bus.write_word(Word::Command(mode_command));

        if options.requires_data_word {
            match options.tr {
                RTAction::Transmit => {
                    let sw = match self.bus.read_next() {
                        Word::Command(_) => todo!(), // should not
                        Word::Data(_) => todo!(),    // should not
                        Word::Status(sw) => sw,
                    };
                    let dw = match self.bus.read_next() {
                        Word::Command(_) => todo!(), // should not
                        Word::Data(dw) => dw,
                        Word::Status(_) => todo!(), // should not
                    };
                    data.unwrap().set_value(dw.value()); // set DataWord value
                    return Ok(Some(sw));
                }
                RTAction::Receive => self.bus.write_word(Word::Data(data.unwrap().clone())),
            }
        }
        let sw = match self.bus.read_next() {
            Word::Command(_) => todo!(), // should not
            Word::Data(_) => todo!(),    // should not
            Word::Status(sw) => sw,
        };
        Ok(Some(sw))
    }
}
