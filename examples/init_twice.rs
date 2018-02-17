/*!
Call env_logger::init twice to demonstrate error.
*/

extern crate env_logger;

fn main() {
    println!("calling init first time");
    env_logger::init();
    println!("calling init second time");
    env_logger::init();
    unreachable!();
}
