use rocket;

pub fn mount(r: rocket::Rocket, base: String) -> rocket::Rocket {
    r.mount(&(base + "unstable"), routes![])
}
