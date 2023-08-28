#![doc = include_str!("../../README.md")]
#![allow(unused_variables, dead_code)]

use evdev;
use std::{
    cmp::PartialEq,
    collections::HashMap,
    error::Error,
    fs::File,
    hash::{Hash, Hasher},
    os::fd::OwnedFd,
    os::linux::fs::MetadataExt,
    sync::atomic::{AtomicU32, Ordering},
};
use udev;

mod evdev_device;
mod hidraw_device;
mod physical_device;
mod types;
mod util;

pub use evdev_device::EvdevDevice;
pub use hidraw_device::HidrawDevice;
pub use physical_device::PhysicalDevice;
pub use types::{AbstractType, Capability};

// Next device id, see [`DeviceIndex::next`]
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

/// The generic return type for [`DeviceTree::get_device`].
#[non_exhaustive]
#[derive(Debug)]
pub enum AttachedDevice {
    Evdev(EvdevDevice),
    Parent(PhysicalDevice),
}

impl AttachedDevice {
    fn set_parent(&mut self, parent: &PhysicalDevice) {
        match self {
            AttachedDevice::Evdev(evdev) => {
                evdev.set_parent(parent);
            }
            AttachedDevice::Parent(_) => {
                panic!("Cannot set a parent to a parent");
            }
        }
    }
}

/// A unique device index to fetch a device from a [`DeviceTree`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceIndex {
    idx: u32,
}

impl DeviceIndex {
    /// Returns the next available device index.
    fn next() -> DeviceIndex {
        DeviceIndex {
            idx: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl Hash for DeviceIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.idx.hash(state);
    }
}

/// A node in the [`DeviceTree`].
#[derive(Clone, Copy, Debug)]
struct Node {
    idx: DeviceIndex,
    /// Points to the parent device, if any. `None` for [`PhysicalDevice`].
    parent: Option<DeviceIndex>,
}

impl Node {
    fn new() -> Self {
        Self {
            idx: DeviceIndex::next(),
            parent: None,
        }
    }

    fn index(&self) -> DeviceIndex {
        self.idx.clone()
    }

    pub(crate) fn set_parent(&mut self, parent: DeviceIndex) {
        self.parent = Some(parent);
    }
}

/// The [`DeviceTree`] is the context object that holds all currently
/// known devices. Devices can be **attached** to the tree,
/// see [`DeviceTree::attach_evdev`] and are then available via
/// [`DeviceTree::get_device`].
///
/// A device tree builds up information about devices based on the
/// devices it gets given - where a caller has access to more than one
/// device it should attach all devices before obtaining information
/// about any one of those devices.
#[derive(Debug)]
pub struct DeviceTree {
    devices: HashMap<DeviceIndex, AttachedDevice>,
}

impl DeviceTree {
    /// Create a new tree with no devices attached.
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Attach a new evdev device from an open evdev file descriptor that can be
    /// `ioctl`'d for information. The returned [`DeviceIndex`] can be used to
    /// obtain the actual [`EvdevDevice`] later, see [`DeviceTree::get_device`]
    ///
    /// Where a caller has multiple file descriptors they should be added first
    /// before calling [`DeviceTree::get_device`] to ensure the resulting device
    /// is built from the maximum information. Likewise, attaching more devices *may*
    /// change the information about an already attached device.
    pub fn attach_evdev(&mut self, fd: OwnedFd) -> Result<DeviceIndex, Box<dyn Error>> {
        let evdev = EvdevDevice::from_fd(fd)?;
        let index = evdev.index();
        let mut attached = AttachedDevice::Evdev(evdev);

        let parent: Option<&mut PhysicalDevice> = self.devices.values_mut().find_map(|d| match d {
            AttachedDevice::Parent(parent) => {
                if parent.match_device(&attached) {
                    Some(parent)
                } else {
                    None
                }
            }
            _ => None,
        });

        match parent {
            Some(parent) => {
                parent.add_child(&attached);
                attached.set_parent(&parent);
            }
            None => {
                let mut parent = PhysicalDevice::new();
                let pindex = parent.index();
                parent.add_child(&attached);
                attached.set_parent(&parent);
                self.devices
                    .insert(pindex.clone(), AttachedDevice::Parent(parent));
            }
        }

        self.devices.insert(index.clone(), attached);

        //println!("Hashmap is {:?}", self.devices);

        Ok(index)
    }

    /// Given the [`DeviceIndex`] returned by [`DeviceTree::attach_evdev`] return
    /// that device. This is the generic version that returns the device
    /// and punts device type detection to the caller. For more specific versions
    /// see [`DeviceTree::get_evdev_device`] and [`DeviceTree::get_parent_device`].
    pub fn get_device(&self, idx: &DeviceIndex) -> Option<&AttachedDevice> {
        self.devices.get(idx).and_then(|x| Some(x))
    }

    /// Given the [`DeviceIndex`] returned by [`DeviceTree::attach_evdev`] return
    /// that device if it is indeed an [`EvdevDevice`].
    pub fn get_evdev_device(&self, idx: &DeviceIndex) -> Option<&EvdevDevice> {
        let d = self.devices.get(idx)?;
        match &d {
            AttachedDevice::Evdev(evdev) => Some(&evdev),
            _ => None,
        }
    }

    /// Given the [`DeviceIndex`] returned by [`DeviceTree::attach_evdev`] return
    /// that device if it is indeed a [`PhysicalDevice`].
    pub fn get_parent_device(&self, idx: &DeviceIndex) -> Option<&PhysicalDevice> {
        let d = self.devices.get(idx)?;
        match &d {
            AttachedDevice::Parent(parent) => Some(&parent),
            _ => None,
        }
    }

    /// Returns an iterator over all [`AttachedDevice`]s that are part of this tree.
    pub fn iter(&self) -> impl Iterator<Item=&AttachedDevice> + '_ {
        self.devices.values()
    }
}

/// The [`HasParent`] trait is implemented by devices that have a single parent
/// device that represents the [`PhysicalDevice`]. See [`HidrawDevice`] and [`EvdevDevice`] for implementations
/// of this trait.
pub trait HasParent {
    /// Return the parent [`DeviceIndex`] of this kernel device - use
    /// with [`DeviceTree::get_device`] to fetch the parent device.
    fn parent(&self) -> DeviceIndex;
}

pub trait HasCapability {
    /// Return the set of capabilities of this device.
    fn capabilities(&self) -> Vec<Capability>;
}
