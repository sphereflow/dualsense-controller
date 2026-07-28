/// Button enum
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    /// Up button
    DpadUp,
    /// Down button
    DpadDown,
    /// Left button
    DpadLeft,
    /// Right button
    DpadRight,
    /// Square button
    Square,
    /// Cross button
    Cross,
    /// Circle button
    Circle,
    /// Triangle button
    Triangle,
    /// Left button under the index finger
    L1,
    /// Right button under the index finger
    R1,
    /// Left button under the middle finger
    L2,
    /// Riht button under the middle finger
    R2,
    /// Button under the left stick
    L3,
    /// Button under the right stick
    R3,
    /// Playstation on/off button
    PS,
    /// button under the touchpad
    Touchpad,
    /// Mute button
    Mute,
    /// Create button
    Create,
    /// Menu button
    Menu,
}
