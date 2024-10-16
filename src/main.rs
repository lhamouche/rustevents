use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read, Write};
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

fn c_uint_to_key_direction(num: c_uint) -> Option<xkb::KeyDirection> {
    match num {
        0 => Some(xkb::KeyDirection::Up),
        1 => Some(xkb::KeyDirection::Down),
        _ => None,
    }
}

fn read_input_event(file: &mut File, buffer: &mut [u8]) -> io::Result<InputEvent> {
    file.read_exact(buffer)?;
    Ok(unsafe { std::ptr::read(buffer.as_ptr() as *const _) })
}

fn handle_event(
    input_event: &InputEvent,
    state: &mut xkb::State,
    compose_state: &mut xkb::compose::State,
) {
    if input_event.type_ == 0x01 {
        let key_code = xkb::Keycode::new(input_event.code as u32 + 8);
        if let Some(key_direction) = c_uint_to_key_direction(input_event.value) {
            state.update_key(key_code, key_direction);
        }

        let mut utf8_char: Option<String> = None;

        if input_event.value == 1 {
            let keysym = state.key_get_one_sym(key_code);
            compose_state.feed(keysym);

            match compose_state.status() {
                xkb::compose::Status::Composed => {
                    utf8_char = compose_state.utf8();
                    compose_state.reset();
                }
                xkb::compose::Status::Nothing => {
                    let c = state.key_get_utf8(key_code);
                    utf8_char = Some(if c == "\r" { "\n".to_string() } else { c });
                }
                xkb::compose::Status::Cancelled => {
                    compose_state.reset();
                }
                _ => {}
            }
        }

        // println!("{:?} {:?}", input_event, utf8_char.as_deref().unwrap_or(""));
        print!("{}", utf8_char.as_deref().unwrap_or(""));
        std::io::stdout().flush().unwrap();
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
    let mut state = xkb::State::new(&keymap);
    let compose_table = xkb::compose::Table::new_from_locale(
        &context,
        OsStr::new("fr.UTF-8"),
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    )
    .unwrap();
    let mut compose_state = xkb::compose::State::new(&compose_table, xkb::STATE_NO_FLAGS);

    loop {
        match read_input_event(&mut file, &mut buffer) {
            Ok(input_event) => handle_event(&input_event, &mut state, &mut compose_state),
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("[!] Error while reading: {:?}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}
