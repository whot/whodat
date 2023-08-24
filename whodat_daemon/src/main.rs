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
}

#[dbus_interface(name = "org.freedesktop.Whodat")]
impl Daemon {
    #[dbus_interface(property)]
    async fn version(&self) -> u32 {
        VERSION
    }

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
        let device = Device {
            inner: inner.clone(),
        };
        let objpath = ObjectPath::try_from(path.clone()).unwrap();
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
