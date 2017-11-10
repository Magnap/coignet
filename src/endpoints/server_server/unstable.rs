use rocket;

pub fn mount(r: rocket::Rocket, base: String) -> rocket::Rocket {
    r.mount(&(base + "v1"), routes![])
}
