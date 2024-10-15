use std::fs::File;
use std::io::{self, Read};
use std::mem;
use std::os::raw::{c_uint, c_ulong, c_ushort};

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

fn main() -> io::Result<()> {
    let mut file = File::open("/dev/input/event0")?;
    let mut buffer = [0u8; mem::size_of::<InputEvent>()];

    loop {
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                let input_event: InputEvent =
                    unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
                println!("{:?}", input_event);
            }
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                eprintln!("[!] Error while reading: {:?}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}
