extern crate coignet;
extern crate rocket;

use coignet::endpoints;

fn main() {
    let r = rocket::ignite();
    let r = endpoints::mount(r, "/".into());
    r.launch();
}
