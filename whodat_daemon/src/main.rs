use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::sync::Arc;
use zbus::zvariant::ObjectPath;
use zbus::{dbus_interface, ConnectionBuilder, ObjectServer, Result};

const PATH_BASE: &'static str = "/org/freedesktop/whodat";
const VERSION: u32 = 1;

// FIXME: this will eventually be a whodat.Device
struct InnerDevice {
    name: String,
}

struct Device {
    inner: Arc<InnerDevice>,
    objpath: String,
    parent_objpath: String,
}

// FIXME: this will eventually be a whodat.Device or something
struct PhysicalDevice {
    inner: Arc<InnerDevice>,
    objpath: String,
}

struct Daemon {
    counter: u32,
    devices: HashMap<String, Arc<InnerDevice>>,
}

#[dbus_interface(name = "org.freedesktop.Whodat.Device")]
impl Device {
    #[dbus_interface(property)]
    async fn version(&self) -> u32 {
        VERSION
    }

    #[dbus_interface(property)]
    async fn name(&self) -> &String {
        &self.inner.name
    }

    #[dbus_interface(property)]
    async fn parent(&self) -> ObjectPath {
        ObjectPath::try_from(self.parent_objpath.clone()).unwrap()
    }
}

#[dbus_interface(name = "org.freedesktop.Whodat.Device")]
impl PhysicalDevice {
    #[dbus_interface(property)]
    async fn version(&self) -> u32 {
        VERSION
    }
}

#[dbus_interface(name = "org.freedesktop.Whodat")]
impl Daemon {
    #[dbus_interface(property)]
    async fn version(&self) -> u32 {
        VERSION
    }

    /// Creates a new whodat.Device given an evdev file descriptor and
    /// returns the object path for that device.
    ///
    /// This device represents the evdev device itself, the physical device
    /// is the parent.
    async fn device_from_evdev(
        &mut self,
        #[zbus(object_server)] object_server: &ObjectServer,
        fd: RawFd,
    ) -> ObjectPath {
        self.counter += 1;
        let path = format!("{PATH_BASE}/e/{}", self.counter);

        let inner = Arc::new(InnerDevice {
            name: String::from("evdev device"),
        });

        let parent_path = format!("{PATH_BASE}/p/{}", self.counter);
        let parent = PhysicalDevice {
            inner: inner.clone(), // FIXME: needs to be its own device obviously
            objpath: parent_path.clone(),
        };

        let parent_objpath = ObjectPath::try_from(parent.objpath.clone()).unwrap();
        let _ = object_server.at(&parent_objpath, parent).await;

        let device = Device {
            inner: inner.clone(),
            objpath: path.clone(),
            parent_objpath: parent_path,
        };

        let objpath = ObjectPath::try_from(device.objpath.clone()).unwrap();
        let _ = object_server.at(&objpath, device).await;

        self.devices.insert(path, inner);

        objpath
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    let daemon = Daemon {
        counter: 0,
        devices: HashMap::new(),
    };

    let _connection = ConnectionBuilder::session()?
        .name("org.freedesktop.Whodat")?
        .serve_at(PATH_BASE, daemon)?
        .build()
        .await?;

    loop {
        std::future::pending::<()>().await;
    }
}
