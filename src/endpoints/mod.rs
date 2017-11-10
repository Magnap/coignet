use rocket;

pub mod server_server;
pub mod client_server;

pub fn mount(r: rocket::Rocket, base: String) -> rocket::Rocket {
    let base = base + "_matrix/";
    let r = server_server::mount(r, base.clone());
    client_server::mount(r, base)
}
