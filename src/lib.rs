//! This library provides methods to change settings on the Skiller Pro+ keyboard
//!
//! To get started, initialize the SkillerProPlus struct using the `new` method.
//!
//! To use the types in this libary with clap, activate the `clap` feature.
//!
//! ## Note
//! Since i only have one of these keyboard available to test,
//! i won't support selecting multiple devices for now, as i
//! have no way to test this feature.

use rusb::{request_type, Context, DeviceHandle, Direction, Recipient, RequestType, UsbContext};
use std::time::Duration;

const INTERFACE: u8 = 1;
const VID: u16 = 0x04d9;
const PID: u16 = 0xa096;

trait ToSkillerBytes {
    fn to_skiller_bytes(&self) -> u8;
}

/// A struct that can interact with the skiller pro plus keyboard
///
/// # Example
/// ```
/// use libskiller::{SkillerProPlus, Brightness, Color, Profile};
/// use std::time::Duration;
///
/// let skiller = SkillerProPlus::new(Duration::from_secs(2))
///     .unwrap() // unwrap() possible libusb errors, leaves Option<SkillerProPlus>
///     .unwrap(); // If this is none, it means that no keyboard was found
///
/// skiller
///     .set_color(Color::Red, Profile::P2)
///     .unwrap();
///
/// skiller
///     .set_brightness(Brightness::Pulsating {color: Color::Blue}, Profile::P3)
///     .unwrap();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct SkillerProPlus {
    handle: DeviceHandle<rusb::Context>,
    timeout: Duration,
}

/// Represents the different LED colors of the keyboard
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Color {
    Red,
    Green,
    Blue,
    Purple,
    Cyan,
    Yellow,
    White,
}

impl ToSkillerBytes for Color {
    fn to_skiller_bytes(&self) -> u8 {
        match self {
            Color::Red => 0,
            Color::Green => 1,
            Color::Blue => 2,
            Color::Purple => 3,
            Color::Cyan => 4,
            Color::Yellow => 5,
            Color::White => 6,
        }
    }
}

/// Represents one of the three profiles of the keyboard
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Profile {
    P1,
    P2,
    P3,
}

impl ToSkillerBytes for Profile {
    fn to_skiller_bytes(&self) -> u8 {
        match self {
            Profile::P1 => 1,
            Profile::P2 => 2,
            Profile::P3 => 3,
        }
    }
}

/// Represents one of the possible brightness settings of the keyboard
#[cfg_attr(feature = "clap", derive(clap::Subcommand))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Brightness {
    /// A static color at the given brightness
    Static { level: u8, color: Color },

    /// A color pulsating
    Pulsating { color: Color },

    /// All colors pulsatinng in a cycle
    Cycle,
}

/// Represents one of four valid polling rates of the keyboard
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum PollingRate {
    HZ125,
    HZ250,
    HZ500,
    HZ1000,
}

impl ToSkillerBytes for PollingRate {
    fn to_skiller_bytes(&self) -> u8 {
        match self {
            PollingRate::HZ125 => 8,
            PollingRate::HZ250 => 4,
            PollingRate::HZ500 => 2,
            PollingRate::HZ1000 => 1,
        }
    }
}

impl ToSkillerBytes for bool {
    fn to_skiller_bytes(&self) -> u8 {
        match self {
            true => 0,
            false => 1,
        }
    }
}

impl SkillerProPlus {
    /// Creates a new SkillerProPlus struct with the given timeout.
    /// The returned result will be an error if any libusb operations failed,
    /// The nested Option indicates if the device is present on the USB bus
    ///
    /// ## Parameters
    /// `timeout` specifies the usb timeout that is passed to libusb.
    /// A sane value for this would be something like 2 seconds.
    pub fn new(timeout: Duration) -> rusb::Result<Option<Self>> {
        let context = Context::new()?;
        let devices = context.devices()?;

        for device in devices.iter() {
            let device_desc = device.device_descriptor()?;

            if device_desc.vendor_id() != VID || device_desc.product_id() != PID {
                continue;
            }

            let mut handle = device.open()?;

            // Detach the kernel driver if it is active.
            // Not doing this causes libusb to return an IO error
            if handle.kernel_driver_active(INTERFACE)? {
                handle.detach_kernel_driver(INTERFACE)?;
            }

            return Ok(Some(SkillerProPlus {
                handle: handle,
                timeout,
            }));
        }
        return Ok(None);
    }

    /// Sets the color of the keyboard for the given profile
    /// Returns the amount of bytes written or any libusb errors
    pub fn set_color(&self, color: Color, profile: Profile) -> rusb::Result<usize> {
        let p = profile.to_skiller_bytes();

        let payload: [u8; 8] = [
            0x07,
            0x0a,
            p,
            0x0a,
            0x04,
            0x00,
            color.to_skiller_bytes(),
            0x00,
        ];

        let mut total_written = 0;

        total_written += self.skiller_write(&switch_profile(p))?;
        total_written += self.skiller_write(&payload)?;

        return Ok(total_written);
    }

    /// Sets the profile of the keyboard
    ///
    /// Returns the amount of bytes written or any error returned by libusb
    pub fn set_profile(&self, profile: Profile) -> rusb::Result<usize> {
        Ok(self.skiller_write(&switch_profile(profile.to_skiller_bytes()))?)
    }

    /// Sets the brightness and color for the given profile.
    /// Returns the amount of bytes written or any error returned by libusb
    ///
    /// ## Note
    /// You have to provide the color as well because the keyboards API is weird
    pub fn set_brightness(&self, brightness: Brightness, profile: Profile) -> rusb::Result<usize> {
        let p = profile.to_skiller_bytes();

        let payload: [u8; 8] = match brightness {
            Brightness::Static { level, color } => [
                0x07,
                0x0a,
                p,
                level,
                0x04,
                0x00,
                color.to_skiller_bytes(),
                0x00,
            ],
            Brightness::Pulsating { color } => [
                0x07,
                0x0a,
                p,
                11,
                0x04,
                0x00,
                color.to_skiller_bytes(),
                0x00,
            ],
            Brightness::Cycle => [0x07, 0x0a, p, 12, 0x04, 0x00, 0x00, 0x00],
        };

        let mut total_written = 0;

        total_written += self.skiller_write(&switch_profile(p))?;
        total_written += self.skiller_write(&payload)?;

        return Ok(total_written);
    }

    /// Sets the global polling rate of the keyboard.
    ///
    /// Returns the amount of bytes written or any error returned by libusb.
    pub fn set_polling_rate(&self, rate: PollingRate) -> rusb::Result<usize> {
        Ok(self.skiller_write(&[
            0x07,
            0x01,
            rate.to_skiller_bytes(),
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ])?)
    }

    /// Sets the windows key to be enabled or disabled
    ///
    /// Returns the amount of bytes written or any error returned by libusb
    pub fn set_win_key(&self, enable: bool, profile: Profile) -> rusb::Result<usize> {
        let p = profile.to_skiller_bytes();
        let e = enable.to_skiller_bytes();

        let payload = [0x07, 0x0b, p, e, 0x00, 0x00, 0x00, 0x00];

        Ok(self.skiller_write(&payload)?)
    }

    /// Writes data to the keyboard
    ///
    /// Returns the amount of bytes written or any error returned by libusb
    fn skiller_write(&self, data: &[u8; 8]) -> rusb::Result<usize> {
        let rt = request_type(Direction::Out, RequestType::Class, Recipient::Interface);

        let written = self
            .handle
            .write_control(rt, 9, 0x0307, 1, data, self.timeout)?;
        return Ok(written);
    }
}

fn switch_profile(profile: u8) -> [u8; 8] {
    [0x07, 0x02, profile, 0x00, 0x00, 0x00, 0x00, 0x00]
}

// fn begin_handshake_profile(profile: u8) -> [u8; 8] {
//     [0x07, 0x0b, profile, 0x00, 0x00, 0x00, 0x00, 0x00]
// }

// fn footer() -> [u8; 8] {
//     let mut cmd = [0; 8];
//     cmd[0] = 0x07;
//     cmd[1] = 0x02;
//     cmd[2] = 0x04;
//     return cmd;
// }
