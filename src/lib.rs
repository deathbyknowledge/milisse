#![no_std]

pub mod primitives;
pub mod words;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        use crate::primitives::*;
        use crate::words::*;

        let cmd = CommandWord::new_mode_command(
            RTAddr::Single(23),
            RTAction::Transmit,
            ModeCode::TransmitLastCommand,
        );
        assert_eq!(cmd.encode(), 0b1011111111110010);

        let dt = CommandWord::new_data_transfer(RTAddr::Single(27), RTAction::Receive, 1, 2);
        assert_eq!(dt.encode(), 0b1101100000100010);
    }
}
