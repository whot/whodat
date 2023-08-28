use crate::*;

use std::path::PathBuf;

/// The [`EvdevDevice`] struct represents a single kernel device and
/// the queryable information about this device.
#[derive(Debug)]
pub struct EvdevDevice {
    /// Attachment in the [`DeviceTree`]
    node: Node,
    name: String,
    vid: u16,
    pid: u16,
    udev_properties: Vec<String>,
    capabilities: Vec<Capability>,
    devnode: Option<PathBuf>,
    sysfs: PathBuf,
}

impl HasParent for EvdevDevice {
    fn parent(&self) -> DeviceIndex {
        match self.node.parent {
            Some(index) => index.clone(),
            None => {
                panic!("No parent set for {:?}, missing set_parent() call", self);
            }
        }
    }
}

impl HasCapability for EvdevDevice {
    fn capabilities(&self) -> Vec<Capability> {
        self.capabilities.clone()
    }
}

impl<'a> EvdevDevice {
    /// Return a new [`EvdevDevice`] based on the device that the fd points to.
    /// The fd must be ready for `ioctl()` no data is read or written on this fd.
    pub fn from_fd(fd: OwnedFd) -> Result<EvdevDevice, Box<dyn Error>> {
        // Get st_rdev from the fd so we can later look this up with udev
        let f = File::from(fd);
        let meta = f.metadata()?;
        let rdev = meta.st_rdev();

        // Now fetch out the udev properties
        let udev_properties: Vec<String> = Vec::new();
        let mut e = udev::Enumerator::new()?;
        e.match_subsystem("input")?;
        let mut devices = e.scan_devices()?;
        let device: Option<udev::Device> = devices.find_map(|d| match &d.devnum() {
            Some(num) if *num == rdev => Some(d),
            _ => None,
        });

        // FIXME: can happen if device was removed since
        let device = device.expect("Unable to find udev devnode");

        let udev_properties = util::input_id_udev_props(&device);
        let devnode = device.devnode().map(|n| n.clone().to_owned());
        let sysfs = device.syspath().to_path_buf();

        // Map udev to capabilities, then fill in any potentially missing ones
        let capabilities: Vec<Capability> = udev_properties
            .iter()
            .filter(|prop| Capability::from_udev_prop(&prop).is_some())
            .map(|prop| Capability::from_udev_prop(&prop).unwrap())
            .collect();
        let capabilities = Capability::extend(capabilities);

        let fd = OwnedFd::from(f);
        let device = evdev::Device::from_fd(fd)?;
        let ids = device.input_id();

        let device_index = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let device = Self {
            node: Node::new(),
            name: device.name().unwrap().to_string(),
            vid: ids.vendor(),
            pid: ids.product(),
            udev_properties,
            capabilities,
            devnode,
            sysfs,
        };

        Ok(device)
    }

    /// Return the device's name as advertised by the kernel. For many
    /// HID devices, this name will have a HID-application specific
    /// suffix like "Pen", "Mouse", "Consumer Control".
    pub fn name(&'a self) -> &'a str {
        &self.name
    }

    /// Return the udev `"ID_INPUT_*"` udev properties that are set to a nonzero value
    /// for this device. If the result is an empty vector, no such properties are set.
    pub fn udev_types(&'a self) -> &'a Vec<String> {
        &self.udev_properties
    }

    /// The 16-bit Vendor ID
    pub fn vid(&self) -> u16 {
        self.vid
    }

    /// The 16-bit Product ID
    pub fn pid(&self) -> u16 {
        self.pid
    }

    pub fn devnode(&self) -> &Option<PathBuf> {
        &self.devnode
    }

    pub fn sysfs_path(&self) -> &PathBuf {
        &self.sysfs
    }

    pub(crate) fn set_parent(&mut self, parent: &PhysicalDevice) {
        //if let Some(ref mut node) = self.node {
        self.node.set_parent(parent.index());
        //}
    }

    pub(crate) fn index(&self) -> DeviceIndex {
        self.node.idx.clone()
    }
}
