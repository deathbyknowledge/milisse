use crate::{primitives::BitField, words::*};

// Should the timing be implemented here? 
pub trait Bus {
    fn write_word(&mut self, value: Word);
    fn read_next(&self) -> Word;
}

pub struct BusController<'a> {
    bus: &'a mut dyn Bus,
}

impl BusController<'_> {
    pub fn send_broadcast_transfer(&mut self, data: &[u16]) -> Result<(), ()> {
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
            self.bus.write_word(Word::Data(*w));
        }
        Ok(())
    }

    pub fn send_transfer(&mut self, addr: RTAddr,  subaddr: BitField<5>, data: &[u16]) -> Result<(), ()> {
        let rcv_cmd = CommandWord::new_data_transfer(
            addr,
            RTAction::Receive,
            subaddr,
            (data.len() as u8).into(),
        );

        self.bus.write_word(Word::Command(rcv_cmd));
        for w in data {
            self.bus.write_word(Word::Data(*w));
        }
        Ok(())
    }
}
