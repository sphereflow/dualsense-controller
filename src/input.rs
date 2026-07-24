use bitflags::bitflags;
use zerocopy::{FromBytes, Immutable, IntoBytes, FromZeros, TryFromBytes};

use crate::button::Button;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromBytes, IntoBytes, Immutable, FromZeros, TryFromBytes)]
    #[repr(transparent)]
    pub struct ButtonsLow: u8 {
        const SQUARE   = 0x10;
        const CROSS    = 0x20;
        const CIRCLE   = 0x40;
        const TRIANGLE = 0x80;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromBytes, IntoBytes, Immutable, FromZeros, TryFromBytes)]
    #[repr(transparent)]
    pub struct ButtonsHigh: u8 {
        const L1       = 0x01;
        const R1       = 0x02;
        const L2       = 0x04;
        const R2       = 0x08;
        const CREATE   = 0x10;
        const MENU     = 0x20;
        const L3       = 0x40;
        const R3       = 0x80;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromBytes, IntoBytes, Immutable, FromZeros, TryFromBytes)]
    #[repr(transparent)]
    pub struct ButtonsMisc: u8 {
        const PS          = 0x01;
        const TOUCHPAD    = 0x02;
        const MUTE        = 0x04;
    }
}

#[repr(C, packed)]
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy, TryFromBytes)]
pub struct DualSenseInputUSB {
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub seq_number: u8,
    pub dpad_state: DPadState,
    pub buttons_low: ButtonsLow,
    pub buttons_high: ButtonsHigh,
    pub buttons_misc: ButtonsMisc,
}

#[repr(u8)]
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy, PartialEq, TryFromBytes)]
pub enum DPadState {
    Up = 0,
    UpRight = 1,
    Right = 2,
    DownRight = 3,
    Down = 4,
    DownLeft = 5,
    Left = 6,
    UpLeft = 7,
    Released = 8,
}

impl DualSenseInputUSB {
    pub fn is_button_down(&self, button: Button) -> bool {
        match button {
            Button::Square => self.buttons_low.contains(ButtonsLow::SQUARE),
            Button::Cross => self.buttons_low.contains(ButtonsLow::CROSS),
            Button::Circle => self.buttons_low.contains(ButtonsLow::CIRCLE),
            Button::Triangle => self.buttons_low.contains(ButtonsLow::TRIANGLE),
            Button::L1 => self.buttons_high.contains(ButtonsHigh::L1),
            Button::R1 => self.buttons_high.contains(ButtonsHigh::R1),
            Button::L2 => self.buttons_high.contains(ButtonsHigh::L2),
            Button::R2 => self.buttons_high.contains(ButtonsHigh::R2),
            Button::Create => self.buttons_high.contains(ButtonsHigh::CREATE),
            Button::Menu => self.buttons_high.contains(ButtonsHigh::MENU),
            Button::L3 => self.buttons_high.contains(ButtonsHigh::L3),
            Button::R3 => self.buttons_high.contains(ButtonsHigh::R3),
            Button::PS => self.buttons_misc.contains(ButtonsMisc::PS),
            Button::Touchpad => self.buttons_misc.contains(ButtonsMisc::TOUCHPAD),
            Button::Mute => self.buttons_misc.contains(ButtonsMisc::MUTE),
            Button::DpadUp => matches!(self.dpad_state, DPadState::Up | DPadState::UpLeft | DPadState::UpRight),
            Button::DpadDown => matches!(self.dpad_state, DPadState::Down | DPadState::DownLeft | DPadState::DownRight),
            Button::DpadLeft => matches!(self.dpad_state, DPadState::Left | DPadState::UpLeft | DPadState::DownLeft),
            Button::DpadRight => matches!(self.dpad_state, DPadState::Right | DPadState::UpRight | DPadState::DownRight),
        }
    }

    pub fn diff(&self, other: &Self) -> Vec<Button> {
        let mut diff = Vec::new();
        for button in [
            Button::Square,
            Button::Cross,
            Button::Circle,
            Button::Triangle,
            Button::L1,
            Button::R1,
            Button::L2,
            Button::R2,
            Button::Create,
            Button::Menu,
            Button::L3,
            Button::R3,
            Button::PS,
            Button::Touchpad,
            Button::Mute,
            Button::DpadUp,
            Button::DpadDown,
            Button::DpadLeft,
            Button::DpadRight,
        ] {
            if self.is_button_down(button) != other.is_button_down(button) {
                diff.push(button);
            }
        }
        diff
    }
}
