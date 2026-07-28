#![warn(missing_docs)]

//! This crate provides access to the Playstation 5 DualSense controller through hidapi

use crossbeam_channel::{Receiver, Sender, unbounded};
use hidapi::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use thiserror::Error;
use zerocopy::{FromZeros, IntoBytes, TryFromBytes};

use crate::button::Button;
use crate::input::{DualSenseInput, DualSenseInputReportBT, DualSenseInputReportSimpleBT};
use crate::output::{DualSenseOutput, Flags1, Flags2};
use crate::output::{DualSenseOutputReportBT, TriggerFFB};
use crate::report::Report;

/// module for Button struct
pub mod button;
/// module for input related structs
pub mod input;
/// module for output related structs
pub mod output;
mod report;

const VENDOR_ID: u16 = 0x054C;
const DUALSENSE_PRODUCT_ID: u16 = 0x0CE6;
const DUALSENSE_EDGE_PRODUCT_ID: u16 = 0x0DF2;

// Report IDs
const INPUT_REPORT_USB_ID: u8 = 0x01;
const INPUT_REPORT_BT_ID: u8 = 0x31;
const OUTPUT_REPORT_USB_ID: u8 = 0x02;
const OUTPUT_REPORT_BT_ID: u8 = 0x31;
const OUTPUT_REPORT_BT_TAG: u8 = 0x10;

/// This is the main struct representing the controller
#[derive(Debug)]
pub struct DualSense {
    input_channel: Receiver<DualSenseInput>,
    last_input: DualSenseInput,
    diff_pressed: DualSenseInput,
    diff_released: DualSenseInput,
    current_output: DualSenseOutput,
    output_channel: Arc<Mutex<Option<DualSenseOutput>>>,
    join_handle: Option<JoinHandle<Result<(), DualSenseError>>>,
    running: Arc<AtomicBool>,
    is_bluetooth: bool,
    deadzone_left: (f32, f32),
    deadzone_right: (f32, f32),
}

impl DualSense {
    /// DualSense constructor also runs the thread to get the input state from the controller
    /// and push the output to the controller
    pub fn run() -> Result<DualSense, DualSenseError> {
        let api = HidApi::new()?;
        let (device, is_bluetooth) = Self::connect(&api)?;
        let (send_input, input_channel) = unbounded();
        let output_channel = Arc::new(Mutex::new(None));
        let receive_output = output_channel.clone();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        let join_handle = thread::spawn(move || {
            Self::update_thread(
                device,
                is_bluetooth,
                running_clone,
                send_input,
                receive_output,
            )
        });
        Ok(DualSense {
            input_channel,
            last_input: DualSenseInput::default(),
            diff_pressed: DualSenseInput::default(),
            diff_released: DualSenseInput::default(),
            current_output: DualSenseOutput::default(),
            output_channel,
            join_handle: Some(join_handle),
            running,
            is_bluetooth,
            deadzone_left: (0.0, 0.0),
            deadzone_right: (0.0, 0.0),
        })
    }

    /// Connects to the controller. This function fails if there is no controller present or if it
    /// did not manage to open the controller
    pub fn connect(api: &hidapi::HidApi) -> Result<(hidapi::HidDevice, bool), DualSenseError> {
        // 1. Find the first matching device
        let device_info = api
            .device_list()
            .find(|d| {
                d.vendor_id() == VENDOR_ID
                    && (d.product_id() == DUALSENSE_PRODUCT_ID
                        || d.product_id() == DUALSENSE_EDGE_PRODUCT_ID)
            })
            .ok_or(DualSenseError::Disconnected)?;

        // 2. Determine if it's Bluetooth
        // Bluetooth devices typically report -1 for the interface number
        let is_bluetooth = device_info.interface_number() == -1;

        // 3. Open it
        let device = device_info.open_device(api)?;

        Ok((device, is_bluetooth))
    }

    /// Returns true if the controller is connected
    pub fn is_device_connected(&self) -> bool {
        if !self
            .join_handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(true)
        {
            true
        } else {
            self.running.store(false, Ordering::Relaxed);
            false
        }
    }

    /// This function returns true if the device is connected via Bluetooth
    pub fn is_bluetooth(&self) -> bool {
        self.is_bluetooth
    }

    fn update_thread(
        device: HidDevice,
        is_bluetooth: bool,
        running: Arc<AtomicBool>,
        send_input: Sender<DualSenseInput>,
        receive_output: Arc<Mutex<Option<DualSenseOutput>>>,
    ) -> Result<(), DualSenseError> {
        // enable_extended_mode(&device);
        let mut input_report_buffer = [0u8; 128];
        let mut output_seq_tag_bt: u8 = 0;

        while running.load(Ordering::Relaxed) {
            // 1. Extract pending output and release the mutex lock IMMEDIATELY
            // before performing the blocking device.write() call.
            let pending_output = receive_output
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take();

            if let Some(output) = pending_output {
                if !is_bluetooth {
                    let report: Report<DualSenseOutput, { OUTPUT_REPORT_USB_ID }> =
                        Report::new(output);
                    device.write(report.as_bytes())?;
                } else {
                    let mut report = DualSenseOutputReportBT {
                        report_id: OUTPUT_REPORT_BT_ID,
                        seq_number_and_flags: output_seq_tag_bt << 4,
                        tag: OUTPUT_REPORT_BT_TAG,
                        base: output,
                        reserved: [0; 24],
                        crc32: 0,
                    };
                    report.add_crc();
                    device.write(report.as_bytes())?;
                    output_seq_tag_bt += 1;
                    output_seq_tag_bt %= 16;
                }
            }

            // 2. Perform a blocking read with a short timeout (e.g. 4ms) to wait for controller data.
            // This yields the thread to the OS and wakes up immediately when the controller sends a packet,
            // or after 4ms if no packet was sent (allowing us to check for new output messages).
            match device.read_timeout(&mut input_report_buffer, 4) {
                Ok(size) if size > 0 => match input_report_buffer[0] {
                    INPUT_REPORT_USB_ID => {
                        if is_bluetooth {
                            if let Ok(report) = DualSenseInputReportSimpleBT::try_read_from_prefix(
                                &input_report_buffer[..size],
                            ) {
                                send_input.send(report.0.into())?;
                            }
                        } else {
                            type Rep = Report<DualSenseInput, 1>;
                            if let Ok(report) =
                                Rep::try_read_from_prefix(&input_report_buffer[..size])
                            {
                                send_input.send(report.0.base)?;
                            }
                        }
                    }
                    INPUT_REPORT_BT_ID => {
                        if let Ok(report) = DualSenseInputReportBT::try_read_from_prefix(
                            &input_report_buffer[..size],
                        ) {
                            send_input.send(report.0.base)?;
                        }
                    }
                    byte => {
                        eprintln!("received unknown input report buffer byte: {byte}");
                    }
                },
                Ok(_) => {
                    // Timeout with no data. Just loop again to check running state and outputs.
                }
                Err(e) => {
                    eprintln!("Error reading from DualSense: {}", e);
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }

    /// gets the last input state discarding all other inputs till now
    pub fn update_input(&mut self) {
        let old = self.last_input;
        if let Some(input) = self.input_channel.try_iter().last() {
            self.last_input = input;
        }
        self.diff_pressed = self.last_input.diff(&old);
        self.diff_released = old.diff(&self.last_input);
    }

    /// Probes the left stick values for the time of duration. Duration defaults to 1 second.
    /// The result is the maximum deviation from the zero position normalized to the range of 0.0 .. 1.0.
    /// The result is returned in coordinate form (x, y). Since the hardware features 2
    /// potentiometers 2 maximum values are recorded.
    /// Note: outliers will scew the results.
    pub fn probe_dead_zone_left(&mut self, duration: Option<Duration>) -> (f32, f32) {
        let start = Instant::now();
        let max_duration = duration.unwrap_or(Duration::from_secs(1));
        let mut stick_x_max: f32 = 0.;
        let mut stick_y_max: f32 = 0.;
        loop {
            self.update_input();
            let duration = Instant::now() - start;
            if duration > max_duration {
                break;
            }
            stick_x_max = stick_x_max.max((self.last_input.left_stick().0 as f32 - 128.).abs());
            stick_y_max = stick_y_max.max((self.last_input.left_stick().1 as f32 - 128.).abs());
            thread::sleep(Duration::from_millis(1));
        }
        let x_normalized: f32 = stick_x_max / 128.;
        let y_normalized: f32 = stick_y_max / 128.;
        (x_normalized, y_normalized)
    }

    /// Probes the right stick values for the time of duration. Duration defaults to 1 second.
    /// The result is the maximum deviation from the zero position normalized to the range of 0.0 .. 1.0.
    /// The result is returned in coordinate form (x, y). Since the hardware features 2
    /// potentiometers 2 maximum values are recorded.
    /// Note: outliers will scew the results.
    pub fn probe_dead_zone_right(&mut self, duration: Option<Duration>) -> (f32, f32) {
        let start = Instant::now();
        let max_duration = duration.unwrap_or(Duration::from_secs(1));
        let mut stick_x_max: f32 = 0.;
        let mut stick_y_max: f32 = 0.;
        loop {
            self.update_input();
            let duration = Instant::now() - start;
            if duration > max_duration {
                break;
            }
            stick_x_max = stick_x_max.max((self.last_input.right_stick().0 as f32 - 128.).abs());
            stick_y_max = stick_y_max.max((self.last_input.right_stick().1 as f32 - 128.).abs());
            thread::sleep(Duration::from_millis(1));
        }
        let x_normalized: f32 = stick_x_max / 128.;
        let y_normalized: f32 = stick_y_max / 128.;
        (x_normalized, y_normalized)
    }

    /// Sets the deadzone of the left stick values range from 0.0 to 0.99.
    /// Values out of range will be clamped
    pub fn set_dead_zone_left(&mut self, deadzone: (f32, f32)) {
        self.deadzone_left = (deadzone.0.clamp(0.0, 0.99), deadzone.1.clamp(0.0, 0.99));
    }

    /// Sets the deadzone of the right stick values range from 0.0 to 1.0.
    /// Values out of range will be clamped
    pub fn set_dead_zone_right(&mut self, deadzone: (f32, f32)) {
        self.deadzone_right = (deadzone.0.clamp(0.0, 0.99), deadzone.1.clamp(0.0, 0.99));
    }

    /// Gets the current (x, y) coordinates of the left stick.
    /// Values range from -1.0 to 1.0.
    /// This function also applys deadzones.
    pub fn get_left_stick_normalized(&self) -> (f32, f32) {
        let x_coord = (self.last_input.left_stick().0 as f32 - 128.0) / 128.0;
        let y_coord = (self.last_input.left_stick().1 as f32 - 128.0) / 128.0;
        apply_deadzone((x_coord, y_coord), self.deadzone_left)
    }

    /// Gets the current (x, y) coordinates of the right stick.
    /// Values range from -1.0 to 1.0.
    /// This function also applys deadzones.
    pub fn get_right_stick_normalized(&self) -> (f32, f32) {
        let x_coord = (self.last_input.right_stick().0 as f32 - 128.0) / 128.0;
        let y_coord = (self.last_input.right_stick().1 as f32 - 128.0) / 128.0;
        apply_deadzone((x_coord, y_coord), self.deadzone_right)
    }

    /// returns if the button was held down while calling update_input
    pub fn is_button_down(&self, button: Button) -> bool {
        self.last_input.is_button_down(button)
    }

    /// returns true only if the button was pressed between 2 calls of update_input
    pub fn button_pressed(&self, button: Button) -> bool {
        self.diff_pressed.is_button_down(button)
    }

    /// returns true only if the button was released between 2 calls of update_input
    pub fn button_released(&self, button: Button) -> bool {
        self.diff_released.is_button_down(button)
    }

    fn send_current_output(&mut self) {
        let mut guard = self
            .output_channel
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        guard.replace(self.current_output);
    }

    /// Writes the rumble state for the left side to the controller
    pub fn set_rumble_left(&mut self, left: u8, power_reduction: u8) {
        self.set_rumble(left, self.current_output.rumble_right, power_reduction);
    }

    /// Writes the rumble state for the right side to the controller
    pub fn set_rumble_right(&mut self, right: u8, power_reduction: u8) {
        self.set_rumble(self.current_output.rumble_left, right, power_reduction);
    }

    fn set_rumble(&mut self, left: u8, right: u8, power_reduction: u8) {
        let old_output = self.current_output;
        self.current_output.set_use_rumble_no_haptics(true);
        self.current_output.set_rumble_left(left);
        self.current_output.set_rumble_right(right);
        self.current_output
            .set_rumble_motor_power_reduction(power_reduction);
        let diff_haptics =
            old_output.use_rumble_no_haptics() != self.current_output.use_rumble_no_haptics();
        let diff_rumble_left = old_output.rumble_left != self.current_output.rumble_left;
        let diff_rumble_right = old_output.rumble_right != self.current_output.rumble_right;
        let diff_power_reduction = old_output.get_rumble_motor_power_reduction()
            != self.current_output.get_rumble_motor_power_reduction();
        if diff_haptics || diff_rumble_left || diff_rumble_right || diff_power_reduction {
            self.send_current_output();
        }
    }

    /// Set the force feedback mode for both triggers
    pub fn set_triggers(&mut self, left: TriggerFFB, right: TriggerFFB) {
        let old_output = self.current_output;
        self.current_output.set_allow_left_trigger_ffb(true); // Enable Trigger Effects
        self.current_output.set_allow_right_trigger_ffb(true); // Enable Trigger Effects
        self.current_output.left_trigger_ffb = left;
        self.current_output.right_trigger_ffb = right;
        if old_output != self.current_output {
            self.send_current_output();
        }
    }

    /// Sets the color of the RGB LED
    pub fn set_led_color(&mut self, r: u8, g: u8, b: u8) {
        let old_output = self.current_output;
        self.current_output.set_allow_led_color(true);
        self.current_output.set_reset_lights(false);
        self.current_output.set_light_fade_animation(0);
        self.current_output.set_mute_light_mode(0);
        self.current_output.set_lightbar_red(r);
        self.current_output.set_lightbar_green(g);
        self.current_output.set_lightbar_blue(b);
        if old_output != self.current_output {
            self.send_current_output();
        }
    }

    /// Clears all output effects
    pub fn clear_effects(&mut self) {
        let old_output = self.current_output;
        self.current_output = DualSenseOutput::new_zeroed();
        // We set the flags to 1 to tell the controller "update these fields"
        // Since the fields themselves are zeroed (via new_zeroed), the hardware turns off.
        self.current_output.flags_1 = (Flags1::ENABLE_RUMBLE_EMULATION
            | Flags1::ALLOW_RIGHT_TRIGGER_FFB
            | Flags1::ALLOW_LEFT_TRIGGER_FFB)
            .bits(); // Enable Rumble + Trigger update
        self.current_output.flags_2 = Flags2::ALLOW_MUTE_LIGHT.bits(); // Enable Lightbar update

        // The values for rumble_left, rumble_right, lightbar, and triggers
        // are already 0x00 thanks to new_zeroed().

        if old_output != self.current_output {
            self.send_current_output();
        }
    }

    /// Getter for [self.last_input] which is raw input from the device.
    /// That means no post processing has been done to it.
    pub fn get_last_input(&self) -> DualSenseInput {
        self.last_input
    }
}

fn apply_deadzone((x, y): (f32, f32), (dz_x, dz_y): (f32, f32)) -> (f32, f32) {
    let res_x = if x.abs() < dz_x {
        0.0
    } else {
        (x - dz_x * x.signum()) / (1.0 - dz_x)
    };
    let res_y = if y.abs() < dz_y {
        0.0
    } else {
        (y - dz_y * y.signum()) / (1.0 - dz_y)
    };
    (res_x, res_y)
}

impl Drop for DualSense {
    fn drop(&mut self) {
        self.clear_effects();
        thread::sleep(Duration::from_millis(5));
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.join_handle.take() {
            handle.join().ok();
        }
    }
}

/// Error enum for the controller
#[derive(Error, Debug)]
pub enum DualSenseError {
    /// Errors concerning the HID API
    #[error("HID device error: {0}")]
    HidError(#[from] hidapi::HidError),

    /// Error variant for loss of the controller HID device
    #[error("Controller disconnected")]
    Disconnected,

    /// Error variant for if the received report ID is not valid
    #[error("Invalid report ID received: {0}")]
    InvalidReport(u8),

    /// Error variant if parsing a report failed
    #[error("Failed to parse packet: {0}")]
    ParseError(String),

    /// Error variant for if reception of an input report through the channel failed
    #[error("Channel receive error")]
    ChannelRecvError(#[from] crossbeam_channel::RecvError),

    /// Error variant for if the sending of an input report through the channel failed
    #[error("Channel send error: Input")]
    ChannelSendErrorInput(#[from] crossbeam_channel::SendError<DualSenseInput>),
}
