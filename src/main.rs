use std::{
    collections::HashMap,
    env,
    fs::{self, OpenOptions},
    io::{self, Error, ErrorKind, Write},
    net::UdpSocket,
    str,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

fn main() -> std::io::Result<()> {
    // digiVIT address
    let digivit_addr = "192.168.0.145:55556";

    // MD : Monitor distance output
    let message = "MD";

    // Data hashmap
    let mut data_map: HashMap<SeqChar, String> = HashMap::new();

    // Time hashmap
    let mut time_map: HashMap<SeqChar, Duration> = HashMap::new();

    // Sequence counter
    let mut seq_counter = SeqChar::A;

    // UDP socket
    print!(
        "Enter the IP address of the network interface, \
            or press ENTER to bind all network interfaces.\n"
    );
    io::stdout().flush()?;
    let mut my_addr = String::new();
    io::stdin().read_line(&mut my_addr)?;
    my_addr = if my_addr.trim().is_empty() {
        "0.0.0.0:55555".to_string()
    } else {
        my_addr.trim().to_string() + ":55555"
    };

    let socket = UdpSocket::bind(&my_addr).expect("Failed to bind socket");
    socket.set_nonblocking(true)?;
    let socket_sender = Arc::new(Mutex::new(socket));
    let socket_receiver = Arc::clone(&socket_sender);

    // Sample rate
    println!("Enter the sampling rate or press ENTER to use the default value (50Hz).");
    io::stdout().flush()?;
    let mut sample_rate = String::new();
    io::stdin().read_line(&mut sample_rate)?;
    let sample_rate = if sample_rate.trim().is_empty() {
        50.0
    } else {
        sample_rate
            .trim()
            .parse::<f64>()
            .expect("Invalid sample rate")
    };
    let sample_period = Duration::from_secs_f64(1.0 / sample_rate);

    // File
    let filename = "digiVIT_output.txt";
    let full_filename = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join(&filename);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&full_filename)?;

    // Keyboard channel
    let (keyboard_sender, keyboard_receiver) = mpsc::channel();

    // Keyboard thread
    thread::spawn(move || {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        keyboard_sender.send(()).unwrap();
    });

    // Receive channel
    let (data_sender, data_receiver) = mpsc::channel();

    // Receive thread
    let _receive_handle = thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match socket_receiver.lock().unwrap().recv_from(&mut buf) {
                Ok((size, addr)) => {
                    data_sender.send((buf[0..size].to_vec(), addr)).unwrap();
                }
                Err(_) => {}
            }
        }
    });

    println!("Listen address  : {}", &my_addr);
    println!("digiVIT address : {}", &digivit_addr);
    println!("Sampling rate   : {} Hz", &sample_rate);

    println!("\nData acquisition in progress..\nPress ENTER to stop.");
    let start_time = Instant::now();
    let mut sample_time = Instant::now() - sample_period;

    let mut break_requested = false;
    let mut break_seq = SeqChar::A;

    // Main loop
    loop {
        // At sampling time
        if sample_time <= Instant::now() {
            // Send udp message
            socket_sender.lock().unwrap().send_to(
                assemble_packet(&seq_counter, message).as_bytes(),
                digivit_addr,
            )?;
            let _ = time_map.insert(seq_counter, start_time.elapsed());

            // Increase seq_char
            seq_counter = seq_counter.next();

            // File output
            match data_map.remove(&seq_counter) {
                Some(value) => {
                    let data_line = format!(
                        "{:.4}, {}\n",
                        time_map.remove(&seq_counter).unwrap().as_secs_f64(),
                        value
                    );

                    file.write(data_line.as_bytes())?;
                }
                None => match time_map.remove(&seq_counter) {
                    Some(elapsed_time) => {
                        println!("{:.4} sec, missing data", elapsed_time.as_secs_f64());
                    }
                    None => {}
                },
            }

            // Update sample_time for next sampling
            sample_time += sample_period;
        }

        // If there is received data, store it in data_map
        let timeout = Duration::from_millis(1);
        match data_receiver.recv_timeout(timeout) {
            Ok((received, _sender_addr)) => {
                let data_str = String::from_utf8(received).map_err(|e| {
                    Error::new(ErrorKind::Other, format!("Invalid UTF-8 sequence: {}", e))
                })?;
                if !verify_checksum(&data_str) {
                    eprintln!("Checksum error");
                    continue;
                }
                match SeqChar::from_str(&data_str[0..1]) {
                    Some(seq_char) => {
                        // Overwrite if a value for the same key already exists.
                        let _ =
                            data_map.insert(seq_char, data_str[2..data_str.len() - 4].to_owned());
                    }
                    None => eprintln!("Sequence character error"),
                }
            }
            Err(_) => {}
        }

        // Check keyboard input
        if keyboard_receiver.try_recv().is_ok() {
            break_requested = true;
            break_seq = seq_counter.before();
            println!("Closing..");
        }

        // Check program termination
        if break_requested && break_seq == seq_counter {
            break;
        }
    }
    // Close file
    file.flush()?;
    drop(file);

    // Rename file
    print!("\nEnter the file name : ");
    io::stdout().flush()?;
    let mut new_filename = String::new();
    io::stdin().read_line(&mut new_filename)?;
    if !new_filename.trim().is_empty() {
        let new_full_filename = env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(&new_filename);
        fs::rename(&full_filename, &new_full_filename)?;
    }

    Ok(())
}

fn assemble_packet(seq_counter: &SeqChar, payload: &str) -> String {
    let seq_payload = format!("{}{}", seq_counter.char(), payload);
    format!("${}#{}", &seq_payload, calculate_checksum(&seq_payload))
}

fn calculate_checksum(payload: &str) -> String {
    let sum: u8 = payload
        .as_bytes()
        .iter()
        .fold(0, |acc, &x| acc.wrapping_add(x));
    format!("{:02X}", !sum)
}

fn verify_checksum(received: &str) -> bool {
    let len = received.len();
    let checksum = &received[len - 3..len - 1];
    let seq_and_payload = &received[0..len - 4];
    if calculate_checksum(seq_and_payload) == checksum {
        return true;
    } else {
        return false;
    }
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
enum SeqChar {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl SeqChar {
    fn next(&self) -> SeqChar {
        match *self {
            SeqChar::A => SeqChar::B,
            SeqChar::B => SeqChar::C,
            SeqChar::C => SeqChar::D,
            SeqChar::D => SeqChar::E,
            SeqChar::E => SeqChar::F,
            SeqChar::F => SeqChar::G,
            SeqChar::G => SeqChar::H,
            SeqChar::H => SeqChar::I,
            SeqChar::I => SeqChar::J,
            SeqChar::J => SeqChar::K,
            SeqChar::K => SeqChar::L,
            SeqChar::L => SeqChar::M,
            SeqChar::M => SeqChar::N,
            SeqChar::N => SeqChar::O,
            SeqChar::O => SeqChar::P,
            SeqChar::P => SeqChar::Q,
            SeqChar::Q => SeqChar::R,
            SeqChar::R => SeqChar::S,
            SeqChar::S => SeqChar::T,
            SeqChar::T => SeqChar::U,
            SeqChar::U => SeqChar::V,
            SeqChar::V => SeqChar::W,
            SeqChar::W => SeqChar::X,
            SeqChar::X => SeqChar::Y,
            SeqChar::Y => SeqChar::Z,
            SeqChar::Z => SeqChar::A,
        }
    }

    fn before(&self) -> SeqChar {
        match *self {
            SeqChar::A => SeqChar::Z,
            SeqChar::B => SeqChar::A,
            SeqChar::C => SeqChar::B,
            SeqChar::D => SeqChar::C,
            SeqChar::E => SeqChar::D,
            SeqChar::F => SeqChar::E,
            SeqChar::G => SeqChar::F,
            SeqChar::H => SeqChar::G,
            SeqChar::I => SeqChar::H,
            SeqChar::J => SeqChar::I,
            SeqChar::K => SeqChar::J,
            SeqChar::L => SeqChar::K,
            SeqChar::M => SeqChar::L,
            SeqChar::N => SeqChar::M,
            SeqChar::O => SeqChar::N,
            SeqChar::P => SeqChar::O,
            SeqChar::Q => SeqChar::P,
            SeqChar::R => SeqChar::Q,
            SeqChar::S => SeqChar::R,
            SeqChar::T => SeqChar::S,
            SeqChar::U => SeqChar::T,
            SeqChar::V => SeqChar::U,
            SeqChar::W => SeqChar::V,
            SeqChar::X => SeqChar::W,
            SeqChar::Y => SeqChar::X,
            SeqChar::Z => SeqChar::Y,
        }
    }

    fn char(&self) -> char {
        match *self {
            SeqChar::A => 'a',
            SeqChar::B => 'b',
            SeqChar::C => 'c',
            SeqChar::D => 'd',
            SeqChar::E => 'e',
            SeqChar::F => 'f',
            SeqChar::G => 'g',
            SeqChar::H => 'h',
            SeqChar::I => 'i',
            SeqChar::J => 'j',
            SeqChar::K => 'k',
            SeqChar::L => 'l',
            SeqChar::M => 'm',
            SeqChar::N => 'n',
            SeqChar::O => 'o',
            SeqChar::P => 'p',
            SeqChar::Q => 'q',
            SeqChar::R => 'r',
            SeqChar::S => 's',
            SeqChar::T => 't',
            SeqChar::U => 'u',
            SeqChar::V => 'v',
            SeqChar::W => 'w',
            SeqChar::X => 'x',
            SeqChar::Y => 'y',
            SeqChar::Z => 'z',
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "a" => Some(SeqChar::A),
            "b" => Some(SeqChar::B),
            "c" => Some(SeqChar::C),
            "d" => Some(SeqChar::D),
            "e" => Some(SeqChar::E),
            "f" => Some(SeqChar::F),
            "g" => Some(SeqChar::G),
            "h" => Some(SeqChar::H),
            "i" => Some(SeqChar::I),
            "j" => Some(SeqChar::J),
            "k" => Some(SeqChar::K),
            "l" => Some(SeqChar::L),
            "m" => Some(SeqChar::M),
            "n" => Some(SeqChar::N),
            "o" => Some(SeqChar::O),
            "p" => Some(SeqChar::P),
            "q" => Some(SeqChar::Q),
            "r" => Some(SeqChar::R),
            "s" => Some(SeqChar::S),
            "t" => Some(SeqChar::T),
            "u" => Some(SeqChar::U),
            "v" => Some(SeqChar::V),
            "w" => Some(SeqChar::W),
            "x" => Some(SeqChar::X),
            "y" => Some(SeqChar::Y),
            "z" => Some(SeqChar::Z),
            _ => None,
        }
    }
}
