use std::{
    env, fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use web_server::ThreadPool;

fn main() {
    let args: Vec<String> = env::args().collect();
    let num_threads: usize = args[1].parse().unwrap();
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(num_threads).unwrap();

    // To invoke the drop function of the ThreadPool, use listener.incoming.take(n) where n is the
    // maximum number of incoming requests you want to process
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // An example of creating theads for each incoming stream
        // This is a really bad idea as it can overload the system with a lot of threads and create
        // issues in case of a DDoS attack. We would need a finite number of threads (a thread pool) to manage this problem
        // thread::spawn(|| handle_connection(stream));

        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // HTTP Request:
    // 1: Method Request-URI HTTP-Version CRLF
    // 2: headers CRLF
    // 3: message-body
    // let http_request: Vec<_> = buf_reader
    // .lines()
    // .map(|result| result.unwrap())
    // .take_while(|line| !line.is_empty())
    // .collect();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "pages/hello.html"),
        // A simulated slow response
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "pages/sleep.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "pages/404.html"),
    };
    // HTTP Response:
    // 1: HTTP-Version Status-Code Reason-Phrase CRLF
    // 2: headers CRLF
    // 3: message-body

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
