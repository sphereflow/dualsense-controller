use crossbeam_channel::{Receiver, Sender, unbounded};
use hidapi::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use thiserror::Error;
use zerocopy::{IntoBytes, TryFromBytes};

use crate::button::Button;
use crate::input::{DualSenseInputReportBT, DualSenseInputReportSimpleBT, DualSenseInputUSB};
use crate::output::DualSenseOutput;
use crate::output::{DualSenseOutputReportBT, TriggerFFB};
use crate::report::Report;

pub mod button;
pub mod input;
pub mod output;
pub mod report;

pub const VENDOR_ID: u16 = 0x054C;
pub const DUALSENSE_PRODUCT_ID: u16 = 0x0CE6;
pub const DUALSENSE_EDGE_PRODUCT_ID: u16 = 0x0DF2;

// Report IDs
pub const INPUT_REPORT_USB_ID: u8 = 0x01;
pub const INPUT_REPORT_BT_ID: u8 = 0x31;
pub const OUTPUT_REPORT_USB_ID: u8 = 0x02;
pub const OUTPUT_REPORT_BT_ID: u8 = 0x31;
pub const OUTPUT_REPORT_BT_TAG: u8 = 0x10;

#[derive(Debug)]
pub struct DualSense {
    input_channel: Receiver<DualSenseInputUSB>,
    pub last_input: DualSenseInputUSB,
    diff_pressed: DualSenseInputUSB,
    diff_released: DualSenseInputUSB,
    current_output: DualSenseOutput,
    output_channel: Sender<DualSenseOutput>,
    join_handle: Option<JoinHandle<Result<(), DualSenseError>>>,
    running: Arc<AtomicBool>,
    pub is_bluetooth: bool,
}

impl DualSense {
    pub fn run() -> Result<DualSense, DualSenseError> {
        let api = HidApi::new()?;
        let (device, is_bluetooth) = Self::connect(&api)?;
        let (send_input, input_channel) = unbounded();
        let (output_channel, receive_output) = unbounded();
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
            last_input: DualSenseInputUSB::default(),
            diff_pressed: DualSenseInputUSB::default(),
            diff_released: DualSenseInputUSB::default(),
            current_output: DualSenseOutput::default(),
            output_channel,
            join_handle: Some(join_handle),
            running,
            is_bluetooth,
        })
    }

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

    fn update_thread(
        device: HidDevice,
        is_bluetooth: bool,
        running: Arc<AtomicBool>,
        send_input: Sender<DualSenseInputUSB>,
        receive_output: Receiver<DualSenseOutput>,
    ) -> Result<(), DualSenseError> {
        // enable_extended_mode(&device);
        let mut input_report_buffer = [0u8; 128];
        let mut input_packet_num = 0;
        let mut output_packet_num = 0;
        let mut output_seq_tag_bt: u8 = 0;

        while running.load(Ordering::Relaxed) {
            // 1. Process any pending outputs first
            while let Ok(output) = receive_output.try_recv() {
                if !is_bluetooth {
                    let report: Report<DualSenseOutput, { OUTPUT_REPORT_USB_ID }> = Report::new(output);
                    device.write(report.as_bytes())?;
                } else {
                    let mut report = DualSenseOutputReportBT {
                        report_id: OUTPUT_REPORT_BT_ID,
                        seq_number_and_flags: output_seq_tag_bt << 4,
                        tag: OUTPUT_REPORT_BT_TAG,
                        base: output,
                        reserved: [0;24],
                        crc32: 0,
                    };
                    report.add_crc();
                    device.write(report.as_bytes())?;
                    output_seq_tag_bt += 1;
                    output_seq_tag_bt %= 16;
                }
                output_packet_num += 1;
                if output_packet_num % 100 == 0 {
                    dbg!(output_packet_num);
                }
            }

            // 2. Perform a blocking read with a short timeout (e.g. 4ms) to wait for controller data.
            // This yields the thread to the OS and wakes up immediately when the controller sends a packet,
            // or after 4ms if no packet was sent (allowing us to check for new output messages).
            match device.read_timeout(&mut input_report_buffer, 4) {
                Ok(size) if size > 0 => {
                    match input_report_buffer[0] {
                        INPUT_REPORT_USB_ID => {
                            if is_bluetooth {
                                if let Ok(report) = DualSenseInputReportSimpleBT::try_read_from_prefix(
                                    &input_report_buffer[..size],
                                ) {
                                    send_input.send(report.0.into())?;
                                    input_packet_num += 1;
                                }
                            } else {
                                type Rep = Report<DualSenseInputUSB, 1>;
                                if let Ok(report) = Rep::try_read_from_prefix(
                                    &input_report_buffer[..size],
                                ) {
                                    send_input.send(report.0.base)?;
                                    input_packet_num += 1;
                                }
                            }
                        }
                        INPUT_REPORT_BT_ID => {
                            if let Ok(report) = DualSenseInputReportBT::try_read_from_prefix(&input_report_buffer[..size]) {
                                send_input.send(report.0.base)?;
                                input_packet_num += 1;
                            }
                        }
                        byte => {
                            eprintln!("received unknown input report buffer byte: {byte}");
                        }
                    }
                }
                Ok(_) => {
                    // Timeout with no data. Just loop again to check running state and outputs.
                }
                Err(e) => {
                    eprintln!("Error reading from DualSense: {}", e);
                    return Err(e.into());
                }
            }

            if input_packet_num % 1000 == 0 && input_packet_num > 0 {
                dbg!(input_packet_num);
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

    pub fn set_rumble_left(&mut self, left: u8, power_reduction: u8) -> Result<(), DualSenseError> {
        self.set_rumble(left, self.current_output.rumble_right, power_reduction)
    }

    pub fn set_rumble_right(
        &mut self,
        right: u8,
        power_reduction: u8,
    ) -> Result<(), DualSenseError> {
        self.set_rumble(self.current_output.rumble_left, right, power_reduction)
    }

    fn set_rumble(
        &mut self,
        left: u8,
        right: u8,
        power_reduction: u8,
    ) -> Result<(), DualSenseError> {
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
            self.output_channel.send(self.current_output)?;
        }
        Ok(())
    }

    pub fn set_triggers(
        &mut self,
        left: TriggerFFB,
        right: TriggerFFB,
    ) -> Result<(), DualSenseError> {
        let old_output = self.current_output;
        self.current_output.set_allow_left_trigger_ffb(true); // Enable Trigger Effects
        self.current_output.set_allow_right_trigger_ffb(true); // Enable Trigger Effects
        self.current_output.left_trigger_ffb = left;
        self.current_output.right_trigger_ffb = right;
        if old_output != self.current_output {
            self.output_channel.send(self.current_output)?;
        }
        Ok(())
    }

    pub fn set_led_color(&mut self, r: u8, g: u8, b: u8) -> Result<(), DualSenseError> {
        let old_output = self.current_output;
        self.current_output.set_allow_led_color(true);
        self.current_output.set_reset_lights(false);
        self.current_output.set_light_fade_animation(0);
        self.current_output.set_mute_light_mode(0);
        self.current_output.set_lightbar_red(r);
        self.current_output.set_lightbar_green(g);
        self.current_output.set_lightbar_blue(b);
        if old_output != self.current_output {
            self.output_channel.send(self.current_output)?;
        }
        Ok(())
    }

    pub fn clear_effects(&mut self) -> Result<(), DualSenseError> {
        let old_output = self.current_output;
        // We set the flags to 1 to tell the controller "update these fields"
        // Since the fields themselves are zeroed (via new_zeroed), the hardware turns off.
        self.current_output.flags_1 = 0x01 | 0x04; // Enable Rumble + Trigger update
        self.current_output.flags_2 = 0x01; // Enable Lightbar update

        // The values for rumble_left, rumble_right, lightbar, and triggers
        // are already 0x00 thanks to new_zeroed().

        if old_output != self.current_output {
            self.output_channel.send(self.current_output)?;
        }
        Ok(())
    }
}

impl Drop for DualSense {
    fn drop(&mut self) {
        self.clear_effects().ok();
        thread::sleep(Duration::from_millis(5));
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.join_handle.take() {
            handle.join().ok();
        }
    }
}

#[derive(Error, Debug)]
pub enum DualSenseError {
    #[error("HID device error: {0}")]
    HidError(#[from] hidapi::HidError),

    #[error("Controller disconnected")]
    Disconnected,

    #[error("Invalid report ID received: {0}")]
    InvalidReport(u8),

    #[error("Failed to parse packet: {0}")]
    ParseError(String),

    #[error("Channel receive error")]
    ChannelRecvError(#[from] crossbeam_channel::RecvError),

    #[error("Channel send error: Output")]
    ChannelSendErrorOutput(#[from] crossbeam_channel::SendError<DualSenseOutput>),

    #[error("Channel send error: Input")]
    ChannelSendErrorInput(#[from] crossbeam_channel::SendError<DualSenseInputUSB>),
}
