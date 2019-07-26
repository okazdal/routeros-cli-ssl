extern crate clap;
use clap::{App, Arg};

use std::thread;
use std::io::{self, Write, Error, BufReader, BufRead, Read};
use std::net::TcpStream;
use openssl::ssl::{SslMethod, SslConnector, SslVerifyMode, SslStream};
use bytes::{Bytes};
use std::process;
use std::str;
use std::intrinsics::transmute;


struct Router {
    ip: String,
    username: String,
    password: String,
    port: String,

}

impl Router {
    fn new(ip: String, username: String, password: String, port: String) -> Router {

        Router {
            ip,
            username,
            password,
            port,
        }
    }


    fn login(&self, stream: &mut SslStream<TcpStream>) -> Result<(), Error> {
        println!("Logging in..");

        let bytes = Bytes::from("/login");
        let length = bytes.len();

        stream.write(&[length as u8])?;
        stream.write(&bytes[..])?;

        let bytes = Bytes::from(format!("=name={}", self.username));
        let length = bytes.len();

        stream.write(&[length as u8])?;
        stream.write(&bytes[..])?;

        let bytes = Bytes::from(format!("=password={}", self.password));
        let length = bytes.len();

        stream.write(&[length as u8])?;
        stream.write(&bytes[..])?;
        stream.write(&[0])?;

        Ok(())
    }

}


fn main() {

    let matches = App::new("RouterOsCli")
                        .version("0.1.0")
                        .about("RouterOS API CLI client")
                        .arg(Arg::with_name("ip")
                            .help("IP Address")
                            .index(1)
                            .required(true))
                        .arg(Arg::with_name("port")
                            .help("API Port")
                            .index(2)
                            .required(true))
                        .arg(Arg::with_name("username")
                            .help("Username")
                            .index(3)
                            .required(true))
                        .arg(Arg::with_name("password")
                            .help("Password")
                            .index(4)
                            .required(true))
                        .get_matches();

    let ip = matches.value_of("ip").unwrap();
    let port = matches.value_of("port").unwrap();
    let username = matches.value_of("username").unwrap();
    let password = matches.value_of("password").unwrap();


    let r = Router::new(ip.to_string(), username.to_string(),
                        password.to_string(), port.to_string());



    let mut ctx = SslConnector::builder(SslMethod::tls()).unwrap();
    ctx.set_verify(SslVerifyMode::NONE);
    let connector = ctx.build();

    let mut tcp_stream = TcpStream::connect(format!("{}:{}", r.ip, r.port))
        .expect("Can not connect to router");
    let mut stream = connector.connect(ip, tcp_stream).unwrap();

    println!("Enter your command: (e To exit)");


    match r.login(&mut stream) {
        Ok(_) => println!("Logged in..."),
        Err(_) => eprintln!("Can not log in...")
    }
    read_reply(&mut stream);


    loop {
        let mut command = vec![];


        loop {
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    if input == "\n" {
                        break;
                    } else if input == "e\n" {
                        process::exit(0);
                    }
                    command.push(input);
                },
                Err(_) => println!("Invalid command")
            };
        };


        match send_command(&mut stream, command) {
            Ok(_) => println!("Waiting for reply..."),
            Err(_) => eprintln!("Error running command...")
        };


        read_reply(&mut stream);

    }

}


fn send_command(stream: &mut SslStream<TcpStream>, command: Vec<String>) -> Result<(), Error> {
    for cmd in command {
        let bytes = Bytes::from(cmd.trim());
        let length = bytes.len();
        stream.write(&[length as u8]).unwrap();
        stream.write(&bytes[..]).unwrap();

    };
    stream.write(&[0]).unwrap();

    Ok(())
}

fn process_su8(sen: &[u8]) -> Result<Vec<&[u8]>, Error> {

    let mut words: Vec<&[u8]> = Vec::new();

    let mut acc = 0;
    loop {
        let n = sen[acc] as usize;
        let word: &[u8];
        if n + acc + 1 > sen.len() {
            word = &sen[..];

        } else {
            word = &sen[acc..n+acc+1];

        }
        words.push(&word[1..]);
        acc = sen[acc] as usize + acc + 1;
        if acc > sen.len() {break;}
        if sen[acc] == 0 {break;}
    };

    Ok(words)

}

fn read_reply(stream: &mut SslStream<TcpStream>) {

    let mut reader = BufReader::new(stream);

    let mut buffer = Vec::new();
    loop {
        match reader.read_until(0u8, &mut buffer) {
            Ok(n) => {
                if n == 0 {
                    process::exit(0)
                } else {


                    let reply = process_su8(&mut buffer[..]).unwrap();
                    let mut break_now: bool = false;
                    for r in reply {

//                       TODO remove non utf chars
                        io::stdout().write(b">>> ").unwrap();
                        io::stdout().write(r).unwrap();
                        io::stdout().write(b"\n").unwrap();

//                        break at !done
                        if r == [33, 100, 111, 110, 101] {
                            break_now = true;
                        }
                    }

                    io::stdout().write(b"\n").unwrap();
                    buffer.clear();
                    if break_now {break;}
                }
            } ,
            Err(e) => eprintln!("Error: {}", e),
        }
    }


}
