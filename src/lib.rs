#![doc = include_str!("../README.md")]
#![allow(unused_variables, dead_code)]

use std::error::Error;

/// The entry point: create a builder with as much information
/// as possible and create a device from that, then query the
/// device for the information the caller needs to know.
///
/// # Example
/// ```
/// use whodat::{Builder, Capability};
/// if let Ok(device) = Builder::new()
///                     .name("Sony Playstation Controller")
///                     .usbid(0x1234, 0x56ab)
///                     .build() {
///     match device.has_capability(Capability::Touchpad) {
///         Some(value) => println!("This device is a touchpad? {}", value),
///         None => println!("I really don't know what this device is"),
///     }
/// }
/// ```
///
/// Note that the order determines the priority, i.e. where
/// a [`Builder::udev_device`] is given first and the [`Builder::name`] second,
/// the latter will override the name as queried from the udev device.
pub struct Builder {}

impl Builder {
    /// Create a new instance of a [`Builder`].
    pub fn new() -> Self {
        Builder {}
    }

    /// Set the device name as advertised by the kernel
    pub fn name(&mut self, name: &str) -> &mut Self {
        self
    }

    /// The USB vendor and product ID
    pub fn usbid(&mut self, vid: u16, pid: u16) -> &mut Self {
        self
    }

    /// The udev device representing this device
    pub fn udev_device(&mut self, path: &str) -> &mut Self {
        self
    } // FIXME: needs to be some udev type, not a path

    /// An open evdev file descriptor that can be `ioctl`'d for information
    pub fn evdev_fd(&mut self, fd: std::os::fd::RawFd) -> &mut Self {
        self
    }

    /// An open hidraw file descriptor that can be `ioctl`'d for information
    pub fn hidraw_fd(&mut self, fd: std::os::fd::RawFd) -> &mut Self {
        self
    }

    /// Path to the device's sysfs entry. If this path does not start with `/sys`,
    /// it is automatically prefixed as such.
    pub fn sysfs_path(&mut self, path: &str) -> &mut Self {
        self
    }

    /// Build the device. If this function returns an error, the provided information
    /// is insufficient to construct a [`Device`].
    pub fn build(&self) -> Result<Device, Box<dyn Error>> {
        Ok(Device {})
    }
}

/// A high-level category describing a capability on this device.
/// Capabilities are not mutually exclusive (some are, see the documentation)
/// and any device may match one or more of those capbilities.
///
/// The availability of capabilities depends on how the device was
/// constructed.
///
/// A caller is expected to check the categories they care about
/// (both for "has" and "has not") and treat the device
/// accordingly. For example, a caller expecting a mouse should check
/// that the [`Capability::Pointer`] is present but the
/// [`Capability::Touchpad`] (amongst others) is not present.
#[non_exhaustive]
#[derive(Debug)]
pub enum Capability {
    Keyboard,
    Pointer,
    Pointingstick,
    Touchpad,
    Touchscreen,
    Trackball,
    Joystick,
    Gamepad,
    Tablet,
    TabletPad,
}

/// Describes the **physical** type of this device. Unlike the [`Device::has_capability`]
/// a device may only have one physical type. For example, modern PlayStation controllers
/// provide a touchpad as well as a gamepad - the physical type of this controller however
/// is [`PhysicalType::Gamepad`].
///
/// The physical type of the device may not always be known, especially if the device
/// is constructed from a single event node via [`Builder::evdev_fd`]. This crate may
/// rely on an internal database for well-known devices to supplement the information
/// where posssible.
#[non_exhaustive]
#[derive(Debug)]
pub enum PhysicalType {
    Keyboard,
    Mouse,
    Pointingstick,
    Touchpad,
    Touchscreen,
    Trackball,
    Tablet,
    Joystick,
    Gamepad,
    RacingWheel,
    FootPedal,
}

/// The Linux kernel splits HID devices up by application and a single
/// HID device may result in multiple evdev nodes.
#[non_exhaustive]
pub enum Application {
    Mouse,
    Touchpad,
    Keyboard,
    Keypad,
    ConsumerControl,
    SystemControl,
}

/// The [`Device`] struct represents the device and the queriable
/// information about this device.
pub struct Device {}

impl Device {
    /// Returns the physical type of this device. Unlike [`Device::has_capability`]
    /// a device is only of one physical type even where it supports multiple different
    /// input methods.
    pub fn physical_type(self) -> Option<PhysicalType> {
        None
    }

    /// Return a result on whether the device has the given capability.
    /// If the capability is known or can be guessed, the result is `true`
    /// or `false`. Otherwise if this cannot be known based on the
    /// data supplied prior to the device creation, `None` is returned.
    pub fn has_capability(self, capability: Capability) -> Option<bool> {
        Some(false)
    }

    // /// Return the HID application this device is mapped to.
    // /// This is a feature of the Linux kernel that HID devices are split
    // /// across various evdev nodes, typically by HID Application. For example
    // /// a mouse device is often split into a [`Application::Mouse`] and
    // /// a [`Application::Keyboard`] device.
    // ///
    // /// Where a device originates from an evdev node (see [`Builder::evdev_fd`])
    // /// this function returns the application that the evdev node represents, if any.
    // /// Otherwise, this function returns None.
    // pub fn hid_application(self) -> Option<Application> {
    //     None
    // }

    /// Returns a library-specific dump of this device that can be used to recreate
    /// the information with [`Device::deserialize`].
    ///
    /// The format of this dump is implementation-private and should not be used
    /// outside users of this crate.
    pub fn serialize(self) -> Option<String> {
        None
    }

    /// Create a new device from the data returned by [`Device::serialize`].
    /// This way a caller that has more information about a
    /// device can transmit data through other channels to a different caller and
    /// that second caller can reliably recreate the information of this
    /// device even without access to the device itself.
    pub fn deserialize(data: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Device {})
    }

    // /// Returns a confidence level between `[0.0, 1.0]` on
    // /// how confident we are the classification of this device
    // /// is correct. This is a summary level, individual capabilities
    // /// may have different confidence levels but that is hopefully
    // /// less of an real-world issue than expected.
    // pub fn confidence(self) -> f32 {
    //     return 0.0;
    // }
}
