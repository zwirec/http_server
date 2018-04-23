extern crate http_server;

use http_server::config::Config;
use http_server::server::server::Server;

fn main() {
    let config = Config::parse("/etc/httpd.conf").unwrap();
    let server = Server::new(&config);
    server.run();
}