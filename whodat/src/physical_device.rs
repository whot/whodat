use crate::*;

use std::collections::HashSet;
use std::path::PathBuf;

/// The [`PhysicalDevice`] struct represents the device and the queryable
/// information about this (physical) device.
///
/// This is a high-level device and represents the whole physical device.
/// For example, for a Sony Playstation 5 controller, this represents
/// the controller which itself has subdevices for the gaming features and
/// the touchpad (and possibly others). For a Wacom Intuos Pro series tablet
/// this is a tablet, even though that tablet also has a touchscreen.
#[derive(Debug)]
pub struct PhysicalDevice {
    /// Attachment in the [`DeviceTree`]
    node: Node,
    abstract_types: Vec<AbstractType>,
    caps: HashSet<Capability>,
    children: Vec<DeviceIndex>,
    sysfs: Option<PathBuf>,
}

impl PhysicalDevice {
    pub fn new() -> Self {
        Self {
            node: Node::new(),
            abstract_types: Vec::new(),
            caps: HashSet::new(),
            children: Vec::new(),
            sysfs: None,
        }
    }

    /// Return true if the given other device is a child of this device or false otherwise
    pub(crate) fn match_device(&mut self, other: &AttachedDevice) -> bool {
        if self.sysfs.is_none() {
            return false;
        }
        match other {
            AttachedDevice::Evdev(evdev) => {
                evdev.sysfs_path().starts_with(self.sysfs.as_ref().unwrap())
            }
            _ => false,
        }
    }

    /// Returns the physical type of this device. Unlike [`Device::capabilities`]
    /// a device is only of one physical type even where it supports multiple different
    /// input methods.
    ///
    /// Over time, the number of abstract device types will be expanded to encompass more
    /// devices and previously assigned device types may become obsolete or not specific enough.
    /// For this reason this function returns a vector of types, with the most recently added
    /// abstract type first. A caller is expected to iterate
    /// through this vector matching against each element until the first element they know.
    pub fn abstract_types(&self) -> Vec<AbstractType> {
        self.abstract_types.clone()
    }

    pub(crate) fn index(&self) -> DeviceIndex {
        self.node.idx.clone()
    }

    /// Reduce our capabilities to one abstract type.
    fn calculate_abstract_type(&mut self) -> AbstractType {
        self.caps.iter().fold(AbstractType::Switch, |at, c| {
            match c {
                // A lot of keyboard-like devices also have a switch, so we only
                // use the switch type for something that's *just* a switch
                Capability::Switch => at,
                // We only override to keyboard if we have a switch, otherwise
                // we keep whatever we have.
                Capability::Keyboard => {
                    match at {
                        AbstractType::Switch => AbstractType::Keyboard,
                        _ => at,
                    }
                },
                Capability::Pointer => {
                    // If it's a keyboard and has pointer caps, it's probably a pointer.
                    // Otherwise if it's anything more sophisticated, stick with what we have
                    match at {
                        AbstractType::Keyboard => at,
                        _ => AbstractType::Pointer,
                    }
                }
                // The ones below are very specific, if we have those set
                // that's probably the device we have
                Capability::Pointingstick => AbstractType::Pointer,
                Capability::Touchpad => AbstractType::Pointer,
                Capability::Clickpad => AbstractType::Pointer,
                Capability::Pressurepad => AbstractType::Pointer,
                Capability::Touchscreen => AbstractType::Touchscreen,
                Capability::Trackball => AbstractType::Pointer,
                Capability::Joystick => AbstractType::GamingDevice,
                Capability::Gamepad => AbstractType::GamingDevice,
                Capability::Tablet => AbstractType::Tablet,
                Capability::TabletScreen => AbstractType::Tablet,
                Capability::TabletExternal => AbstractType::Tablet,
                Capability::TabletPad => AbstractType::Tablet,
            }
        })
    }

    pub(crate) fn add_child(&mut self, child: &AttachedDevice) {
        match child {
            AttachedDevice::Evdev(device) => {
                self.children.push(device.index());
                self.set_syspath(child);
                for cap in device.capabilities().iter() {
                    self.caps.insert(*cap);
                }
            }
            AttachedDevice::Parent(device) => {
                panic!("Cannot attach a parent to a parent");
            }
        }

        // Now let's see if we can calculate our abstract type
        let atype = self.calculate_abstract_type();
        self.abstract_types.push(atype);
    }

    fn set_syspath(&mut self, child: &AttachedDevice) {
        if self.sysfs.is_some() {
            return;
        }

        let evdev = match child {
            AttachedDevice::Evdev(ref device) => device,
            _ => {
                panic!("Not implemented");
            }
        };
        let device =
            udev::Device::from_syspath(evdev.sysfs_path()).expect("Unable to find udev device");
        let syspath: Option<PathBuf> = loop {
            let parent = device.parent();
            if parent.is_none() {
                break None;
            }
            let parent = parent.unwrap();
            match parent.subsystem() {
                Some(str) if str == "input" => {
                    // we go up one from input to find the real device
                    let grandparent = parent.parent().or(Some(parent)).unwrap();
                    break Some(grandparent.syspath().to_owned());
                }
                _ => {},
            };
        };

        self.sysfs = syspath;
    }

    /// Returns an iterator over all children of this parent device
    pub fn iter(&self) -> impl Iterator<Item=&DeviceIndex> + '_ {
        self.children.iter()
    }
}

impl HasCapability for PhysicalDevice {
    fn capabilities(&self) -> Vec<Capability> {
        self.caps.iter().map(|c| c.clone()).collect()
    }
}
