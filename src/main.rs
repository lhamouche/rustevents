use clap::Parser;
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file: String,

    #[arg(short, long, default_value = "evdev")]
    rules: String,

    #[arg(short, long, default_value = "pc105")]
    model: String,

    #[arg(short, long, default_value = "us")]
    layout: String,

    #[arg(short, long, default_value = "")]
    variant: String,

    #[arg(short = 'c', long, default_value = "en_US.UTF-8")]
    locale: String,

    #[arg(short, long)]
    no_numlock: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut file = File::open(args.file)?;
    let mut buffer = [0u8; mem::size_of::<InputEvent>()];

    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let keymap = xkb::Keymap::new_from_names(
        &context,
        &args.rules,
        &args.model,
        &args.layout,
        &args.variant,
        None,
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    )
    .expect("[!] Failed to create keymap.");

    let mut state = xkb::State::new(&keymap);
    if !args.no_numlock {
        state.update_key(xkb::Keycode::new(69 as u32 + 8), xkb::KeyDirection::Down);
    }

    let compose_table = xkb::compose::Table::new_from_locale(
        &context,
        OsStr::new(&args.locale),
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    )
    .expect("[!] Failed to create compose table.");

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
