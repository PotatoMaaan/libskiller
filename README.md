# libskiller

A rust library to control the skiller pro+ keyboard.

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

For a more complete documentation experience, see [Documentation](###Documentation)

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
