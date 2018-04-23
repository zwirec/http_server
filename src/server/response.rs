extern crate chrono;

use self::chrono::prelude::*;
use strum::{EnumProperty, EnumMessage};
use std::fs::File;
use std::fmt;
use std::net::TcpStream;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::string::ToString;


static CRLF: &'static str = "\r\n";
static HTTP: &'static str = "HTTP/1.1 ";

#[derive(EnumProperty, EnumString, Debug, EnumMessage, ToString)]
pub enum Headers {
    #[strum(serialize = "Content-Type: ")]
    #[strum(message = "Content-Type: ")]
    ContentType(String),

    #[strum(serialize = "Content-Length: ")]
    #[strum(message = "Content-Length: ")]
    ContentLength(String),

    #[strum(serialize = "Date: ")]
    #[strum(message = "Date: ")]
    Date(String),

    #[strum(serialize = "RST")]
    #[strum(message = "Server: ")]
    Server(String),

    #[strum(serialize = "Connection: ")]
    #[strum(message = "Connection: ")]
    Connection(String),
}


#[derive(EnumProperty, EnumString, Debug, EnumMessage)]
pub enum Statuses {
    #[strum(message = "200 OK")]
    OK,

    #[strum(message = "404 Not Found")]
    NotFound,

    #[strum(message = "405 Not Allowed")]
    NotAllowed,
}

#[derive(EnumProperty, EnumString, Debug, EnumMessage)]
pub enum MimeTypes {
    #[strum(message = "text/html")]
    #[strum(serialize = "html", serialize = "htm")]
    Html(String),

    #[strum(message = "text/css")]
    #[strum(serialize = "css")]
    Css(String),

    #[strum(message = "application/javascript")]
    #[strum(serialize = "js")]
    Js(String),

    #[strum(message = "image/jpeg")]
    #[strum(serialize = "jpeg", serialize = "jpg")]
    Jpg(String),

    #[strum(message = "image/png")]
    #[strum(serialize = "png")]
    Png(String),

    #[strum(message = "image/gif")]
    #[strum(serialize = "gif")]
    Gif(String),

    #[strum(message = "application/x-shockwave-flash")]
    #[strum(serialize = "swf")]
    Swf(String),

    #[strum(message = "application/octet-stream")]
    Other(String),
}

#[derive(Debug)]
pub struct Response {
    pub headers: Vec<Headers>,
    pub status: Option<Statuses>,
    pub file: Option<File>,
}


impl Response {
    pub fn new() -> Response {
        Response {
            headers: vec![],
            status: None,
            file: None,
        }
    }

    pub fn send(self, ref mut stream: &mut TcpStream) {
        let status = self.status.unwrap();
        let mut buf = String::new();
        buf.push_str(HTTP);
        buf.push_str(status.get_message().unwrap());
        buf.push_str(CRLF);
        for h in self.headers {
            let h_value = match &h {
                &Headers::ContentLength(ref v) => &v,
                &Headers::ContentType(ref v) => &v,
                &Headers::Date(ref v) => &v,
                &Headers::Connection(ref v) => &v,
                &Headers::Server(_) => "RST",
            };
            buf.push_str(format!("{}{}", h.get_message().unwrap(),  &&h_value).as_str());
            buf.push_str(CRLF);
        }
        buf.push_str(CRLF);
        stream.write(buf.as_bytes()).unwrap();
        match self.file {
            Some(mut f) => {
//                buf.push_str(CRLF);
                let mut buf = [0; 1024 * 1024];
                let mut n: u64 = 0;
                loop {
                    match f.read(&mut buf).unwrap() {
                        0 => { break; }
                        i => {
                            n += i as u64;
//                println!("{}", i);
                            stream.write(&buf[..i]).unwrap();
                            f.seek(SeekFrom::Start(n as u64));
                        }
                    }
                }
            }
            None => {}
        }
        stream.flush().unwrap();
    }
}