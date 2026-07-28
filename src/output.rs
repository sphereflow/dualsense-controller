use bitflags::bitflags;
use crc32fast::Hasher;
use zerocopy::{FromZeros, Immutable, IntoBytes};

/// Main struct for writing output to the controller.
///
/// Controls rumble motors, the RGB lightbar, audio volumes, trigger force feedback,
/// player indicator LEDs, and power save flags.
#[repr(C, packed)]
#[derive(IntoBytes, Immutable, FromZeros, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DualSenseOutput {
    // 00 EnableRumbleEmulation1 UseRumbleNoHaptics1 AllowRightTriggerFFB1
    // AllowLeftTriggerFFB1 AllowHeadphoneVolume1 AllowSpeakerVolume1 AllowMicVolume1
    // AllowAudiocontrol(1)1
    pub(crate) flags_1: u8,

    // 01 AllowMuteLight1 AllowAudioMute1 AllowLEDColor1 ResetLights1
    // AllowPlayerIndicators1 AllowHapticLowPassFilter1 AllowMotorPowerLevel1 AllowAudioControl(2)1
    pub(crate) flags_2: u8,
    /// strength of the right side rumble motor
    pub rumble_right: u8, // 02
    /// strength of the left side rumble motor
    pub rumble_left: u8, // 03
    /// Headphone volume
    pub volume_headphones: u8, // 04
    /// Speaker volume
    pub volume_speaker: u8, // 05
    /// Microphone volume
    pub volume_mic: u8, // 06

    // 07 MicSelect2 EchoCancelEnable1 NoiseCancelEnable1 OutputPathSelect2 InputPathSelect2
    pub(crate) audio_control_flags_1: u8,
    pub(crate) mute_light_mode: u8, // 08

    // 09 PowerSave(Touch1 Motion1 Haptic1 Audio1)4 Mute(Mic1 Speaker1 Headphone1 Haptic1)4
    pub(crate) power_save_mute_control: u8,
    /// Force feedback for the right trigger
    pub right_trigger_ffb: TriggerFFB, // 10-20
    /// Force feedback for the left trigger
    pub left_trigger_ffb: TriggerFFB, // 21-31
    /// Host timestamp value
    pub host_time_stamp: u32, // 32-35

    /// 36 TriggerMotorPowerReduction4 RumbleMotorPowerReduction4
    /// In 12.5% steps; both values only have range 0-7
    pub motor_power_level: u8,

    /// 37 SpeakerCompPreGain3 BeamformingEnable1 Unknown4
    pub audio_control_flags_2: u8,

    /// 38 AllowLightBrightnessChange1 AllowColorLightFadeAnimation1
    /// EnableImprovedRumbleEmulation1 Unused5
    pub flags_3: u8,

    /// 39 HapticLowPassFilter1 Unknown7
    pub haptic_low_pass_filter: u8,
    unknown: u8, // 40
    /// Value to controll light animations
    pub light_fade_animation: u8, // 41
    /// Brightness of the RGB LED
    pub light_brightness: u8, // 42
    /// Value to controll the lit areas on the light strip
    pub player_light_flags: u8, // 43
    /// RGB LED red component
    pub lightbar_red: u8, // 44
    /// RGB LED green component
    pub lightbar_green: u8, // 45
    /// RGB LED blue component
    pub lightbar_blue: u8, // 46
}

bitflags! {
    pub(crate) struct Flags1: u8 {
        const ENABLE_RUMBLE_EMULATION    = 0x01;
        const USE_RUMBLE_NO_HAPTICS      = 0x02;
        const ALLOW_RIGHT_TRIGGER_FFB    = 0x04;
        const ALLOW_LEFT_TRIGGER_FFB     = 0x08;
        const ALLOW_HEADPHONE_VOLUME     = 0x10;
        const ALLOW_SPEAKER_VOLUME       = 0x20;
        const ALLOW_MIC_VOLUME           = 0x40;
        const ALLOW_AUDIO_CONTROL_1      = 0x80;
    }
}

bitflags! {
    pub(crate) struct Flags2: u8 {
        const ALLOW_MUTE_LIGHT           = 0x01;
        const ALLOW_AUDIO_MUTE           = 0x02;
        const ALLOW_LED_COLOR            = 0x04;
        const RESET_LIGHTS               = 0x08;
        const ALLOW_PLAYER_INDICATORS    = 0x10;
        const ALLOW_HAPTIC_LOW_PASS      = 0x20;
        const ALLOW_MOTOR_POWER_LEVEL    = 0x40;
        const ALLOW_AUDIO_CONTROL_2      = 0x80;
    }
}

bitflags! {
    /// Flags corresponding to `flags_3` field in DualSenseOutput
    pub(crate) struct Flags3: u8 {
        const ALLOW_LIGHT_BRIGHTNESS_CHANGE = 0x01;
        const ALLOW_COLOR_LIGHT_FADE        = 0x02;
        const ENABLE_IMPROVED_RUMBLE        = 0x04;
    }
}

bitflags! {
    /// Flags corresponding to `power_save_mute_control` field
    pub(crate) struct PowerSaveMute: u8 {
        const POWER_SAVE_TOUCH  = 0x01;
        const POWER_SAVE_MOTION = 0x02;
        const POWER_SAVE_HAPTIC = 0x04;
        const POWER_SAVE_AUDIO  = 0x08;
        const MUTE_MIC          = 0x10;
        const MUTE_SPEAKER      = 0x20;
        const MUTE_HEADPHONE    = 0x40;
        const MUTE_HAPTIC       = 0x80;
    }
}

impl DualSenseOutput {
    // --- Flags 1 (LSB first) ---
    /// Getter for [Flags1]
    pub(crate) fn flags1(&self) -> Flags1 {
        Flags1::from_bits_truncate(self.flags_1)
    }
    /// Setter for [Flags1]
    pub(crate) fn set_flags1(&mut self, f: Flags1) {
        self.flags_1 = f.bits();
    }

    /// Returns true if rumble emulation is enabled.
    pub fn enable_rumble_emulation(&self) -> bool {
        self.flags1().contains(Flags1::ENABLE_RUMBLE_EMULATION)
    }
    /// Enables or disables rumble emulation.
    pub fn set_enable_rumble_emulation(&mut self, on: bool) {
        let mut f = self.flags1();
        f.set(Flags1::ENABLE_RUMBLE_EMULATION, on);
        self.set_flags1(f);
    }

    /// Returns true if rumble motors are used instead of haptics.
    pub fn use_rumble_no_haptics(&self) -> bool {
        self.flags1().contains(Flags1::USE_RUMBLE_NO_HAPTICS)
    }
    /// Sets whether rumble motors are used instead of haptic actuators.
    pub fn set_use_rumble_no_haptics(&mut self, on: bool) {
        let mut f = self.flags1();
        f.set(Flags1::USE_RUMBLE_NO_HAPTICS, on);
        self.set_flags1(f);
    }

    /// Returns true if force feedback is allowed for the right trigger.
    pub fn allow_right_trigger_ffb(&self) -> bool {
        self.flags1().contains(Flags1::ALLOW_RIGHT_TRIGGER_FFB)
    }
    /// Enables or disables force feedback updates for the right trigger.
    pub fn set_allow_right_trigger_ffb(&mut self, on: bool) {
        let mut f = self.flags1();
        f.set(Flags1::ALLOW_RIGHT_TRIGGER_FFB, on);
        self.set_flags1(f);
    }

    /// Returns true if force feedback is allowed for the left trigger.
    pub fn allow_left_trigger_ffb(&self) -> bool {
        self.flags1().contains(Flags1::ALLOW_LEFT_TRIGGER_FFB)
    }
    /// Enables or disables force feedback updates for the left trigger.
    pub fn set_allow_left_trigger_ffb(&mut self, on: bool) {
        let mut f = self.flags1();
        f.set(Flags1::ALLOW_LEFT_TRIGGER_FFB, on);
        self.set_flags1(f);
    }

    /// Returns true if headphone volume updates are allowed.
    pub fn allow_headphone_volume(&self) -> bool {
        self.flags1().contains(Flags1::ALLOW_HEADPHONE_VOLUME)
    }
    /// Enables or disables headphone volume updates.
    pub fn set_allow_headphone_volume(&mut self, on: bool) {
        let mut f = self.flags1();
        f.set(Flags1::ALLOW_HEADPHONE_VOLUME, on);
        self.set_flags1(f);
    }

    /// Returns true if built-in speaker volume updates are allowed.
    pub fn allow_speaker_volume(&self) -> bool {
        self.flags1().contains(Flags1::ALLOW_SPEAKER_VOLUME)
    }
    /// Enables or disables built-in speaker volume updates.
    pub fn set_allow_speaker_volume(&mut self, on: bool) {
        let mut f = self.flags1();
        f.set(Flags1::ALLOW_SPEAKER_VOLUME, on);
        self.set_flags1(f);
    }

    /// Returns true if microphone volume updates are allowed.
    pub fn allow_mic_volume(&self) -> bool {
        self.flags1().contains(Flags1::ALLOW_MIC_VOLUME)
    }
    /// Enables or disables microphone volume updates.
    pub fn set_allow_mic_volume(&mut self, on: bool) {
        let mut f = self.flags1();
        f.set(Flags1::ALLOW_MIC_VOLUME, on);
        self.set_flags1(f);
    }

    /// Returns true if audio control register group 1 is allowed.
    pub fn allow_audio_control_1(&self) -> bool {
        self.flags1().contains(Flags1::ALLOW_AUDIO_CONTROL_1)
    }
    /// Enables or disables audio control register group 1 updates.
    pub fn set_allow_audio_control_1(&mut self, on: bool) {
        let mut f = self.flags1();
        f.set(Flags1::ALLOW_AUDIO_CONTROL_1, on);
        self.set_flags1(f);
    }

    // --- Flags 2 (LSB first) ---
    /// Getter for [Flags2].
    fn flags2(&self) -> Flags2 {
        Flags2::from_bits_truncate(self.flags_2)
    }
    /// Setter for [Flags2].
    fn set_flags2(&mut self, f: Flags2) {
        self.flags_2 = f.bits();
    }

    /// Returns true if mute button LED updates are allowed.
    pub fn allow_mute_light(&self) -> bool {
        self.flags2().contains(Flags2::ALLOW_MUTE_LIGHT)
    }
    /// Enables or disables mute button LED updates.
    pub fn set_allow_mute_light(&mut self, on: bool) {
        let mut f = self.flags2();
        f.set(Flags2::ALLOW_MUTE_LIGHT, on);
        self.set_flags2(f);
    }

    /// Returns true if audio mute flag updates are allowed.
    pub fn allow_audio_mute(&self) -> bool {
        self.flags2().contains(Flags2::ALLOW_AUDIO_MUTE)
    }
    /// Enables or disables audio mute flag updates.
    pub fn set_allow_audio_mute(&mut self, on: bool) {
        let mut f = self.flags2();
        f.set(Flags2::ALLOW_AUDIO_MUTE, on);
        self.set_flags2(f);
    }

    /// Returns true if RGB LED lightbar updates are allowed.
    pub fn allow_led_color(&self) -> bool {
        self.flags2().contains(Flags2::ALLOW_LED_COLOR)
    }
    /// Enables or disables RGB LED lightbar updates.
    pub fn set_allow_led_color(&mut self, on: bool) {
        let mut f = self.flags2();
        f.set(Flags2::ALLOW_LED_COLOR, on);
        self.set_flags2(f);
    }

    /// Returns true if reset lights flag is set.
    pub fn reset_lights(&self) -> bool {
        self.flags2().contains(Flags2::RESET_LIGHTS)
    }
    /// Enables or disables light reset.
    pub fn set_reset_lights(&mut self, on: bool) {
        let mut f = self.flags2();
        f.set(Flags2::RESET_LIGHTS, on);
        self.set_flags2(f);
    }

    /// Returns true if player indicator LED updates are allowed.
    pub fn allow_player_indicators(&self) -> bool {
        self.flags2().contains(Flags2::ALLOW_PLAYER_INDICATORS)
    }
    /// Enables or disables player indicator LED updates.
    pub fn set_allow_player_indicators(&mut self, on: bool) {
        let mut f = self.flags2();
        f.set(Flags2::ALLOW_PLAYER_INDICATORS, on);
        self.set_flags2(f);
    }

    /// Returns true if haptic low pass filter flag updates are allowed.
    pub fn allow_haptic_low_pass_filter_flag(&self) -> bool {
        self.flags2().contains(Flags2::ALLOW_HAPTIC_LOW_PASS)
    }
    /// Enables or disables haptic low pass filter flag updates.
    pub fn set_allow_haptic_low_pass_filter_flag(&mut self, on: bool) {
        let mut f = self.flags2();
        f.set(Flags2::ALLOW_HAPTIC_LOW_PASS, on);
        self.set_flags2(f);
    }

    /// Returns true if motor power level flag updates are allowed.
    pub fn allow_motor_power_level_flag(&self) -> bool {
        self.flags2().contains(Flags2::ALLOW_MOTOR_POWER_LEVEL)
    }
    /// Enables or disables motor power level flag updates.
    pub fn set_allow_motor_power_level_flag(&mut self, on: bool) {
        let mut f = self.flags2();
        f.set(Flags2::ALLOW_MOTOR_POWER_LEVEL, on);
        self.set_flags2(f);
    }

    /// Returns true if audio control register group 2 is allowed.
    pub fn allow_audio_control_2(&self) -> bool {
        self.flags2().contains(Flags2::ALLOW_AUDIO_CONTROL_2)
    }
    /// Enables or disables audio control register group 2 updates.
    pub fn set_allow_audio_control_2(&mut self, on: bool) {
        let mut f = self.flags2();
        f.set(Flags2::ALLOW_AUDIO_CONTROL_2, on);
        self.set_flags2(f);
    }

    // --- Rumble / Volumes ---
    /// Returns the intensity of the right rumble motor (`0..=255`).
    pub fn rumble_right(&self) -> u8 {
        self.rumble_right
    }
    /// Sets the intensity of the right rumble motor (`0..=255`).
    pub fn set_rumble_right(&mut self, v: u8) {
        self.rumble_right = v;
    }

    /// Returns the intensity of the left rumble motor (`0..=255`).
    pub fn rumble_left(&self) -> u8 {
        self.rumble_left
    }
    /// Sets the intensity of the left rumble motor (`0..=255`).
    pub fn set_rumble_left(&mut self, v: u8) {
        self.rumble_left = v;
    }

    /// Returns the headphone volume output level (`0..=255`).
    pub fn volume_headphones(&self) -> u8 {
        self.volume_headphones
    }
    /// Sets the headphone volume output level (`0..=255`).
    pub fn set_volume_headphones(&mut self, v: u8) {
        self.volume_headphones = v;
    }

    /// Returns the built-in speaker volume output level (`0..=255`).
    pub fn volume_speaker(&self) -> u8 {
        self.volume_speaker
    }
    /// Sets the built-in speaker volume output level (`0..=255`).
    pub fn set_volume_speaker(&mut self, v: u8) {
        self.volume_speaker = v;
    }

    /// Returns the microphone input gain level (`0..=255`).
    pub fn volume_mic(&self) -> u8 {
        self.volume_mic
    }
    /// Sets the microphone input gain level (`0..=255`).
    pub fn set_volume_mic(&mut self, v: u8) {
        self.volume_mic = v;
    }

    // --- Audio control flags 1 (bitfields, LSB first) ---
    /// Returns the microphone select setting (bits 0-1).
    pub fn mic_select(&self) -> u8 {
        self.audio_control_flags_1 & 0x03
    }
    /// Sets the microphone select setting (bits 0-1).
    pub fn set_mic_select(&mut self, sel: u8) {
        self.audio_control_flags_1 = (self.audio_control_flags_1 & !0x03) | (sel & 0x03);
    }

    /// Returns true if echo cancellation is enabled.
    pub fn echo_cancel_enable(&self) -> bool {
        (self.audio_control_flags_1 & 0x04) != 0
    }
    /// Enables or disables echo cancellation.
    pub fn set_echo_cancel_enable(&mut self, on: bool) {
        if on {
            self.audio_control_flags_1 |= 0x04;
        } else {
            self.audio_control_flags_1 &= !0x04;
        }
    }

    /// Returns true if noise cancellation is enabled.
    pub fn noise_cancel_enable(&self) -> bool {
        (self.audio_control_flags_1 & 0x08) != 0
    }
    /// Enables or disables noise cancellation.
    pub fn set_noise_cancel_enable(&mut self, on: bool) {
        if on {
            self.audio_control_flags_1 |= 0x08;
        } else {
            self.audio_control_flags_1 &= !0x08;
        }
    }

    /// Returns the audio output path selection (bits 4-5).
    pub fn output_path_select(&self) -> u8 {
        (self.audio_control_flags_1 >> 4) & 0x03
    }
    /// Sets the audio output path selection (bits 4-5).
    pub fn set_output_path_select(&mut self, sel: u8) {
        self.audio_control_flags_1 =
            (self.audio_control_flags_1 & !(0x03 << 4)) | ((sel & 0x03) << 4);
    }

    /// Returns the audio input path selection (bits 6-7).
    pub fn input_path_select(&self) -> u8 {
        (self.audio_control_flags_1 >> 6) & 0x03
    }
    /// Sets the audio input path selection (bits 6-7).
    pub fn set_input_path_select(&mut self, sel: u8) {
        self.audio_control_flags_1 =
            (self.audio_control_flags_1 & !(0x03 << 6)) | ((sel & 0x03) << 6);
    }

    /// Returns the mute light LED mode mode byte.
    pub fn mute_light_mode(&self) -> u8 {
        self.mute_light_mode
    }
    /// Sets the mute light LED mode mode byte.
    pub fn set_mute_light_mode(&mut self, v: u8) {
        self.mute_light_mode = v;
    }

    // --- Power save / Mute control (LSB first) ---
    /// Returns the [PowerSaveMute] bitflags.
    fn power_save_mute_flags(&self) -> PowerSaveMute {
        PowerSaveMute::from_bits_truncate(self.power_save_mute_control)
    }
    /// Sets the [PowerSaveMute] bitflags.
    fn set_power_save_mute_flags(&mut self, f: PowerSaveMute) {
        self.power_save_mute_control = f.bits();
    }

    /// Returns true if power save mode for touchpad is enabled.
    pub fn power_save_touch(&self) -> bool {
        self.power_save_mute_flags()
            .contains(PowerSaveMute::POWER_SAVE_TOUCH)
    }
    /// Enables or disables power save mode for touchpad.
    pub fn set_power_save_touch(&mut self, on: bool) {
        let mut f = self.power_save_mute_flags();
        f.set(PowerSaveMute::POWER_SAVE_TOUCH, on);
        self.set_power_save_mute_flags(f);
    }

    /// Returns true if power save mode for motion sensors is enabled.
    pub fn power_save_motion(&self) -> bool {
        self.power_save_mute_flags()
            .contains(PowerSaveMute::POWER_SAVE_MOTION)
    }
    /// Enables or disables power save mode for motion sensors.
    pub fn set_power_save_motion(&mut self, on: bool) {
        let mut f = self.power_save_mute_flags();
        f.set(PowerSaveMute::POWER_SAVE_MOTION, on);
        self.set_power_save_mute_flags(f);
    }

    /// Returns true if power save mode for haptics is enabled.
    pub fn power_save_haptic(&self) -> bool {
        self.power_save_mute_flags()
            .contains(PowerSaveMute::POWER_SAVE_HAPTIC)
    }
    /// Enables or disables power save mode for haptics.
    pub fn set_power_save_haptic(&mut self, on: bool) {
        let mut f = self.power_save_mute_flags();
        f.set(PowerSaveMute::POWER_SAVE_HAPTIC, on);
        self.set_power_save_mute_flags(f);
    }

    /// Returns true if power save mode for audio is enabled.
    pub fn power_save_audio(&self) -> bool {
        self.power_save_mute_flags()
            .contains(PowerSaveMute::POWER_SAVE_AUDIO)
    }
    /// Enables or disables power save mode for audio.
    pub fn set_power_save_audio(&mut self, on: bool) {
        let mut f = self.power_save_mute_flags();
        f.set(PowerSaveMute::POWER_SAVE_AUDIO, on);
        self.set_power_save_mute_flags(f);
    }

    /// Returns true if the microphone is muted.
    pub fn mute_mic(&self) -> bool {
        self.power_save_mute_flags()
            .contains(PowerSaveMute::MUTE_MIC)
    }
    /// Mutes or unmutes the microphone.
    pub fn set_mute_mic(&mut self, muted: bool) {
        let mut f = self.power_save_mute_flags();
        f.set(PowerSaveMute::MUTE_MIC, muted);
        self.set_power_save_mute_flags(f);
    }

    /// Returns true if the built-in speaker is muted.
    pub fn mute_speaker(&self) -> bool {
        self.power_save_mute_flags()
            .contains(PowerSaveMute::MUTE_SPEAKER)
    }
    /// Mutes or unmutes the built-in speaker.
    pub fn set_mute_speaker(&mut self, muted: bool) {
        let mut f = self.power_save_mute_flags();
        f.set(PowerSaveMute::MUTE_SPEAKER, muted);
        self.set_power_save_mute_flags(f);
    }

    /// Returns true if the headphone output is muted.
    pub fn mute_headphone(&self) -> bool {
        self.power_save_mute_flags()
            .contains(PowerSaveMute::MUTE_HEADPHONE)
    }
    /// Mutes or unmutes the headphone output.
    pub fn set_mute_headphone(&mut self, muted: bool) {
        let mut f = self.power_save_mute_flags();
        f.set(PowerSaveMute::MUTE_HEADPHONE, muted);
        self.set_power_save_mute_flags(f);
    }

    /// Returns true if haptics are muted.
    pub fn mute_haptic(&self) -> bool {
        self.power_save_mute_flags()
            .contains(PowerSaveMute::MUTE_HAPTIC)
    }
    /// Mutes or unmutes haptics.
    pub fn set_mute_haptic(&mut self, on: bool) {
        let mut f = self.power_save_mute_flags();
        f.set(PowerSaveMute::MUTE_HAPTIC, on);
        self.set_power_save_mute_flags(f);
    }

    // --- Audio control flags 2 ---
    /// Returns speaker compensation pre-gain (bits 0-2).
    pub fn speaker_comp_pregain(&self) -> u8 {
        self.audio_control_flags_2 & 0x07
    }
    /// Sets speaker compensation pre-gain (bits 0-2).
    pub fn set_speaker_comp_pregain(&mut self, v: u8) {
        self.audio_control_flags_2 = (self.audio_control_flags_2 & !0x07) | (v & 0x07);
    }

    /// Returns true if microphone beamforming is enabled.
    pub fn beamforming_enable(&self) -> bool {
        (self.audio_control_flags_2 & 0x08) != 0
    }
    /// Enables or disables microphone beamforming.
    pub fn set_beamforming_enable(&mut self, on: bool) {
        if on {
            self.audio_control_flags_2 |= 0x08;
        } else {
            self.audio_control_flags_2 &= !0x08;
        }
    }

    /// Returns upper 4 unknown bits of audio control flags 2.
    pub fn audio_control_flags_2_unknown(&self) -> u8 {
        self.audio_control_flags_2 >> 4
    }
    /// Sets upper 4 unknown bits of audio control flags 2.
    pub fn set_audio_control_flags_2_unknown(&mut self, v: u8) {
        self.audio_control_flags_2 = (self.audio_control_flags_2 & 0x0F) | ((v & 0x0F) << 4);
    }

    // --- Flags 3 ---
    /// Returns [Flags3] bitflags.
    fn flags3(&self) -> Flags3 {
        Flags3::from_bits_truncate(self.flags_3)
    }
    /// Sets [Flags3] bitflags.
    fn set_flags3(&mut self, f: Flags3) {
        self.flags_3 = f.bits();
    }

    /// Returns true if lightbar brightness changes are allowed.
    pub fn allow_light_brightness_change(&self) -> bool {
        self.flags3()
            .contains(Flags3::ALLOW_LIGHT_BRIGHTNESS_CHANGE)
    }
    /// Enables or disables lightbar brightness changes.
    pub fn set_allow_light_brightness_change(&mut self, on: bool) {
        let mut f = self.flags3();
        f.set(Flags3::ALLOW_LIGHT_BRIGHTNESS_CHANGE, on);
        self.set_flags3(f);
    }

    /// Returns true if color light fade animations are allowed.
    pub fn allow_color_light_fade_animation(&self) -> bool {
        self.flags3().contains(Flags3::ALLOW_COLOR_LIGHT_FADE)
    }
    /// Enables or disables color light fade animations.
    pub fn set_allow_color_light_fade_animation(&mut self, on: bool) {
        let mut f = self.flags3();
        f.set(Flags3::ALLOW_COLOR_LIGHT_FADE, on);
        self.set_flags3(f);
    }

    /// Returns true if improved rumble emulation is enabled.
    pub fn enable_improved_rumble_emulation(&self) -> bool {
        self.flags3().contains(Flags3::ENABLE_IMPROVED_RUMBLE)
    }
    /// Enables or disables improved rumble emulation.
    pub fn set_enable_improved_rumble_emulation(&mut self, on: bool) {
        let mut f = self.flags3();
        f.set(Flags3::ENABLE_IMPROVED_RUMBLE, on);
        self.set_flags3(f);
    }

    /// Returns true if the haptic low pass filter is enabled.
    pub fn haptic_low_pass_filter_enabled(&self) -> bool {
        (self.haptic_low_pass_filter & 0x01) != 0
    }
    /// Enables or disables the haptic low pass filter.
    pub fn set_haptic_low_pass_filter_enabled(&mut self, on: bool) {
        if on {
            self.haptic_low_pass_filter |= 0x01;
        } else {
            self.haptic_low_pass_filter &= !0x01;
        }
    }
    /// Returns unknown bits of haptic low pass filter byte.
    pub fn haptic_low_pass_filter_unknown(&self) -> u8 {
        self.haptic_low_pass_filter >> 1
    }
    /// Sets unknown bits of haptic low pass filter byte.
    pub fn set_haptic_low_pass_filter_unknown(&mut self, v: u8) {
        self.haptic_low_pass_filter = (self.haptic_low_pass_filter & 0x01) | ((v & 0x7F) << 1);
    }

    // --- Light and player fields ---
    /// Returns the light fade animation mode byte.
    pub fn light_fade_animation(&self) -> u8 {
        self.light_fade_animation
    }
    /// Sets the light fade animation mode byte.
    pub fn set_light_fade_animation(&mut self, v: u8) {
        self.light_fade_animation = v;
    }

    /// Returns lightbar brightness setting (`0..=2` where 0=bright, 1=medium, 2=dim).
    pub fn light_brightness(&self) -> u8 {
        self.light_brightness
    }
    /// Sets lightbar brightness setting (`0..=2` where 0=bright, 1=medium, 2=dim).
    pub fn set_light_brightness(&mut self, v: u8) {
        self.light_brightness = v;
    }

    /// Returns player indicator LED flags.
    pub fn player_light_flags(&self) -> u8 {
        self.player_light_flags
    }
    /// Sets player indicator LED flags.
    pub fn set_player_light_flags(&mut self, v: u8) {
        self.player_light_flags = v;
    }

    /// Returns the intensity of the red part of the RGB LED (`0..=255`).
    pub fn lightbar_red(&self) -> u8 {
        self.lightbar_red
    }
    /// Sets the intensity of the red part of the RGB LED (`0..=255`).
    pub fn set_lightbar_red(&mut self, v: u8) {
        self.lightbar_red = v;
    }

    /// Returns the intensity of the green part of the RGB LED (`0..=255`).
    pub fn lightbar_green(&self) -> u8 {
        self.lightbar_green
    }
    /// Sets the intensity of the green part of the RGB LED (`0..=255`).
    pub fn set_lightbar_green(&mut self, v: u8) {
        self.lightbar_green = v;
    }

    /// Returns the intensity of the blue part of the RGB LED (`0..=255`).
    pub fn lightbar_blue(&self) -> u8 {
        self.lightbar_blue
    }
    /// Sets the intensity of the blue part of the RGB LED (`0..=255`).
    pub fn set_lightbar_blue(&mut self, v: u8) {
        self.lightbar_blue = v;
    }
}

// Default bit patterns chosen explicitly for readability.
const DEFAULT_FLAGS_1: u8 = Flags1::ENABLE_RUMBLE_EMULATION.bits()
    | Flags1::USE_RUMBLE_NO_HAPTICS.bits()
    | Flags1::ALLOW_RIGHT_TRIGGER_FFB.bits()
    | Flags1::ALLOW_LEFT_TRIGGER_FFB.bits()
    | Flags1::ALLOW_HEADPHONE_VOLUME.bits()
    | Flags1::ALLOW_SPEAKER_VOLUME.bits()
    | Flags1::ALLOW_MIC_VOLUME.bits()
    | Flags1::ALLOW_AUDIO_CONTROL_1.bits();

const DEFAULT_FLAGS_2: u8 = Flags2::ALLOW_MUTE_LIGHT.bits()
    | Flags2::ALLOW_AUDIO_MUTE.bits()
    | Flags2::ALLOW_LED_COLOR.bits()
    | Flags2::ALLOW_PLAYER_INDICATORS.bits()
    | Flags2::ALLOW_HAPTIC_LOW_PASS.bits()
    | Flags2::ALLOW_MOTOR_POWER_LEVEL.bits()
    | Flags2::ALLOW_AUDIO_CONTROL_2.bits();

const DEFAULT_FLAGS_3: u8 = Flags3::ALLOW_LIGHT_BRIGHTNESS_CHANGE.bits()
    | Flags3::ALLOW_COLOR_LIGHT_FADE.bits()
    | Flags3::ENABLE_IMPROVED_RUMBLE.bits();

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

    /// Set trigger motor power reduction (`0..=7`). Value is clamped.
    pub fn set_trigger_motor_power_reduction(&mut self, v: u8) {
        let t = v.min(7) & 0x0F;
        self.motor_power_level = (self.motor_power_level & 0xF0) | t;
    }

    /// Get trigger motor power reduction (`0..=7`).
    pub fn get_trigger_motor_power_reduction(&self) -> u8 {
        self.motor_power_level & 0x0F
    }

    /// Set rumble motor power reduction (`0..=7`). Value is clamped.
    pub fn set_rumble_motor_power_reduction(&mut self, v: u8) {
        let r = (v.min(7) & 0x0F) << 4;
        self.motor_power_level = (self.motor_power_level & 0x0F) | r;
    }

    /// Get rumble motor power reduction (`0..=7`).
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
pub(crate) struct DualSenseOutputReportBT {
    pub(crate) report_id: u8,            // OUTPUT_REPORT_BT_ID
    pub(crate) seq_number_and_flags: u8, // Unknown1 EnableHID1 Unknown2 SequenceNumber4
    pub(crate) tag: u8,                  // OUTPUT_REPORT_BT_TAG
    pub(crate) base: DualSenseOutput,
    pub(crate) reserved: [u8; 24],
    pub(crate) crc32: u32,
}

impl DualSenseOutputReportBT {
    pub(crate) fn add_crc(&mut self) {
        // DualSense Bluetooth hardware requires the 0xA2 seed injected before the payload
        const PS_OUTPUT_CRC32_SEED: u8 = 0xA2;

        let mut hasher = Hasher::new();

        // 1. Feed the mandatory Bluetooth initialization seed
        hasher.update(&[PS_OUTPUT_CRC32_SEED]);

        // 2. Feed the payload bytes (everything except the final 4-byte CRC field)
        let total_len = std::mem::size_of::<Self>();
        let crc_len = std::mem::size_of::<u32>();
        let payload_len = total_len - crc_len;
        hasher.update(&self.as_bytes()[..payload_len]);

        self.crc32 = hasher.finalize();
    }
}

/// Force feedback component of [DualSenseOutput]
#[repr(C, packed)]
#[derive(IntoBytes, FromZeros, Immutable, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TriggerFFB {
    /// Force feedback mode for more info on this variable see the various functions for this struct
    pub mode: u8,
    /// Depending on [TriggerFFB::mode] representing different forces or positions
    pub parameters: [u8; 10],
}

impl TriggerFFB {
    /// No force feedback
    pub fn off() -> Self {
        Self::new_zeroed()
    }

    /// Initialize with feedback mode
    pub fn feedback(start_position: u8, strength: u8) -> Self {
        let mut effect = Self::new_zeroed();
        effect.mode = 0x01;
        effect.parameters[0] = start_position; // Start position (0-255)
        effect.parameters[1] = strength; // Resistance strength
        effect
    }

    /// Initialize with weapon mode
    /// If start > end there will only be a short impulse at end
    pub fn weapon(start: u8, end: u8, strength: u8) -> Self {
        let mut effect = Self::new_zeroed();
        effect.mode = 0x02;
        effect.parameters[0] = start;
        effect.parameters[1] = end;
        effect.parameters[2] = strength;
        effect
    }

    /// Fully disengages the force feedback motor and turns off force feedback
    pub fn disengage() -> Self {
        let mut effect = Self::new_zeroed();
        effect.mode = 0x05;
        effect
    }

    /// Vibration mode. Frequency is in Hertz up to a certain point and then loses granularity and
    /// accuracy. vibration_strength is clamped between 0 and 63
    pub fn vibration(start: u8, frequency: u8, vibration_strength: u8) -> Self {
        let mut effect = Self::new_zeroed();
        effect.mode = 0x06;
        effect.parameters[0] = frequency;
        effect.parameters[1] = vibration_strength.clamp(0, 63);
        effect.parameters[2] = start;
        effect
    }

    /// Bow trigger effect. Only the lower 3 bits of strength and snap_force are used.
    /// All other bits are masked out.
    pub fn bow(start: u8, end: u8, strength: u8, snap_force: u8) -> Self {
        let mut effect = Self::new_zeroed();
        effect.mode = 0x22;
        effect.parameters[0] = start;
        effect.parameters[1] = end;
        // lower 3 bits are strength bits 4..=6 are snap_force
        effect.parameters[2] = (strength & 0x07) | ((snap_force & 0x07) << 3);
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

    #[test]
    fn get_set_enable_rumble_emulation() {
        let mut out = DualSenseOutput::default();
        assert!(out.enable_rumble_emulation());
        out.set_enable_rumble_emulation(false);
        assert!(!out.enable_rumble_emulation());
        out.set_enable_rumble_emulation(true);
        assert!(out.enable_rumble_emulation());
    }

    #[test]
    fn get_set_use_rumble_no_haptics() {
        let mut out = DualSenseOutput::default();
        assert!(out.use_rumble_no_haptics());
        assert!(out.enable_rumble_emulation());
        out.set_use_rumble_no_haptics(false);
        assert!(!out.use_rumble_no_haptics());
        assert!(out.enable_rumble_emulation());
        out.set_use_rumble_no_haptics(true);
        assert!(out.use_rumble_no_haptics());
        assert!(out.enable_rumble_emulation());
    }
}
