whodat
======

`whodat` is an input device classification database/library/something/other.
The idea is that given *some* information about a device we can query `whodat`
to tell us something more about the device with little effort and, more importantly,
**unified across implementations**. For example, given an evdev or hidraw file
descriptor, we may query whether the device is a mouse or a joystick.

`whodat` differs between the **physical type** and **capabilities**. The physical
type is what the device would typically be sold as. The capabilities are one or
more tags of events that the device can produce. For example
- mice and touchpads have a [`Capability::Pointer`] capability but their physical type differs,
- joysticks and racing wheels may have similar capabilities but have different physical types,
- pointing sticks and mice have different physical types but usually the same capabilities.

The goal of `whodat` is to be the generic lookup table used by compositors
and clients so that both sides always agree on what capabilities a
device have. Having this as part of a library means we only have a
single place to maintain this information.

## Sharing data with unprivileged processes

Two users of `whodat` can share this information. A
privileged process that e.g. has access to udev may create a device,
[`Device::serialize`] that device and pass it to the second process.
That second process, without udev access, can recreate the [`Device`]
without further information and can thus query for the same classifiers
as the first process.

```rust
use whodat::{Builder, Device};
use std::os::fd::{AsRawFd, RawFd};

if let Ok(fd) = std::fs::File::open("/dev/input/event0") {
    if let Ok(device) = Builder::new()
                           .evdev_fd(fd.as_raw_fd())
                           .build() {
        let magic_description = device.serialize().unwrap();
        let other_device = Device::deserialize(magic_description.as_str()).unwrap();
        // other_device is the same as device
    }
}
```
## The device database

`whodat` has an internal database of devices, so some device information
can be looked up merely by the USB IDs. For example, a PlayStation controller
exposes a touchpad event node as well as a gamepad event node. `whodat` knows
that the controller is a gamepad, even where we construct the device from the
touchpad event node. This obviously only works where we have that device in the
database.

```rust
use whodat::{Builder, Capability};
if let Ok(device) = Builder::new()
                    .usbid(0x1234, 0xabcd)
                    .build() {
     match device.physical_type() {
         Some(value) => println!("This device is a {:?}", value),
         None => println!("I really don't know what this device is"),
     }
}
```

Ideally and over time, most commonly used devices will be added to the database,
making `whodat` more reliable in identifying any single device.
