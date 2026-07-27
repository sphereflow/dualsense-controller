# dualsense-controller

Rust library for interfacing with a PS5 dualsense controller over USB and Bluetooth.

## Quick Start 

```rust
use dualsense_controller::DualSense;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut controller = DualSense::run()?;
    controller.set_led_color(0, 255, 0); // Green LED

    loop {
        controller.update_input();
        if controller.is_button_down(dualsense_controller::button::Button::Cross) {
            println!("Cross button pressed!");
        }
        thread::sleep(Duration::from_millis(16));
    }
}
