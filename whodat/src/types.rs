/// A high-level category describing a capability on this device.
/// Capabilities are not mutually exclusive (some are, see the documentation for
/// each capability) and any device may match one or more of those capabilities.
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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Capability {
    Keyboard,
    Pointer,
    Pointingstick,
    Touchpad,
    /// A touchpad with a hinge instead of physical, separate buttons. Also called ButtonPads.
    Clickpad,
    /// A touchpad without physical buttons that uses physical pressure to detect button
    /// presses instead of e.g. a mechanical hinge.
    Pressurepad,
    Touchscreen,
    Trackball,
    Joystick,
    Gamepad,
    Tablet,
    /// A tablet built into a screen, e.g. like the Wacom Cintiq series.
    /// This capability is mutually exclusive with the [`Capability::TabletExternal`] capability.
    TabletScreen,
    /// A tablet external to a device, e.g. like the Wacom Intuos series.
    /// This capability is mutually exclusive with the [`Capability::TabletScreen`] capability.
    TabletExternal,
    /// This device is a tablet pad, i.e. the set of buttons, strips and rings that are available
    /// on many [`Capability::Tablet`] devices.
    TabletPad,
    Switch,
}

impl Capability {
    /// Returns the *single* capability that matches to the udev property, if any.
    pub(crate) fn from_udev_prop(name: &str) -> Option<Self> {
        let cap = match name {
            "ID_INPUT_KEY" => Capability::Keyboard,
            "ID_INPUT_KEYBOARD" => Capability::Keyboard,
            "ID_INPUT_MOUSE" => Capability::Pointer,
            "ID_INPUT_TOUCHPAD" => Capability::Touchpad,
            "ID_INPUT_TOUCHSCREEN" => Capability::Touchscreen,
            "ID_INPUT_TRACKBALL" => Capability::Trackball,
            "ID_INPUT_POINTINGSTICK" => Capability::Pointingstick,
            "ID_INPUT_TABLET" => Capability::Tablet,
            "ID_INPUT_TABLET_PAD" => Capability::TabletPad,
            "ID_INPUT_TABLET_JOYSTICK" => Capability::Joystick,
            "ID_INPUT_SWITCH" => Capability::Switch,
            _ => return None,
        };
        Some(cap)
    }

    /// Create a new vector of capabilities that extend the given
    /// capabilities with missing parent capabilities, if any.
    /// For example, any [`Capability::Touchpad`] requires
    /// that [`Capability::Pointer`] is also set - this function will
    /// add that latter capability.
    pub(crate) fn extend(capabilities: Vec<Capability>) -> Vec<Capability> {
        // Most of these will be noops, we expect udev to set these correctly
        let mut caps = Cap::new(capabilities);
        if caps.has(Capability::Pressurepad) {
            caps.set(Capability::Clickpad);
        }
        if caps.has(Capability::Clickpad) || caps.has(Capability::Pressurepad) {
            caps.set(Capability::Touchpad);
        }
        if caps.has(Capability::Touchpad) {
            caps.set(Capability::Pointer);
        }

        // FIXME: need more settings here

        let caps = caps.to_vec();
        caps
    }
}

/// Describes the primary high-level type of this device.
///
/// This is the highest level of categorization and only one of these types
/// applies to each device. Devices may technically fall into multiple categories
/// (e.g. many gaming mice can send key events) but this represents the most obvious
/// category for this device.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum AbstractType {
    /// Device is primarily a keyboard
    Keyboard,
    /// Device is primarily a pointer device, e.g. a mouse, touchpad, or pointingstick
    Pointer,
    /// Device is primarily a touchscreen
    Touchscreen,
    /// Device is primarily a graphics tablet
    Tablet,
    /// Device is primarily a gaming device, e.g. a joystick, gamepad or racing wheel
    GamingDevice,
    /// Device is primarily a switch toggle
    Switch,
}

/// Describes the **physical** type of this device. Unlike the [`Device::has_capability`]
/// a device may only have one physical type. For example, modern PlayStation controllers
/// provide a touchpad as well as a gamepad - the physical type of this controller however
/// is [`AbstractType::GamingDevice`].
///
/// The physical type of the device may not always be known, especially if the device
/// is constructed from a single event node via [`Builder::evdev_fd`]. This crate may
/// rely on an internal database for well-known devices to supplement the information
/// where posssible.
#[non_exhaustive]
#[derive(Debug)]
pub enum DeviceType {
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

/// Internal helper for converting to/from [`Capability`]
struct Cap {
    mask: u32,
}

impl Cap {
    fn new(capabilities: Vec<Capability>) -> Cap {
        let mut mask: u32 = 0;
        for c in capabilities {
            mask |= Cap::as_mask(c);
        }

        Cap { mask }
    }

    fn as_mask(cap: Capability) -> u32 {
        match cap {
            Capability::Keyboard => 1 << 0,
            Capability::Pointer => 1 << 1,
            Capability::Pointingstick => 1 << 2,
            Capability::Touchpad => 1 << 3,
            Capability::Clickpad => 1 << 4,
            Capability::Pressurepad => 1 << 5,
            Capability::Touchscreen => 1 << 6,
            Capability::Trackball => 1 << 7,
            Capability::Joystick => 1 << 8,
            Capability::Gamepad => 1 << 9,
            Capability::Tablet => 1 << 10,
            Capability::TabletScreen => 1 << 11,
            Capability::TabletExternal => 1 << 12,
            Capability::TabletPad => 1 << 13,
            Capability::Switch => 1 << 14,
        }
    }

    fn from_mask(mask: u32) -> Option<Capability> {
        let c: Capability = match mask {
            0b0000000000000001 => Capability::Keyboard,
            0b0000000000000010 => Capability::Pointer,
            0b0000000000000100 => Capability::Pointingstick,
            0b0000000000001000 => Capability::Touchpad,
            0b0000000000010000 => Capability::Clickpad,
            0b0000000000100000 => Capability::Pressurepad,
            0b0000000001000000 => Capability::Touchscreen,
            0b0000000010000000 => Capability::Trackball,
            0b0000000100000000 => Capability::Joystick,
            0b0000001000000000 => Capability::Gamepad,
            0b0000010000000000 => Capability::Tablet,
            0b0000100000000000 => Capability::TabletScreen,
            0b0001000000000000 => Capability::TabletExternal,
            0b0010000000000000 => Capability::TabletPad,
            0b0100000000000000 => Capability::Switch,
            _ => return None,
        };
        Some(c)
    }

    fn set(&mut self, cap: Capability) {
        self.mask |= Cap::as_mask(cap);
    }

    fn has(&self, cap: Capability) -> bool {
        (self.mask & Cap::as_mask(cap)) != 0
    }

    fn to_vec(self) -> Vec<Capability> {
        let mut caps: Vec<Capability> = Vec::new();
        let mut mask: u32 = 1 << 31;
        while mask > 0 {
            if self.mask & mask != 0 {
                caps.push(Cap::from_mask(mask).unwrap());
            }
            mask >>= 1;
        }
        caps
    }
}
