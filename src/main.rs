use std::fs::File;
use std::io::{self, Read};
use std::mem;
use std::os::raw::{c_uint, c_ulong, c_ushort};
use xkbcommon::xkb;

#[repr(C)]
#[derive(Debug)]
struct Timeval {
    tv_sec: c_ulong,
    tv_usec: c_ulong,
}

#[repr(C)]
#[derive(Debug)]
struct InputEvent {
    time: Timeval,
    type_: c_ushort,
    code: c_ushort,
    value: c_uint,
}

fn read_input_event(file: &mut File, buffer: &mut [u8]) -> io::Result<InputEvent> {
    file.read_exact(buffer)?;
    Ok(unsafe { std::ptr::read(buffer.as_ptr() as *const _) })
}

fn handle_event(input_event: &InputEvent, state: &xkb::State) {
    if input_event.type_ == 0x01 {
        let key_code = xkb::Keycode::new(input_event.code as u32 + 8);
        let utf8 = state.key_get_utf8(key_code);
        println!("{:?} {:?}", input_event, utf8);
    }
}

fn main() -> io::Result<()> {
    let mut file = File::open("/dev/input/event0")?;
    let mut buffer = [0u8; mem::size_of::<InputEvent>()];

    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let keymap = xkb::Keymap::new_from_names(
        &context,
        "evdev",
        "pc105",
        "fr",
        "latin9",
        None,
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    )
    .expect("[!] Failed to create keymap");
    let state = xkb::State::new(&keymap);

    loop {
        match read_input_event(&mut file, &mut buffer) {
            Ok(input_event) => handle_event(&input_event, &state),
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("[!] Error while reading: {:?}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}
