# libskiller

A rust library to control the skiller pro+ keyboard (currently linux only).

> NOTE: This is unofficial software, use it at your own risk!

## Example usage

```rs
use libskiller::{SkillerProPlus, Brightness, Color, Profile};
use std::time::Duration;

let skiller = SkillerProPlus::new(Duration::from_secs(2))
    .unwrap() // unwrap() possible libusb errors, leaves Option<SkillerProPlus>
    .unwrap(); // If this is none, it means that no keyboard was found

skiller
    .set_color(Color::Red, Profile::P2)
    .unwrap();

skiller
    .set_brightness(Brightness::Pulsating {color: Color::Blue}, Profile::P3)
    .unwrap();
```

For a more complete documentation experience, see [Documentation](#Documentation)

## Windows

`libusb` and `rusb` are supposed to work on windows, but i couln't get them to work. libusb keeps returning `NotSupported`, no matter what I do.

## Testing & docs

### Documentation

```sh
cargo doc --open
```

### Running tests

Running the tests requires root privileges and a keyboard to be connected.

```sh
sudo -E cargo test
```
