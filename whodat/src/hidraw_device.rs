use crate::*;

/// The [`HidrawDevice`] struct represents a single kernel device and
/// the queryable information about this device.
pub struct HidrawDevice {
    /// Attachment in the [`DeviceTree`]
    node: Option<Node>,
}

impl HasParent for HidrawDevice {
    fn parent(&self) -> DeviceIndex {
        assert!(self.node.is_some());
        self.node.unwrap().parent.unwrap().clone()
    }
}

impl HidrawDevice {
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
