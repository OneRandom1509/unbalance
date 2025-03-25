use std::{
    env, fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
mod health_checker;
mod scheduling;
use health_checker::check_health;
use load_balancer::ThreadPool;

pub struct Message {
    pub message: String,
    pub worker_addr: String,
    pub client_addr: String,
}
fn main() {
    //let args: Vec<String> = env::args().collect();
    //let num_threads: usize = args[1].parse().unwrap();
    let args: Vec<String> = env::args().collect();
    let worker: String = fs::read_to_string(".env").unwrap();
    let mut worker_list: Vec<String> = worker.split("\n").map(|x| x.to_string()).collect();
    worker_list.remove(worker_list.len() - 1);
    let num_workers = worker_list.len();
    let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool: ThreadPool = ThreadPool::new(num_workers).unwrap();
    //  let mut client_worker_map: HashMap<String, String> = HashMap::new();

    if args.len() > 1 {
        if args[1] == "-h" || args[1] == "--health-worker" {
            println!("Health mode on!");
            check_health(worker_list.clone());
        }
    }
    let mut worker: usize = 0;
    // To invoke the drop function of the ThreadPool, use listener.incoming.take(n) where n is the
    // maximum number of incoming requests you want to process
    for stream in listener.incoming() {
        // handle_connection(worker);
        let stream = stream.unwrap();
        // An example of creating theads for each incoming stream
        // This is a really bad idea as it can overload the system with a lot of threads and create
        // issues in case of a DDoS attack. We would need a finite number of threads (a thread pool) to manage this problem
        // thread::spawn(|| handle_connection(stream));

        // think of a better way to handle this, cloning everything everytime is kinda bad
        let worker_id = worker.clone();
        let worker_list_cl = worker_list.clone();
        pool.execute(move || {
            handle_connection(stream, worker_id, worker_list_cl);
        });

        worker = (worker + 1) % num_workers;
        // This is the health checker function runs on a separate thread;
        // let health_monitor = thread::spawn(|| {
        //     check_health(worker_list);
        //     std::thread::sleep(Duration::from_secs(10));
        // });
    }
    println!("Shutting down.");
}

/*fn handle_connection(mut stream: TcpStream) {
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
*/
fn handle_connection(mut stream: TcpStream, worker_id: usize, worker_list: Vec<String>) {
    let buf_reader = BufReader::new(&stream);
    let request_message = buf_reader.lines().next().unwrap().unwrap();
    let client_addr = stream.peer_addr().unwrap().to_string();
    let worker_addr = worker_list[worker_id].clone();
    // client_worker_map.insert(client_addr, worker_addr);
    let message: Message = Message {
        message: request_message,
        worker_addr: worker_addr.clone(),
        client_addr: client_addr.clone(),
    };
    let response = send_job(worker_addr, message);
    stream.write_all(response.as_bytes()).unwrap();
}

fn send_job(client_addr: String, message: Message) -> String {
    println!("Trying to connect to..., {}", &client_addr);
    let mut worker_connection = TcpStream::connect(client_addr).unwrap();
    worker_connection
        .write_all(message.message.as_bytes())
        .unwrap();
    worker_connection.flush().unwrap();
    println!("Send a message");
    let mut response = String::new();
    let _ = worker_connection.read_to_string(&mut response);
    println!("received response {}", &response);
    response
}
