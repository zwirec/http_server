extern crate lazy_static;
use ThreadPool;
use std::net::TcpListener;
use config::config::Config;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::io::SeekFrom;
use ::server::request::{Request, Methods};
use ::server::response::{Response, Statuses, MimeTypes, Headers};
use std::path::Path;
use std::str::FromStr;
use strum::EnumMessage;
#[macro_use]
use std::sync::Mutex;
use chrono::prelude::*;

lazy_static! {
    static ref DOC_ROOT: Mutex<String> = Mutex::new("".to_string());
}

pub struct Server {
    pool: ThreadPool,
    listener: TcpListener,
}

impl Server {
    pub fn new(config: &Config) -> Server {
        let listener = TcpListener::bind(format!("{}{}", String::from("0.0.0.0:"), config.port.to_string())).unwrap();
        let pool = ThreadPool::new(config.thread_num);
        DOC_ROOT.lock().unwrap().push_str(config.document_root.as_str());
        Server {
            pool,
            listener,
        }
    }

    pub fn run(&self) {
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
            self.pool.execute(|| {
                handle_connection(stream);
            });
        }
        println!("Shutting down.");
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    match stream.read(&mut buffer) {
        Ok(n) if n == 0 => return,
        Ok(..) => {}
        Err(_e) => return,
    };

    let req = match Request::parse(&mut buffer) {
        Some(req) => req,
        None => return,
    };

    let resp = handle_request(req);

    match resp {
        Some(resp) => resp.send(&mut stream),
        None => {
            stream.flush().unwrap();
            return;
        }
    }

}

fn handle_request(req: Request) -> Option<Response> {
    match req.path.as_ref().and(req.proto).and(req.method.as_ref()) {
        None => return None,
        Some(_) => {}
    };

    let method = req.method.unwrap();

    let mut resp = Response::new();
    let path = req.path.unwrap();
    let doc_root = DOC_ROOT.lock().unwrap();
    match method {
        Methods::GET => {
            match File::open(format!("{}{}", doc_root, path)) {
                Ok(f) => {
                    let path = format!("{}{}", doc_root, path);
                    let p = Path::new(&path);
//                    println!("{:?}", p);
                    resp.headers.push(Headers::ContentType(
                        MimeTypes::from_str(p.
                            extension().
                            unwrap_or_else(|| panic!("extension panic")).
                            to_str().
                            unwrap_or_else(|| panic!("to_str panic"))).
                            unwrap_or(MimeTypes::Other("application/octet-stream".to_owned())).
                            get_message().unwrap_or_else(|| panic!("get_message panic")).to_owned()));
                    resp.headers.push(Headers::ContentLength(p.metadata().unwrap().len().to_string()));
                    resp.file = Some(f);
                    resp.status = Some(Statuses::OK);
                }
                Err(_) => {
                    resp.file = None;
                    resp.status = Some(Statuses::NotFound);
                }
            }
        }
        Methods::HEAD => {
            let path = format!("{}{}", doc_root, path);
            let p = Path::new(&path);
            match p.exists() {
                true => {
                    resp.headers.
                        push(Headers::ContentType(
                            MimeTypes::from_str(p.
                                extension().
                                unwrap_or_else(|| panic!("extension panic")).
                                to_str().
                                unwrap_or_else(|| panic!("to_str panic"))).
                                unwrap_or(MimeTypes::Other("application/octet-stream".to_owned())).
                                get_message().unwrap_or_else(|| panic!("get_message panic")).to_owned()));
                    resp.headers.
                        push(Headers::ContentLength(p.metadata().unwrap().len().to_string()));
                    resp.status = Some(Statuses::OK);
                }
                false => {
                    resp.file = None;
                    resp.status = Some(Statuses::NotFound);
                }
            }
        }
        _ => {
            resp.file = None;
            resp.status = Some(Statuses::NotAllowed);
        }
    }
    let utc = Utc::now();
    let utc_string = utc.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    resp.headers.push(Headers::Date(utc_string));
    resp.headers.push(Headers::Server("STATIC_SERVER_TP".to_owned()));
    resp.headers.push(Headers::Connection("close".to_owned()));
    Some(resp)
}
