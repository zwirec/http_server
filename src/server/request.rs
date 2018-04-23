extern crate regex;
extern crate percent_encoding;

use self::regex::Regex;
use std::str;
use std::str::FromStr;
use self::percent_encoding::percent_decode;

#[derive(EnumString, Debug, PartialEq)]
pub enum Methods {
    #[strum(serialize = "GET")]
    GET,

    #[strum(serialize = "POST")]
    POST,

    #[strum(serialize = "HEAD")]
    HEAD,

    PUT,

    OPTION,

    PATCH,
}

#[derive(EnumString)]
pub enum Statuses {
    #[strum(serialize = "200")]
    Status200,

    #[strum(serialize = "404")]
    Status404,

    #[strum(serialize = "405")]
    Status405,
}

#[derive(EnumString)]
pub enum Proto {
    #[strum(serialize = "0")]
    H10,
    #[strum(serialize = "1")]
    H11,
}

pub struct Request {
    pub proto: Option<Proto>,
    pub method: Option<Methods>,
    pub path: Option<String>,
}

impl Request {
    pub fn new() -> Request {
        Request {
            proto: None,
            method: None,
            path: None,
        }
    }

    pub fn parse(buffer: &[u8]) -> Option<Request> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?P<method>\w*)\s*(?P<path>.*?)\s*HTTP/1\.(?P<proto>0|1).*").unwrap();
        }
        let caps = RE.captures(str::from_utf8(buffer).unwrap());
        let caps = match caps {
            Some(c) => c,
            None => return None,
        };

        let method = caps.name("method").
            map(|v| Methods::from_str(v.as_str()).unwrap());

        let proto = caps.name("proto").
            map(|v| Proto::from_str(v.as_str()).unwrap());

        let path = caps.name("path").
            map(|v| v.as_str()).
            map(|v| trunc_query_string(v)).
            map(|v| percent_decode(v.as_bytes()).decode_utf8().unwrap()).
            map(|v| remove_dot_segments(v.to_string())).
            map(|v| add_index(v));

        return Some(Request {
            method,
            path: path.map(|v| String::from(v)),
            proto,
        });


//        self.method = caps.name("method").
//            map(|v| Methods::from_str(v.as_str()).unwrap());
//
//        self.proto = caps.name("proto").
//            map(|v| Proto::from_str(v.as_str()).unwrap());
//
//        self.path = caps.name("path").
//            map(|v| String::from(v.as_str()));
    }
}

fn trunc_query_string(path: &str) -> &str {
    path.split('?').nth(0).unwrap()
}

fn add_index(path: String) -> String {
    let mut out = String::from(path.clone());
//    println!("{}split", path.split("/").last().unwrap());
    match path.split("/").last().unwrap().find(".") {
        None => { out.push_str("index.html") }
        Some(_) => {}
    }
    out.to_owned()
}

fn remove_dot_segments(path: String) -> String {
    if path.find('.') == None {
        return path.clone();
    }
    let mut path = path.clone();
    let mut output = String::with_capacity(path.len());

    while path != "" {
//        println!("path = {}", path);
        let mut p = path.clone();
        let len = p.len();

        if path.len() == 1 {
            output.push('/');
            break;
        }

        match path.find("./") {
            Some(i) => {
                if i == 0 {
                    path = p.drain(2..).collect();
                    continue;
                    ;
                }
            }
            None => {}
        }

        match path.find("../") {
            Some(i) => {
                if i == 0 {
                    path = p.drain(3..).collect();
                    continue;
                }
            }
            None => {}
        }

        if path == "/." {
            output.push('/');
            break;
        }

        if &path[0..3] == "/./" {
            path = p.drain(2..).collect();
            continue;
        }

        if path == "/.." {
//            let mut o = output.clone();
//            let len = output.len();
            output.pop();
            output.push('/');
            break;
        }

        if &path[0..4] == "/../" {
//            let mut o = output.clone();
//            let len = output.len();
//            println!("len {}", len);
            output.pop();
            path = p.drain(3..).collect();
            continue;
        }

        if path == "." || path == ".." {
            break;
        }

        let mut sub_path = String::from("");

//        println!("path_ {}", path);


        let mut p_t = p.clone();
        sub_path = p_t.drain(1..).collect();


//        println!("subpath {}", sub_path);

        match sub_path.find("/") {
            None => {
                output.push_str(path.as_str());
                break;
            }
            Some(i) => {
                let s = &path.clone()[0..i + 1];
                output.push_str(s.as_ref());
                path = p.drain(i + 1..).collect();
            }
        }
//        println!("path_end = {}", path);
    }
//    println!("output = {}", output);
    output
}