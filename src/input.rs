use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::button::Button;

#[repr(C, packed)]
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy, Default)]
pub struct DualSenseInputReportSimpleBT {
    report_id: u8, // 0x01
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub buttons_low: u8,  // D-pad and Shapes
    pub buttons_high: u8, // L1, R1, L2, R2, Create, Menu, L3, R3
    pub buttons_misc: u8, // Home1 Pad1 Counter6
    pub l2_axis: u8,
    pub r2_axis: u8,
}

impl From<DualSenseInputReportSimpleBT> for DualSenseInputUSB {
    fn from(value: DualSenseInputReportSimpleBT) -> Self {
        DualSenseInputUSB {
            left_stick_x: value.left_stick_x,
            left_stick_y: value.left_stick_y,
            right_stick_x: value.right_stick_x,
            right_stick_y: value.right_stick_y,
            buttons_low: value.buttons_low,
            buttons_high: value.buttons_high,
            buttons_misc: value.buttons_misc,
            l2_axis: value.l2_axis,
            r2_axis: value.r2_axis,
            ..Default::default()
        }
    }
}

#[repr(C, packed)]
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy, Default)]
pub struct DualSenseInputReportBT {
    report_id: u8,
    pub flags_and_seq: u8, // HasHID1, HasMic1, Unknown2, SeqNumber4
    pub base: DualSenseInputUSB,
}

impl From<DualSenseInputReportBT> for DualSenseInputUSB {
    fn from(value: DualSenseInputReportBT) -> Self {
        value.base
    }
}

#[repr(C, packed)]
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy, Default)]
pub struct DualSenseInputUSB {
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub l2_axis: u8,
    pub r2_axis: u8,
    pub seq_number: u8,
    pub buttons_low: u8,    // D-pad and Shapes
    pub buttons_high: u8,   // L1, R1, L2, R2, Create, Menu, L3, R3
    pub buttons_misc: u8,   // PS, TouchClick, Mute
    pub unknown_1: [u8; 5], // 10
    // --- Motion Sensors (IMU) ---
    pub gyro_x: i16,           // 15-16
    pub gyro_y: i16,           // 17-18
    pub gyro_z: i16,           // 19-20
    pub accel_x: i16,          // 21-22
    pub accel_y: i16,          // 23-24
    pub accel_z: i16,          // 25-26
    pub sensor_timestamp: u32, // 27-30
    pub temperature: u8,       // 31
    // --- Touchpad Data ---
    pub touch_1: TouchData,    // 32-35
    pub touch_2: TouchData,    // 36-40
    pub trigger_right: u8,     // 41 TriggerRightStop4 TriggerRightStatus4
    pub trigger_left: u8,      // 42 TriggerLeftStop4 TriggerLeftStatus4
    pub host_timestamp: u32,   // 43-46
    pub trigger_effects: u8,   // 47 TriggerRightEffect4 TriggerLeftEffect4
    pub device_timestamp: u32, // 48-51
    pub power: u8,             // 52 PowerPercent4 PowerState4
    pub misc_flags_1: u8,      // 53
    pub misc_flags_2: u8,      // 54
    pub aes_cmac: u8,          // 55
}

impl DualSenseInputUSB {
    pub fn is_button_down(&self, b: Button) -> bool {
        match b {
            // D-pad: treat diagonal positions as also pressing both relevant directions
            Button::DpadUp => matches!(
                self.dpad(),
                DPadState::North | DPadState::NorthEast | DPadState::NorthWest
            ),
            Button::DpadRight => matches!(
                self.dpad(),
                DPadState::East | DPadState::NorthEast | DPadState::SouthEast
            ),
            Button::DpadDown => matches!(
                self.dpad(),
                DPadState::South | DPadState::SouthEast | DPadState::SouthWest
            ),
            Button::DpadLeft => matches!(
                self.dpad(),
                DPadState::West | DPadState::NorthWest | DPadState::SouthWest
            ),

            // Face buttons
            Button::Square => (self.buttons_low & 0x10) != 0,
            Button::Cross => (self.buttons_low & 0x20) != 0,
            Button::Circle => (self.buttons_low & 0x40) != 0,
            Button::Triangle => (self.buttons_low & 0x80) != 0,

            // Shoulders & triggers
            Button::L1 => (self.buttons_high & 0x01) != 0,
            Button::R1 => (self.buttons_high & 0x02) != 0,
            Button::L2 => (self.buttons_high & 0x04) != 0,
            Button::R2 => (self.buttons_high & 0x08) != 0,
            Button::L3 => (self.buttons_high & 0x40) != 0,
            Button::R3 => (self.buttons_high & 0x80) != 0,

            // System/misc buttons
            Button::PS => (self.buttons_misc & 0x01) != 0,
            Button::Touchpad => (self.buttons_misc & 0x02) != 0,
            Button::Mute => (self.buttons_misc & 0x04) != 0,
            Button::Create => (self.buttons_high & 0x10) != 0,
            Button::Menu => (self.buttons_high & 0x20) != 0,
        }
    }

    // --- Analog Stick Axes ---
    pub fn left_stick(&self) -> (u8, u8) {
        (self.left_stick_x, self.left_stick_y)
    }
    pub fn right_stick(&self) -> (u8, u8) {
        (self.right_stick_x, self.right_stick_y)
    }

    // --- Trigger Axes (0-255) ---
    pub fn l2_axis(&self) -> u8 {
        self.l2_axis
    }
    pub fn r2_axis(&self) -> u8 {
        self.r2_axis
    }

    // --- D-Pad (Hat Switch) ---
    /// Returns: 0:N, 1:NE, 2:E, 3:SE, 4:S, 5:SW, 6:W, 7:NW, 8:Released
    pub fn dpad(&self) -> DPadState {
        DPadState::from(self.buttons_low & 0x0F)
    }

    // --- System Buttons (buttons_misc) ---
    pub fn ps(&self) -> bool {
        (self.buttons_misc & 0x01) != 0
    }
    pub fn touchpad_click(&self) -> bool {
        (self.buttons_misc & 0x02) != 0
    }
    pub fn mute(&self) -> bool {
        (self.buttons_misc & 0x04) != 0
    }

    pub fn battery_state(&self) -> PowerState {
        PowerState::from(self.power)
    }

    /// new.diff(&old).is_button_down() => button pressed
    /// old.diff(&new).is_button_down() => button released
    pub fn diff(&self, other: &DualSenseInputUSB) -> DualSenseInputUSB {
        let mut diff = *self;
        diff.buttons_low = self.buttons_low & !other.buttons_low;
        // dpad is handled differently
        diff.buttons_low &= 0xF0;
        diff.buttons_low |= self.dpad().diff(&other.dpad()) as u8;
        diff.buttons_high = self.buttons_high & !other.buttons_high;
        diff.buttons_misc = self.buttons_misc & !other.buttons_misc;
        diff
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DPadState {
    North = 0,
    NorthEast = 1,
    East = 2,
    SouthEast = 3,
    South = 4,
    SouthWest = 5,
    West = 6,
    NorthWest = 7,
    Released = 8,
}

impl DPadState {
    fn diff(&self, other: &DPadState) -> DPadState {
        let diff_3 = |a: DPadState, b: DPadState, c: DPadState| {
            (*self == a || *self == b || *self == c) && (*other != a && *other != b && *other != c)
        };
        let diff_north = diff_3(DPadState::North, DPadState::NorthEast, DPadState::NorthWest);
        let diff_east = diff_3(DPadState::East, DPadState::NorthEast, DPadState::SouthEast);
        let diff_south = diff_3(DPadState::South, DPadState::SouthEast, DPadState::SouthWest);
        let diff_west = diff_3(DPadState::West, DPadState::NorthWest, DPadState::SouthWest);
        match (diff_north, diff_east, diff_south, diff_west) {
            (true, true, _, _) => DPadState::NorthEast,
            (_, true, true, _) => DPadState::SouthEast,
            (_, _, true, true) => DPadState::SouthWest,
            (true, _, _, true) => DPadState::NorthWest,
            (true, _, _, _) => DPadState::North,
            (_, true, _, _) => DPadState::East,
            (_, _, true, _) => DPadState::South,
            (_, _, _, true) => DPadState::West,
            _ => DPadState::Released,
        }
    }
}

impl From<u8> for DPadState {
    fn from(value: u8) -> Self {
        match value {
            0 => DPadState::North,
            1 => DPadState::NorthEast,
            2 => DPadState::East,
            3 => DPadState::SouthEast,
            4 => DPadState::South,
            5 => DPadState::SouthWest,
            6 => DPadState::West,
            7 => DPadState::NorthWest,
            _ => DPadState::Released, // HID value 8 and any undefined values
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    Discharging(u8), // Percentage (roughly 0-100)
    Charging(u8),    // Percentage
    Full,
    Unknown,
}

impl From<u8> for PowerState {
    fn from(value: u8) -> Self {
        // Bit 4 (0x10) usually indicates charging status
        let is_charging = (value & 0x10) != 0;
        let level = value & 0x0F; // Lower 4 bits are the level (0-10)

        let percentage = (level * 10).min(100);

        match (is_charging, level) {
            (true, 10) => PowerState::Full,
            (true, _) => PowerState::Charging(percentage),
            (false, _) => PowerState::Discharging(percentage),
        }
    }
}

#[repr(C, packed)]
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy, Default)]
pub struct TouchData {
    pub contact_id: u8, // Bit 7 is "up" (1) or "down" (0)
    pub x_low: u8,
    pub y_low_x_high: u8, // Packed bits
    pub y_high: u8,
}

impl TouchData {
    pub fn is_active(&self) -> bool {
        (self.contact_id & 0x80) == 0
    }
    pub fn x(&self) -> u16 {
        ((self.y_low_x_high as u16 & 0x0F) << 8) | self.x_low as u16
    }
    pub fn y(&self) -> u16 {
        ((self.y_high as u16) << 4) | ((self.y_low_x_high as u16 & 0xF0) >> 4)
    }
}

#[repr(C, packed)]
#[derive(FromBytes, Debug, Clone, Copy)]
pub struct DualSenseInputReport31 {
    pub report_id: u8, // 0x31
    pub tag: u8,       // Usually 0xA1 (BT header tag)

    // Reuse your 0x01 logic here (ensure 0x01 struct DOES NOT have report_id field)
    pub base: DualSenseInputUSB,

    pub counter: u8,    // Changes every report
    pub r2_axis: u8,    // Duplicated in some firmwares
    pub l2_axis: u8,    // Duplicated in some firmwares
    pub timestamp: u32, // Controller internal clock

    // IMU Data (Gyro/Accel)
    pub gyro_x: i16,
    pub gyro_y: i16,
    pub gyro_z: i16,
    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,

    pub sensor_timestamp: u32,
    pub padding_1: u8,

    // Touchpad Data
    pub touch_1: TouchData,
    pub touch_2: TouchData,
    pub padding_2: [u8; 8],

    // Power/Status
    pub battery_level: u8, // 0-10 (approx), bit 4 is "charging"
    pub status_flags: u8,
    pub padding_3: [u8; 9],

    pub crc32: u32, // Bluetooth requires this checksum
}
