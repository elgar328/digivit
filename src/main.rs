use std::{
    str, env, thread,
    fs::OpenOptions,
    net::{UdpSocket, SocketAddr},
    time::{Duration, Instant},
    io::{self, Error, ErrorKind, Write},
    sync::mpsc,
};

fn main() -> std::io::Result<()> {
    
    let mut ip_addr = String::new();
    print!("Enter IP address (default: 0.0.0.0): ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut ip_addr).unwrap();
    if ip_addr.is_empty() {
        ip_addr = "0.0.0.0".to_string();
    }
    ip_addr += ":55555";

    let socket = UdpSocket::bind(ip_addr)
        .expect("Failed to bind socket");
    socket.set_nonblocking(true)?;

    // a  : sequence character from ASCII 'a' to 'z'
    // MD : Monitor distance output 100000=100%
    let message = "aMD";
    let server_address = "192.168.0.145:55556";
    let timeout = Duration::from_millis(300);

    let output_filename = env::current_exe().unwrap()
        .parent().unwrap().join("output.txt");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_filename)?;

    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        sender.send(()).unwrap();
    });

    let start_time = Instant::now();
    let mut buf = [0; 1024];

    println!("Data acquisition in progress..\nPress ENTER key to stop.");
    loop {
        socket.send_to(assemble_packet(message).as_bytes(), server_address)?;

        match receive_with_timeout(&socket, timeout, &mut buf) {
            Ok((received, _addr)) => {
                let elapsed_time = start_time.elapsed().as_secs_f64();
                let output = format!("{:.4}, {}\n", elapsed_time, received);
                file.write(output.as_bytes())?;
            },
            Err(err) => {
                let elapsed_time = start_time.elapsed().as_secs_f64();
                println!("{:.2} sec : {}", elapsed_time, err);
            },
        };

        if receiver.try_recv().is_ok() { break; }
    }

    Ok(())
}

fn assemble_packet(payload: &str) -> String {
    format!("${}#{}", payload, calculate_checksum(payload))
}

fn calculate_checksum(payload: &str) -> String {
    let sum: u8 = payload.as_bytes().iter().fold(0, |acc, &x| acc.wrapping_add(x));
    format!("{:02X}", !sum)
}

fn receive_with_timeout(socket: &UdpSocket, timeout: Duration, buf: &mut [u8]) -> Result<(String, SocketAddr), Error> {
    let start_time = std::time::Instant::now();

    loop {
        match socket.recv_from(buf) {
            Ok((size, addr)) => {
                let received = String::from_utf8(buf[..size].to_vec())
                    .map_err(|e| Error::new(ErrorKind::Other, format!("Invalid UTF-8 sequence: {}", e)))?;
                if verify_checksum(&received) {
                    return Ok((received[2..received.len()-4].to_owned(), addr));
                } else {
                    return Err(Error::new(ErrorKind::Other, "checksum error"));
                }
            },
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                if start_time.elapsed() >= timeout {
                    return Err(Error::new(ErrorKind::TimedOut, "timed out while waiting for response."));
                }
            },
            Err(e) => return Err(e),
        }
    }
}

fn verify_checksum(received: &str) -> bool {
    let len = received.len();
    let checksum = &received[len-3..len-1];
    let seq_and_payload = &received[0..len-4];
    if calculate_checksum(seq_and_payload) == checksum {
        return true;
    } else {
        return false;
    }
}
