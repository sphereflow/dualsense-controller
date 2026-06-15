use crc32fast::Hasher;
use zerocopy::{FromZeros, Immutable, IntoBytes};

#[repr(C, packed)]
#[derive(IntoBytes, Immutable, FromZeros, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DualSenseOutput {
    /// 00 EnableRumbleEmulation1 UseRumbleNoHaptics1 AllowRightTriggerFFB1
    /// AllowLeftTriggerFFB1 AllowHeadphoneVolume1 AllowSpeakerVolume1 AllowMicVolume1
    /// AllowAudiocontrol(1)1
    pub flags_1: u8,

    /// 01 AllowMuteLight1 AllowAudioMute1 AllowLEDColor1 ResetLights1
    /// AllowPlayerIndicators1 AllowHapticLowPassFilter1 AllowMotorPowerLevel1 AllowAudioControl(2)1
    pub flags_2: u8,
    pub rumble_right: u8,      // 02
    pub rumble_left: u8,       // 03
    pub volume_headphones: u8, // 04
    pub volume_speaker: u8,    // 05
    pub volume_mic: u8,        // 06

    /// 07 MicSelect2 EchoCancelEnable1 NoiseCancelEnable1 OutputPathSelect2 InputPathSelect2
    pub audio_control_flags_1: u8,
    pub mute_light_mode: u8, // 08

    /// 09 PowerSave(Touch1 Motion1 Haptic1 Audio1)4 Mute(Mic1 Speaker1 Headphone1 Haptic1)4
    pub power_save_mute_control: u8,
    pub right_trigger_ffb: TriggerFFB, // 10-20
    pub left_trigger_ffb: TriggerFFB,  // 21-31
    pub host_time_stamp: u32,          // 32-35

    /// 36 TriggerMotorPowerReduction4 RumbleMotorPowerReduction4
    /// in 12.5 % steps both values only have range 0-7
    pub motor_power_level: u8,

    /// 37 SpeakerCompPreGain3 BeamformingEnable1 Unknown4
    pub audio_control_flags_2: u8,

    /// 38 AllowLightBrightnessChange1 AllowColorLightFadeAnimation1
    /// EnableImprovedRumbleEmulation1 Unused5
    pub flags_3: u8,

    /// 39 HapticLowPassFilter1 Unknown7
    pub haptic_low_pass_filter: u8,
    unknown: u8,                  // 40
    pub light_fade_animation: u8, // 41
    pub light_brightness: u8,     // 42
    pub player_light_flags: u8,   // 43
    pub lightbar_red: u8,         // 44
    pub lightbar_green: u8,       // 45
    pub lightbar_blue: u8,        // 46
}

impl DualSenseOutput {
    // --- Flags 1 (LSB first) ---
    pub fn enable_rumble_emulation(&self) -> bool {
        (self.flags_1 & 0x01) != 0
    }
    pub fn set_enable_rumble_emulation(&mut self, on: bool) {
        if on {
            self.flags_1 |= 0x01
        } else {
            self.flags_1 &= !0x01
        }
    }

    pub fn use_rumble_no_haptics(&self) -> bool {
        (self.flags_1 & 0x02) != 0
    }
    pub fn set_use_rumble_no_haptics(&mut self, on: bool) {
        if on {
            self.flags_1 |= 0x02
        } else {
            self.flags_1 &= !0x02
        }
    }

    pub fn allow_right_trigger_ffb(&self) -> bool {
        (self.flags_1 & 0x04) != 0
    }
    pub fn set_allow_right_trigger_ffb(&mut self, on: bool) {
        if on {
            self.flags_1 |= 0x04
        } else {
            self.flags_1 &= !0x04
        }
    }

    pub fn allow_left_trigger_ffb(&self) -> bool {
        (self.flags_1 & 0x08) != 0
    }
    pub fn set_allow_left_trigger_ffb(&mut self, on: bool) {
        if on {
            self.flags_1 |= 0x08
        } else {
            self.flags_1 &= !0x08
        }
    }

    pub fn allow_headphone_volume(&self) -> bool {
        (self.flags_1 & 0x10) != 0
    }
    pub fn set_allow_headphone_volume(&mut self, on: bool) {
        if on {
            self.flags_1 |= 0x10
        } else {
            self.flags_1 &= !0x10
        }
    }

    pub fn allow_speaker_volume(&self) -> bool {
        (self.flags_1 & 0x20) != 0
    }
    pub fn set_allow_speaker_volume(&mut self, on: bool) {
        if on {
            self.flags_1 |= 0x20
        } else {
            self.flags_1 &= !0x20
        }
    }

    pub fn allow_mic_volume(&self) -> bool {
        (self.flags_1 & 0x40) != 0
    }
    pub fn set_allow_mic_volume(&mut self, on: bool) {
        if on {
            self.flags_1 |= 0x40
        } else {
            self.flags_1 &= !0x40
        }
    }

    pub fn allow_audio_control_1(&self) -> bool {
        (self.flags_1 & 0x80) != 0
    }
    pub fn set_allow_audio_control_1(&mut self, on: bool) {
        if on {
            self.flags_1 |= 0x80
        } else {
            self.flags_1 &= !0x80
        }
    }

    // --- Flags 2 (LSB first) ---
    pub fn allow_mute_light(&self) -> bool {
        (self.flags_2 & 0x01) != 0
    }
    pub fn set_allow_mute_light(&mut self, on: bool) {
        if on {
            self.flags_2 |= 0x01
        } else {
            self.flags_2 &= !0x01
        }
    }

    pub fn allow_audio_mute(&self) -> bool {
        (self.flags_2 & 0x02) != 0
    }
    pub fn set_allow_audio_mute(&mut self, on: bool) {
        if on {
            self.flags_2 |= 0x02
        } else {
            self.flags_2 &= !0x02
        }
    }

    pub fn allow_led_color(&self) -> bool {
        (self.flags_2 & 0x04) != 0
    }
    pub fn set_allow_led_color(&mut self, on: bool) {
        if on {
            self.flags_2 |= 0x04
        } else {
            self.flags_2 &= !0x04
        }
    }

    pub fn reset_lights(&self) -> bool {
        (self.flags_2 & 0x08) != 0
    }
    pub fn set_reset_lights(&mut self, on: bool) {
        if on {
            self.flags_2 |= 0x08
        } else {
            self.flags_2 &= !0x08
        }
    }

    pub fn allow_player_indicators(&self) -> bool {
        (self.flags_2 & 0x10) != 0
    }
    pub fn set_allow_player_indicators(&mut self, on: bool) {
        if on {
            self.flags_2 |= 0x10
        } else {
            self.flags_2 &= !0x10
        }
    }

    pub fn allow_haptic_low_pass_filter_flag(&self) -> bool {
        (self.flags_2 & 0x20) != 0
    }
    pub fn set_allow_haptic_low_pass_filter_flag(&mut self, on: bool) {
        if on {
            self.flags_2 |= 0x20
        } else {
            self.flags_2 &= !0x20
        }
    }

    pub fn allow_motor_power_level_flag(&self) -> bool {
        (self.flags_2 & 0x40) != 0
    }
    pub fn set_allow_motor_power_level_flag(&mut self, on: bool) {
        if on {
            self.flags_2 |= 0x40
        } else {
            self.flags_2 &= !0x40
        }
    }

    pub fn allow_audio_control_2(&self) -> bool {
        (self.flags_2 & 0x80) != 0
    }
    pub fn set_allow_audio_control_2(&mut self, on: bool) {
        if on {
            self.flags_2 |= 0x80
        } else {
            self.flags_2 &= !0x80
        }
    }

    // --- Rumble / Volumes ---
    pub fn rumble_right(&self) -> u8 {
        self.rumble_right
    }
    pub fn set_rumble_right(&mut self, v: u8) {
        self.rumble_right = v
    }

    pub fn rumble_left(&self) -> u8 {
        self.rumble_left
    }
    pub fn set_rumble_left(&mut self, v: u8) {
        self.rumble_left = v
    }

    pub fn volume_headphones(&self) -> u8 {
        self.volume_headphones
    }
    pub fn set_volume_headphones(&mut self, v: u8) {
        self.volume_headphones = v
    }

    pub fn volume_speaker(&self) -> u8 {
        self.volume_speaker
    }
    pub fn set_volume_speaker(&mut self, v: u8) {
        self.volume_speaker = v
    }

    pub fn volume_mic(&self) -> u8 {
        self.volume_mic
    }
    pub fn set_volume_mic(&mut self, v: u8) {
        self.volume_mic = v
    }

    // --- Audio control flags 1 (bitfields, LSB first) ---
    /// Mic select: bits 0-1
    pub fn mic_select(&self) -> u8 {
        self.audio_control_flags_1 & 0x03
    }
    pub fn set_mic_select(&mut self, sel: u8) {
        self.audio_control_flags_1 = (self.audio_control_flags_1 & !0x03) | (sel & 0x03)
    }

    /// Echo cancel enable: bit 2
    pub fn echo_cancel_enable(&self) -> bool {
        (self.audio_control_flags_1 & 0x04) != 0
    }
    pub fn set_echo_cancel_enable(&mut self, on: bool) {
        if on {
            self.audio_control_flags_1 |= 0x04
        } else {
            self.audio_control_flags_1 &= !0x04
        }
    }

    /// Noise cancel enable: bit 3
    pub fn noise_cancel_enable(&self) -> bool {
        (self.audio_control_flags_1 & 0x08) != 0
    }
    pub fn set_noise_cancel_enable(&mut self, on: bool) {
        if on {
            self.audio_control_flags_1 |= 0x08
        } else {
            self.audio_control_flags_1 &= !0x08
        }
    }

    /// Output path select: bits 4-5
    pub fn output_path_select(&self) -> u8 {
        (self.audio_control_flags_1 >> 4) & 0x03
    }
    pub fn set_output_path_select(&mut self, sel: u8) {
        self.audio_control_flags_1 =
            (self.audio_control_flags_1 & !(0x03 << 4)) | ((sel & 0x03) << 4)
    }

    /// Input path select: bits 6-7
    pub fn input_path_select(&self) -> u8 {
        (self.audio_control_flags_1 >> 6) & 0x03
    }
    pub fn set_input_path_select(&mut self, sel: u8) {
        self.audio_control_flags_1 =
            (self.audio_control_flags_1 & !(0x03 << 6)) | ((sel & 0x03) << 6)
    }

    pub fn mute_light_mode(&self) -> u8 {
        self.mute_light_mode
    }
    pub fn set_mute_light_mode(&mut self, v: u8) {
        self.mute_light_mode = v
    }

    // --- Power save / Mute control (LSB first) ---
    pub fn power_save_touch(&self) -> bool {
        (self.power_save_mute_control & 0x01) != 0
    }
    pub fn set_power_save_touch(&mut self, on: bool) {
        if on {
            self.power_save_mute_control |= 0x01
        } else {
            self.power_save_mute_control &= !0x01
        }
    }

    pub fn power_save_motion(&self) -> bool {
        (self.power_save_mute_control & 0x02) != 0
    }
    pub fn set_power_save_motion(&mut self, on: bool) {
        if on {
            self.power_save_mute_control |= 0x02
        } else {
            self.power_save_mute_control &= !0x02
        }
    }

    pub fn power_save_haptic(&self) -> bool {
        (self.power_save_mute_control & 0x04) != 0
    }
    pub fn set_power_save_haptic(&mut self, on: bool) {
        if on {
            self.power_save_mute_control |= 0x04
        } else {
            self.power_save_mute_control &= !0x04
        }
    }

    pub fn power_save_audio(&self) -> bool {
        (self.power_save_mute_control & 0x08) != 0
    }
    pub fn set_power_save_audio(&mut self, on: bool) {
        if on {
            self.power_save_mute_control |= 0x08
        } else {
            self.power_save_mute_control &= !0x08
        }
    }

    pub fn mute_mic(&self) -> bool {
        (self.power_save_mute_control & 0x10) != 0
    }
    pub fn set_mute_mic(&mut self, on: bool) {
        if on {
            self.power_save_mute_control |= 0x10
        } else {
            self.power_save_mute_control &= !0x10
        }
    }

    pub fn mute_speaker(&self) -> bool {
        (self.power_save_mute_control & 0x20) != 0
    }
    pub fn set_mute_speaker(&mut self, on: bool) {
        if on {
            self.power_save_mute_control |= 0x20
        } else {
            self.power_save_mute_control &= !0x20
        }
    }

    pub fn mute_headphone(&self) -> bool {
        (self.power_save_mute_control & 0x40) != 0
    }
    pub fn set_mute_headphone(&mut self, on: bool) {
        if on {
            self.power_save_mute_control |= 0x40
        } else {
            self.power_save_mute_control &= !0x40
        }
    }

    pub fn mute_haptic(&self) -> bool {
        (self.power_save_mute_control & 0x80) != 0
    }
    pub fn set_mute_haptic(&mut self, on: bool) {
        if on {
            self.power_save_mute_control |= 0x80
        } else {
            self.power_save_mute_control &= !0x80
        }
    }

    // --- Audio control flags 2 ---
    /// Speaker compensation pre-gain: bits 0-2
    pub fn speaker_comp_pregain(&self) -> u8 {
        self.audio_control_flags_2 & 0x07
    }
    pub fn set_speaker_comp_pregain(&mut self, v: u8) {
        self.audio_control_flags_2 = (self.audio_control_flags_2 & !0x07) | (v & 0x07)
    }

    /// Beamforming enable: bit 3
    pub fn beamforming_enable(&self) -> bool {
        (self.audio_control_flags_2 & 0x08) != 0
    }
    pub fn set_beamforming_enable(&mut self, on: bool) {
        if on {
            self.audio_control_flags_2 |= 0x08
        } else {
            self.audio_control_flags_2 &= !0x08
        }
    }

    /// Upper 4 bits are currently unknown
    pub fn audio_control_flags_2_unknown(&self) -> u8 {
        self.audio_control_flags_2 >> 4
    }
    pub fn set_audio_control_flags_2_unknown(&mut self, v: u8) {
        self.audio_control_flags_2 = (self.audio_control_flags_2 & 0x0F) | ((v & 0x0F) << 4)
    }

    // --- Flags 3 ---
    pub fn allow_light_brightness_change(&self) -> bool {
        (self.flags_3 & 0x01) != 0
    }
    pub fn set_allow_light_brightness_change(&mut self, on: bool) {
        if on {
            self.flags_3 |= 0x01
        } else {
            self.flags_3 &= !0x01
        }
    }

    pub fn allow_color_light_fade_animation(&self) -> bool {
        (self.flags_3 & 0x02) != 0
    }
    pub fn set_allow_color_light_fade_animation(&mut self, on: bool) {
        if on {
            self.flags_3 |= 0x02
        } else {
            self.flags_3 &= !0x02
        }
    }

    pub fn enable_improved_rumble_emulation(&self) -> bool {
        (self.flags_3 & 0x04) != 0
    }
    pub fn set_enable_improved_rumble_emulation(&mut self, on: bool) {
        if on {
            self.flags_3 |= 0x04
        } else {
            self.flags_3 &= !0x04
        }
    }

    // --- Haptic low pass filter ---
    pub fn haptic_low_pass_filter_enabled(&self) -> bool {
        (self.haptic_low_pass_filter & 0x01) != 0
    }
    pub fn set_haptic_low_pass_filter_enabled(&mut self, on: bool) {
        if on {
            self.haptic_low_pass_filter |= 0x01
        } else {
            self.haptic_low_pass_filter &= !0x01
        }
    }
    pub fn haptic_low_pass_filter_unknown(&self) -> u8 {
        self.haptic_low_pass_filter >> 1
    }
    pub fn set_haptic_low_pass_filter_unknown(&mut self, v: u8) {
        self.haptic_low_pass_filter = (self.haptic_low_pass_filter & 0x01) | ((v & 0x7F) << 1)
    }

    // --- Light and player fields ---
    pub fn light_fade_animation(&self) -> u8 {
        self.light_fade_animation
    }
    pub fn set_light_fade_animation(&mut self, v: u8) {
        self.light_fade_animation = v
    }

    pub fn light_brightness(&self) -> u8 {
        self.light_brightness
    }
    pub fn set_light_brightness(&mut self, v: u8) {
        self.light_brightness = v
    }

    pub fn player_light_flags(&self) -> u8 {
        self.player_light_flags
    }
    pub fn set_player_light_flags(&mut self, v: u8) {
        self.player_light_flags = v
    }

    pub fn lightbar_red(&self) -> u8 {
        self.lightbar_red
    }
    pub fn set_lightbar_red(&mut self, v: u8) {
        self.lightbar_red = v
    }

    pub fn lightbar_green(&self) -> u8 {
        self.lightbar_green
    }
    pub fn set_lightbar_green(&mut self, v: u8) {
        self.lightbar_green = v
    }

    pub fn lightbar_blue(&self) -> u8 {
        self.lightbar_blue
    }
    pub fn set_lightbar_blue(&mut self, v: u8) {
        self.lightbar_blue = v
    }
}

// Bit masks and default values for DualSenseOutput
pub const FLAGS1_ENABLE_RUMBLE_EMULATION: u8 = 0x01;
pub const FLAGS1_USE_RUMBLE_NO_HAPTICS: u8 = 0x02;
pub const FLAGS1_ALLOW_RIGHT_TRIGGER_FFB: u8 = 0x04;
pub const FLAGS1_ALLOW_LEFT_TRIGGER_FFB: u8 = 0x08;
pub const FLAGS1_ALLOW_HEADPHONE_VOLUME: u8 = 0x10;
pub const FLAGS1_ALLOW_SPEAKER_VOLUME: u8 = 0x20;
pub const FLAGS1_ALLOW_MIC_VOLUME: u8 = 0x40;
pub const FLAGS1_ALLOW_AUDIO_CONTROL_1: u8 = 0x80;

pub const FLAGS2_ALLOW_MUTE_LIGHT: u8 = 0x01;
pub const FLAGS2_ALLOW_AUDIO_MUTE: u8 = 0x02;
pub const FLAGS2_ALLOW_LED_COLOR: u8 = 0x04;
pub const FLAGS2_RESET_LIGHTS: u8 = 0x08;
pub const FLAGS2_ALLOW_PLAYER_INDICATORS: u8 = 0x10;
pub const FLAGS2_ALLOW_HAPTIC_LOW_PASS: u8 = 0x20;
pub const FLAGS2_ALLOW_MOTOR_POWER_LEVEL: u8 = 0x40;
pub const FLAGS2_ALLOW_AUDIO_CONTROL_2: u8 = 0x80;

pub const FLAGS3_ALLOW_LIGHT_BRIGHTNESS_CHANGE: u8 = 0x01;
pub const FLAGS3_ALLOW_COLOR_LIGHT_FADE: u8 = 0x02;
pub const FLAGS3_ENABLE_IMPROVED_RUMBLE: u8 = 0x04;

// Default bit patterns chosen explicitly for readability. Previous code used magic bytes;
// these constants make intent clear while preserving behaviour.
const DEFAULT_FLAGS_1: u8 = FLAGS1_ENABLE_RUMBLE_EMULATION
    | FLAGS1_USE_RUMBLE_NO_HAPTICS
    | FLAGS1_ALLOW_RIGHT_TRIGGER_FFB
    | FLAGS1_ALLOW_LEFT_TRIGGER_FFB
    | FLAGS1_ALLOW_HEADPHONE_VOLUME
    | FLAGS1_ALLOW_SPEAKER_VOLUME
    | FLAGS1_ALLOW_MIC_VOLUME
    | FLAGS1_ALLOW_AUDIO_CONTROL_1;

const DEFAULT_FLAGS_2: u8 = FLAGS2_ALLOW_MUTE_LIGHT
    | FLAGS2_ALLOW_AUDIO_MUTE
    | FLAGS2_ALLOW_LED_COLOR
    | FLAGS2_RESET_LIGHTS
    | FLAGS2_ALLOW_PLAYER_INDICATORS
    | FLAGS2_ALLOW_HAPTIC_LOW_PASS
    | FLAGS2_ALLOW_MOTOR_POWER_LEVEL
    | FLAGS2_ALLOW_AUDIO_CONTROL_2;

const DEFAULT_FLAGS_3: u8 = FLAGS3_ALLOW_LIGHT_BRIGHTNESS_CHANGE
    | FLAGS3_ALLOW_COLOR_LIGHT_FADE
    | FLAGS3_ENABLE_IMPROVED_RUMBLE;

impl DualSenseOutput {
    /// Construct a validated default DualSenseOutput. Use when callers want explicit defaults
    /// and clamped motor power values.
    pub fn new() -> Self {
        let mut s = Self {
            flags_1: DEFAULT_FLAGS_1,
            flags_2: DEFAULT_FLAGS_2,
            rumble_right: 0,
            rumble_left: 0,
            volume_headphones: 0,
            volume_speaker: 0,
            volume_mic: 0,
            audio_control_flags_1: 0b00001100, // Echo/Noise cancel defaults
            mute_light_mode: 0,
            power_save_mute_control: 0,
            right_trigger_ffb: TriggerFFB::off(),
            left_trigger_ffb: TriggerFFB::off(),
            host_time_stamp: Default::default(),
            motor_power_level: 0,
            audio_control_flags_2: Default::default(),
            flags_3: DEFAULT_FLAGS_3,
            haptic_low_pass_filter: Default::default(),
            unknown: Default::default(),
            light_fade_animation: Default::default(),
            light_brightness: Default::default(),
            player_light_flags: Default::default(),
            lightbar_red: Default::default(),
            lightbar_green: Default::default(),
            lightbar_blue: Default::default(),
        };

        // Initialize motor power with clamping helpers (0..=7 per nibble)
        s.set_trigger_motor_power_reduction(0);
        s.set_rumble_motor_power_reduction(0);

        s
    }

    /// Set trigger motor power reduction (0..=7). Value is clamped.
    pub fn set_trigger_motor_power_reduction(&mut self, v: u8) {
        let t = v.min(7) & 0x0F;
        self.motor_power_level = (self.motor_power_level & 0xF0) | t;
    }

    /// Get trigger motor power reduction (0..=7)
    pub fn get_trigger_motor_power_reduction(&self) -> u8 {
        self.motor_power_level & 0x0F
    }

    /// Set rumble motor power reduction (0..=7). Value is clamped.
    pub fn set_rumble_motor_power_reduction(&mut self, v: u8) {
        let r = (v.min(7) & 0x0F) << 4;
        self.motor_power_level = (self.motor_power_level & 0x0F) | r;
    }

    /// Get rumble motor power reduction (0..=7)
    pub fn get_rumble_motor_power_reduction(&self) -> u8 {
        (self.motor_power_level >> 4) & 0x0F
    }
}

impl Default for DualSenseOutput {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C, packed)]
#[derive(IntoBytes, FromZeros, Immutable, Debug, Clone, Copy)]
pub struct DualSenseOutputReportUSB {
    report_id: u8, // 0x02
    pub base: DualSenseOutput,
}

#[repr(C, packed)]
#[derive(IntoBytes, FromZeros, Immutable, Debug, Clone, Copy)]
pub struct DualSenseOutputReportBT {
    pub report_id: u8,            // 0x31
    pub seq_number_and_flags: u8, // Unknown1 EnableHID1 Unknown2 SequenceNumber4
    pub tag: u8,
    pub base: DualSenseOutput,
    pub reserved: [u8; 24],
    pub crc32: u32,
}

impl DualSenseOutputReportBT {
    pub fn add_crc(&mut self) {
        // DualSense Bluetooth hardware requires the 0xA2 seed injected before the payload
        const PS_OUTPUT_CRC32_SEED: u8 = 0xA2;

        let mut hasher = Hasher::new();

        // 1. Feed the mandatory Bluetooth initialization seed
        hasher.update(&[PS_OUTPUT_CRC32_SEED]);

        // 2. Feed the payload bytes (everything except the final 4-byte CRC field)
        // Compute payload length programmatically to avoid off-by-one errors.
        let total_len = std::mem::size_of::<Self>();
        let crc_len = std::mem::size_of::<u32>();
        let payload_len = total_len - crc_len;
        hasher.update(&self.as_bytes()[..payload_len]);

        self.crc32 = hasher.finalize();
    }
}

#[repr(C, packed)]
#[derive(IntoBytes, FromZeros, Immutable, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TriggerFFB {
    pub mode: u8, // 0: Off, 1: Feedback, 2: Weapon, 6: Vibration
    pub parameters: [u8; 10],
}

impl TriggerFFB {
    pub fn off() -> Self {
        Self::new_zeroed()
    }

    pub fn feedback(position: u8, strength: u8) -> Self {
        let mut effect = Self::new_zeroed();
        effect.mode = 0x01;
        effect.parameters[0] = position; // Start position (0-255)
        effect.parameters[1] = strength; // Resistance strength
        effect
    }

    pub fn weapon(start: u8, end: u8, strength: u8) -> Self {
        let mut effect = Self::new_zeroed();
        effect.mode = 0x02;
        effect.parameters[0] = start;
        effect.parameters[1] = end;
        effect.parameters[2] = strength;
        effect
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn size_is_47_bytes() {
        assert_eq!(size_of::<DualSenseOutput>(), 47);
    }

    #[test]
    fn default_values_and_clamping() {
        let d = DualSenseOutput::default();
        // Trigger FFB should be off
        assert_eq!(d.right_trigger_ffb.mode, 0);
        assert_eq!(d.left_trigger_ffb.mode, 0);
        // Motor power should be zeroed and within 0..=7
        assert_eq!(d.get_trigger_motor_power_reduction(), 0);
        assert_eq!(d.get_rumble_motor_power_reduction(), 0);
        // Flags equal defaults
        assert_eq!(d.flags_1, DEFAULT_FLAGS_1);
        assert_eq!(d.flags_2, DEFAULT_FLAGS_2);
        assert_eq!(d.flags_3, DEFAULT_FLAGS_3);
    }

    #[test]
    fn motor_power_clamping() {
        let mut d = DualSenseOutput::default();
        d.set_trigger_motor_power_reduction(255);
        d.set_rumble_motor_power_reduction(255);
        assert_eq!(d.get_trigger_motor_power_reduction(), 7);
        assert_eq!(d.get_rumble_motor_power_reduction(), 7);
    }
}
