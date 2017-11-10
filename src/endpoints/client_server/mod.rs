use rocket;
use rocket_contrib::Json;

use serde_json::Value;

pub mod r0;
pub mod unstable;

pub fn mount(r: rocket::Rocket, base: String) -> rocket::Rocket {
    let base = base + "client/";
    let r = r0::mount(r, base.clone());
    let r = unstable::mount(r, base.clone());
    r.mount(&base, routes![versions])
}

#[derive(Serialize)]
pub struct Version(String);

#[get("/versions")]
fn versions() -> Json<Value> {
    let versions: Vec<_> = vec!["r.0.2.0"]
        .into_iter()
        .map(|x| Version(x.into()))
        .collect();
    Json(json!({ "versions": versions }))
}
