use evdev_rs::*;
use std::fs::File;
use std::io;

fn print_event(inputevent: &InputEvent) {
    match inputevent.event_code {
        _ => {
            let event_type = inputevent
                .event_type()
                .map(|ev_type| format!("{}", ev_type))
                .unwrap_or("None".to_owned());
            if !(event_type == "EV_MSC" || event_type == "EV_SYN") {
                // Drop EV_MSC && EV_SYN as it is literally useless.
                log::debug!(
                    "type {} , code {} , value {}",
                    event_type,
                    inputevent.event_code,
                    inputevent.value
                )
            }
        }
    }
}

fn print_sync_dropped_event(ev: &InputEvent) {
    print!("SYNC DROPPED: ");
    print_event(ev);
}

fn main() {
    let mut args = std::env::args();

    std::env::set_var("RUST_LOG", "swhkd=trace");
    env_logger::init();

    if args.len() != 2 {
        println!("Usage: evtest /path/to/device");
        std::process::exit(1);
    }

    let path = &args.nth(1).unwrap();
    let f = File::open(path).unwrap();

    let u_d = UninitDevice::new().unwrap();
    let d = u_d.set_file(f).unwrap();

    log::trace!("Input device name: \"{}\"", d.name().unwrap_or(""));

    let mut a: io::Result<(ReadStatus, InputEvent)>;
    loop {
        a = d.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING);
        if a.is_ok() {
            let mut result = a.ok().unwrap();
            match result.0 {
                ReadStatus::Sync => {
                    println!("::::::::::::::::::::: dropped ::::::::::::::::::::::");
                    while result.0 == ReadStatus::Sync {
                        print_sync_dropped_event(&result.1);
                        a = d.next_event(ReadFlag::SYNC);
                        if a.is_ok() {
                            result = a.ok().unwrap();
                        } else {
                            break;
                        }
                    }
                    println!("::::::::::::::::::::: re-synced ::::::::::::::::::::");
                }
                ReadStatus::Success => print_event(&result.1),
            }
        } else {
            let err = a.err().unwrap();
            match err.raw_os_error() {
                Some(libc::EAGAIN) => continue,
                _ => {
                    println!("{}", err);
                    break;
                }
            }
        }
    }
}
