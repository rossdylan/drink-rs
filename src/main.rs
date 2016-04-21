#[macro_use]
extern crate nom;
extern crate mio;


pub mod sunday;
pub mod sunday_server;
pub mod sunday_dispatcher;


pub fn main() {
    sunday_server::start("0.0.0.0:4242".parse().unwrap());
}
