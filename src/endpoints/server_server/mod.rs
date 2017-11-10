use rocket;

pub mod unstable;

pub fn mount(r: rocket::Rocket, base: String) -> rocket::Rocket {
    let base = base + "federation/";
    unstable::mount(r, base)
}
