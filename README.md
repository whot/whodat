whodat
======

`whodat` is an input device classification database/library/something/other.

**This will most likely be a DBus service that runs outside the sandboxes**

The idea is that given *some* information about a device we can query `whodat`
to tell us something more about the device with little effort and, more importantly,
**unified across implementations**. For example, given an evdev or hidraw file
descriptor, we may query whether the device is a mouse or a joystick.

`whodat` differs between the **physical device** and the **kernel device**. The physical
device represents what the device would typically be interpreted as by the
user, e.g. a PlayStation controller is a gaming device. The kernel device is what the
individual file descriptor refers to and it may be a subset of what the device actually is,
e.g. the PlayStation controllers have an integrated touchpad. The kernel device
works with capabilities which are
one or more tags of events that the device can produce. For example
- mice and touchpads have a [`Capability::Pointer`] capability but their physical type differs,
- joysticks and racing wheels may have similar capabilities but have different physical types,
- pointing sticks and mice have different physical types but usually the same capabilities.

The goal of `whodat` is to be the generic lookup table used by compositors
and clients so that both sides always agree on what capabilities a
device have. Having this as part of a library/daemon means we only have a
single place to maintain this information.

## Implementation as DBus service

If implemented as DBus service, `whodat` can provide full device identification
even for unprivileged processes. Such a process would get an fd to the (evdev,
hidraw,...) device via some other channel (wayland, inputfd, portal, ...) an pass
that fd to the `whodat` DBus service which can identify the actual device using
`stat(3)`. It can then gather information from udev, ioctls, etc. and provide
that to the process.

In pseudo-code:

```python
import whodat

fd: int = obtain_evdev_fd_from_somewhere()
objpath: string = whodat.DeviceFromEvdev(fd)
device = whodat.Device.from_objpath(objpath)

if device.has_capability(whodat.Capability.Touchpad):
    print("This is a touchpad")
```

Notably: the process itself needs no access to the device beyond what it already
has.

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
