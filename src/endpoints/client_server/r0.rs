use rocket;

pub fn mount(r: rocket::Rocket, base: String) -> rocket::Rocket {
    r.mount(&(base + "r0"), routes![])
}
