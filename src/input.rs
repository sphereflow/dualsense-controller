use bitflags::bitflags;
use zerocopy::{FromBytes, Immutable, IntoBytes};

use crate::button::Button;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct ButtonsLow: u8 {
        const SQUARE   = 0x10;
        const CROSS    = 0x20;
        const CIRCLE   = 0x40;
        const TRIANGLE = 0x80;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct ButtonsHigh: u8 {
        const L1       = 0x01;
        const R1       = 0x02;
        const L2       = 0x04;
        const R2       = 0x08;
        const CREATE   = 0x10;
        const MENU     = 0x20;
        const L3       = 0x40;
        const R3       = 0x80;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct ButtonsMisc: u8 {
        const PS          = 0x01;
        const TOUCHPAD    = 0x02;
        const MUTE        = 0x04;
    }
}

/// Bluetooth input report
#[repr(C, packed)]
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy, Default)]
pub struct DualSenseInputReportSimpleBT {
    report_id: u8, // INPUT_REPORT_SIMPLE_BT_ID
    /// X position fo the left stick
    pub left_stick_x: u8,
    /// Y position fo the left stick
    pub left_stick_y: u8,
    /// X position fo the right stick
    pub right_stick_x: u8,
    /// Y position fo the right stick
    pub right_stick_y: u8,
    /// DPad an shape buttons
    pub buttons_low: u8,
    /// L1, R1, L2, R2, Create, Menu, L3, R3 buttons
    pub buttons_high: u8,
    /// All the other buttons
    pub buttons_misc: u8,
    /// Variable position data of the L2 button
    pub l2_axis: u8,
    /// Variable position data of the R2 button
    pub r2_axis: u8,
}

impl From<DualSenseInputReportSimpleBT> for DualSenseInput {
    fn from(value: DualSenseInputReportSimpleBT) -> Self {
        DualSenseInput {
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
pub(crate) struct DualSenseInputReportBT {
    report_id: u8,                // INPUT_REPORT_BT_ID
    pub(crate) flags_and_seq: u8, // HasHID1, HasMic1, Unknown2, SeqNumber4
    pub(crate) base: DualSenseInput,
}

impl From<DualSenseInputReportBT> for DualSenseInput {
    fn from(value: DualSenseInputReportBT) -> Self {
        value.base
    }
}

/// Default deadzone of 10 %
pub const DEFAULT_DEADZONE: f32 = 0.1;

/// Base struct for input reports
#[repr(C, packed)]
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy, Default)]
pub struct DualSenseInput {
    left_stick_x: u8,
    left_stick_y: u8,
    right_stick_x: u8,
    right_stick_y: u8,
    l2_axis: u8,
    r2_axis: u8,
    seq_number: u8,
    buttons_low: u8,    // D-pad and Shapes
    buttons_high: u8,   // L1, R1, L2, R2, Create, Menu, L3, R3
    buttons_misc: u8,   // PS, TouchClick, Mute
    unknown_1: [u8; 5], // 10
    // --- Motion Sensors (IMU) ---
    /// X coordinate for the gyro
    pub gyro_x: i16, // 15-16
    /// Y coordinate for the gyro
    pub gyro_y: i16, // 17-18
    /// Z coordinate for the gyro
    pub gyro_z: i16, // 19-20
    /// X coordinate for the accelerometer
    pub accel_x: i16, // 21-22
    /// Y coordinate for the accelerometer
    pub accel_y: i16, // 23-24
    /// Z coordinate for the accelerometer
    pub accel_z: i16, // 25-26
    sensor_timestamp: u32, // 27-30
    temperature: u8,       // 31
    // --- Touchpad Data ---
    /// Touch data for one finger
    pub touch_1: TouchData, // 32-35
    /// Touch data for a second finger
    pub touch_2: TouchData, // 36-40
    trigger_right: u8,     // 41 TriggerRightStop4 TriggerRightStatus4
    trigger_left: u8,      // 42 TriggerLeftStop4 TriggerLeftStatus4
    host_timestamp: u32,   // 43-46
    trigger_effects: u8,   // 47 TriggerRightEffect4 TriggerLeftEffect4
    device_timestamp: u32, // 48-51
    power: u8,             // 52 PowerPercent4 PowerState4
    misc_flags_1: u8,      // 53
    misc_flags_2: u8,      // 54
    aes_cmac: u8,          // 55
}

impl DualSenseInput {
    /// Returns true if a [Button] is currently being held down
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
            Button::Square => {
                ButtonsLow::from_bits_truncate(self.buttons_low).contains(ButtonsLow::SQUARE)
            }
            Button::Cross => {
                ButtonsLow::from_bits_truncate(self.buttons_low).contains(ButtonsLow::CROSS)
            }
            Button::Circle => {
                ButtonsLow::from_bits_truncate(self.buttons_low).contains(ButtonsLow::CIRCLE)
            }
            Button::Triangle => {
                ButtonsLow::from_bits_truncate(self.buttons_low).contains(ButtonsLow::TRIANGLE)
            }

            // Shoulders & triggers
            Button::L1 => {
                ButtonsHigh::from_bits_truncate(self.buttons_high).contains(ButtonsHigh::L1)
            }
            Button::R1 => {
                ButtonsHigh::from_bits_truncate(self.buttons_high).contains(ButtonsHigh::R1)
            }
            Button::L2 => {
                ButtonsHigh::from_bits_truncate(self.buttons_high).contains(ButtonsHigh::L2)
            }
            Button::R2 => {
                ButtonsHigh::from_bits_truncate(self.buttons_high).contains(ButtonsHigh::R2)
            }
            Button::L3 => {
                ButtonsHigh::from_bits_truncate(self.buttons_high).contains(ButtonsHigh::L3)
            }
            Button::R3 => {
                ButtonsHigh::from_bits_truncate(self.buttons_high).contains(ButtonsHigh::R3)
            }

            // System/misc buttons
            Button::PS => {
                ButtonsMisc::from_bits_truncate(self.buttons_misc).contains(ButtonsMisc::PS)
            }
            Button::Touchpad => {
                ButtonsMisc::from_bits_truncate(self.buttons_misc).contains(ButtonsMisc::TOUCHPAD)
            }
            Button::Mute => {
                ButtonsMisc::from_bits_truncate(self.buttons_misc).contains(ButtonsMisc::MUTE)
            }
            Button::Create => {
                ButtonsHigh::from_bits_truncate(self.buttons_high).contains(ButtonsHigh::CREATE)
            }
            Button::Menu => {
                ButtonsHigh::from_bits_truncate(self.buttons_high).contains(ButtonsHigh::MENU)
            }
        }
    }

    // --- Analog Stick Axes ---
    /// Returns raw data for the (x, y) coordinate of the left stick
    /// Note: deadzones are not applied to these values
    pub fn left_stick(&self) -> (u8, u8) {
        (self.left_stick_x, self.left_stick_y)
    }
    /// Returns raw data for the (x, y) coordinate of the right stick
    /// Note: deadzones are not applied to these values
    pub fn right_stick(&self) -> (u8, u8) {
        (self.right_stick_x, self.right_stick_y)
    }

    // --- Trigger Axes (0-255) ---
    /// returns the variable position data for the L2 button (under the left middle finger)
    pub fn l2_axis(&self) -> u8 {
        self.l2_axis
    }
    /// returns the variable position data for the R2 button (under the right middle finger)
    pub fn r2_axis(&self) -> u8 {
        self.r2_axis
    }

    // --- D-Pad (Hat Switch) ---
    /// Returns: [DPadState]
    pub fn dpad(&self) -> DPadState {
        DPadState::from(self.buttons_low & 0x0F)
    }

    // --- System Buttons (buttons_misc) ---
    /// Returns true if the PlayStation button is being pressed
    pub fn ps(&self) -> bool {
        ButtonsMisc::from_bits_truncate(self.buttons_misc).contains(ButtonsMisc::PS)
    }
    /// Returns true if the touchpad button is being pressed
    pub fn touchpad_click(&self) -> bool {
        ButtonsMisc::from_bits_truncate(self.buttons_misc).contains(ButtonsMisc::TOUCHPAD)
    }
    /// Returns true if the mute button is being pressed
    pub fn mute(&self) -> bool {
        ButtonsMisc::from_bits_truncate(self.buttons_misc).contains(ButtonsMisc::MUTE)
    }

    /// Returns the [PowerState] of the battery
    pub fn battery_state(&self) -> PowerState {
        PowerState::from(self.power)
    }

    /// new.diff(&old).is_button_down() => button pressed
    /// old.diff(&new).is_button_down() => button released
    pub fn diff(&self, other: &DualSenseInput) -> DualSenseInput {
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

/// 8 Directions of the DPad or no direction at all
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DPadState {
    /// Up
    North = 0,
    /// Upper right
    NorthEast = 1,
    /// Right
    East = 2,
    /// Lower right
    SouthEast = 3,
    /// Down
    South = 4,
    /// Lower left
    SouthWest = 5,
    /// left
    West = 6,
    /// Upper left
    NorthWest = 7,
    /// No buttons pressed
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

/// Battery power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    /// Percentage (0-100)
    Discharging(u8),
    /// Percentage (0-100)
    Charging(u8),
    /// Battery is fully charged
    Full,
    /// Battery state is unknown
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

/// Touchpad data for a single finger x and y coordinate are both 12 bit values
#[repr(C, packed)]
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy, Default)]
pub struct TouchData {
    /// First bit is a flag wether the finger is touching the touchpad.
    /// The other bytes are a finger ID.
    /// Every time a finger touches the touchpad it gets an incremented ID.
    pub contact_id: u8, // Bit 7 is "up" (1) or "down" (0)
    /// lower 8 bits of the x coordinate
    pub x_low: u8,
    /// lower 4 bits of the y coordinate and higher 4 bits of the x coordinate
    pub y_low_x_high: u8, // Packed bits
    /// higher 8 bits of the y coordinate
    pub y_high: u8,
}

impl TouchData {
    /// Returns true if the finger is touching the touchpad
    pub fn is_active(&self) -> bool {
        (self.contact_id & 0x80) == 0
    }

    /// x coordinate of touching finger
    pub fn x(&self) -> u16 {
        ((self.y_low_x_high as u16 & 0x0F) << 8) | self.x_low as u16
    }
    /// y coordinate of touching finger
    pub fn y(&self) -> u16 {
        ((self.y_high as u16) << 4) | ((self.y_low_x_high as u16 & 0xF0) >> 4)
    }
}
