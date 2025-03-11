#![allow(unused)]

struct Conf {
    retry: u32,
    timeout: u32,
}

trait RequestClient {
    fn send(&self);
}

struct GrpcRequestClient {
    conf: Conf,
}
impl RequestClient for GrpcRequestClient {
    fn send(&self) {
        println!("Send grpc request");
    }
}

struct HttpRequestClient {
    conf: Conf,
}
impl RequestClient for HttpRequestClient {
    fn send(&self) {
        println!("Send http request");
    }
}

trait Logger {
    fn log(&self);
}

struct StdoutLogger;
impl Logger for StdoutLogger {
    fn log(&self) {
        println!("Log to stdout");
    }
}

struct RemoteLogger;
impl Logger for RemoteLogger {
    fn log(&self) {
        println!("Log to remote");
    }
}

struct Service {
    client: Box<dyn RequestClient>,
    logger: Box<dyn Logger>,
}
impl Service {
    fn call(&self) {
        self.client.send();
    }
}

fn main() {
    println!("This is my sample file.");

    let conf = Conf {
        retry: 3,
        timeout: 30,
    };
    let stdout_logger = StdoutLogger;
    let grpc_client = GrpcRequestClient { conf };
    let grpc_service = Service {
        client: Box::new(grpc_client),
        logger: Box::new(stdout_logger),
    };
    grpc_service.call();

    let conf = Conf {
        retry: 3,
        timeout: 60,
    };
    let remote_logger = RemoteLogger;
    let http_client = HttpRequestClient { conf };
    let http_service = Service {
        client: Box::new(http_client),
        logger: Box::new(remote_logger),
    };
    http_service.call();
}
