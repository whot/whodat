use clap::{arg, command, Parser, Subcommand};
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::os::fd::OwnedFd;
use whodat::{AttachedDevice, EvdevDevice, HasCapability, HasParent, PhysicalDevice};

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // shows information about a given device.
    Show { path: std::path::PathBuf },
    Tree { paths: Vec<std::path::PathBuf> },
}

fn print_evdev(device: &EvdevDevice, prefix: &str) {
    println!("{prefix}- evdev:");
    println!("{prefix}    name: {}", device.name());
    println!("{prefix}    id: {:04x}:{:04x}", device.vid(), device.pid());
    println!("{prefix}    udev: {:?}", device.udev_types());
    println!("{prefix}    capabilities:");
    for c in device.capabilities().into_iter() {
        println!("{prefix}    - {c:?}");
    }
}

fn print_parent(parent: &PhysicalDevice, prefix: &str) {
    let atypes = parent.abstract_types();
    let atype = atypes.first().unwrap();
    println!("{prefix}- parent:");
    println!("{prefix}    type: {atype:?}");
    println!("{prefix}    capabilities:");
    for c in parent.capabilities().into_iter() {
        println!("{prefix}    - {c:?}");
    }
}

fn show_evdev(path: &std::path::PathBuf) -> Result<(), Box<dyn Error>> {
    assert!(path.starts_with("/dev/input"));
    let f = File::open(path)?;

    let mut tree = whodat::DeviceTree::new();
    let idx = tree.attach_evdev(OwnedFd::from(f))?;
    let device = tree.get_device(&idx).unwrap();
    match device {
        AttachedDevice::Evdev(device) => {
            println!("For evdev device {path:?}:");
            print_evdev(&device, "");

            let pidx = device.parent();
            let parent = tree
                .get_parent_device(&pidx)
                .expect(format!("Bug: no parent for device {:?}", &device).as_str());
            print_parent(&parent, "");
        }
        _ => {}
    }

    Ok(())
}

fn show_hidraw(path: &std::path::PathBuf) -> Result<(), Box<dyn Error>> {
    assert!(path.starts_with("/dev/hidraw"));
    let _fd = File::open(path)?;

    Ok(())
}

fn show(path: &std::path::PathBuf) -> Result<(), Box<dyn Error>> {
    let cpath = std::fs::canonicalize(path)?;
    let devnode = cpath.as_os_str().to_str().unwrap();
    if devnode.starts_with("/dev/input/") {
        show_evdev(path)?
    } else if devnode.starts_with("/dev/hidraw") {
        show_hidraw(path)?
    } else {
        panic!("Support for path {:?} is not implemented", path);
    }
    Ok(())
}

fn tree(paths: &Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    let mut tree = whodat::DeviceTree::new();

    for path in paths {
        let cpath = std::fs::canonicalize(path)?;
        let f = File::open(path)?;
        let devnode = cpath.as_os_str().to_str().unwrap();
        if devnode.starts_with("/dev/input/") {
            tree.attach_evdev(OwnedFd::from(f))?;
        } else {
            panic!("Support for path {:?} is not implemented", path);
        }
    }

    for node in tree.iter() {
        match node {
            AttachedDevice::Parent(parent) => {
                print_parent(&parent, "");
                println!("    children:");
                for child in parent.iter() {
                    match tree.get_device(child).expect("Device disappeared?") {
                        AttachedDevice::Evdev(evdev) => {
                            print_evdev(&evdev, "    ");
                        },
                        _ => {},
                    }
                }

            },
            _ => {},
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Show { path } => show(&path)?,
        Commands::Tree { paths } => tree(paths)?,
    }

    Ok(())
}
