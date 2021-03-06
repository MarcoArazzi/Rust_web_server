extern crate multi_threaded_web_server;

use multi_threaded_web_server::ThreadPool;

use std::io::prelude::*;

use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::SeekFrom;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Couldn't connect");
    let pool = ThreadPool::new(10).expect("The size must be bigger than zero");
    
    for stream in listener.incoming(){
        
        match stream {
            Ok(stream) =>{
                     let buffer = match handle_buffer_stream(&stream){
                        Ok(buffer) => (buffer),
                        Err(())=>{break;},
                     };
                     pool.execute(move || {
                     handle_connection(stream, buffer);
                })
                },
            Err(e) => {
                println!("{:?}", e);
                break;
            },
        };
     }
}

fn handle_buffer_stream(mut stream: &TcpStream)-> Result<[u8; 512],()>{
    let mut buffer = [0; 512];  
    stream.read(&mut buffer).expect("Failed to read the message");  
    let close = b"GET /shutdown HTTP/1.1\r\n";
    
   if buffer.starts_with(close){
       return Err(());
   }
         
    Ok(buffer)
}

fn handle_connection(mut stream: TcpStream, buffer: [u8; 512]){
    
    let get = b"GET / HTTP/1.1\r\n";
    let test = b"GET /test HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    let secure_c = b"GET /secure_c HTTP/1.1\r\n";
    
    let (status, file) = if buffer.starts_with(get){
            ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
        }else if buffer.starts_with(test){
            ("HTTP/1.1 200 OK\r\n\r\n", "test.html")
        }else if buffer.starts_with(sleep){
            thread::sleep(Duration::from_secs(20));
            ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
        }else if buffer.starts_with(secure_c){
            ("HTTP/1.1 200 OK\r\n\r\n", "secure_c.html")
        }else{
            ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
        };
    
    let mut file = File::open(file).expect("Failed to open the file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Failed to read the file");
    
    let log = format!("Peer: {}\nRequest {}",            
                            stream.peer_addr().unwrap(), String::from_utf8_lossy(&buffer[..]));
    
    log_file(&log);
    
    let response = format!("{}{}", status, contents);
    
    stream.write(response.as_bytes()).expect("Failed to write the response");
    
    stream.flush().expect("Failed to flush the stream");
    
    //println!("{}", log);
    
}

fn log_file(string: &String){
    let mut option = OpenOptions::new();
    option.read(true);
    option.write(true);
    option.create(true);

    match option.open("log.txt") {
        Err(_) => {
            println!("Error");
        }
        Ok(mut f) => {
            println!("File opened");
            let size = f.seek(SeekFrom::End(0)).unwrap();
            let n_string = match size {
                0 => string.clone(),
                _ => format!("\n{}", string),
            };
            match f.write_all(n_string.as_bytes()) {
                Err(_) => {
                    println!("Write error");
                }
                Ok(_) => {
                    println!("Write success");
                }
            };

            f.sync_all();
        }
    }
}
