#![allow(clippy::extra_unused_lifetimes)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

#[macro_use]
extern crate rocket;
use rocket::http::{Cookie, CookieJar};
use rocket::response::status::{BadRequest, Unauthorized};
use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::{State};

use kbs_types::{Attestation, Request, SevRequest, Tee};
use uuid::Uuid;

use reference_kbs::attester::Attester;
use reference_kbs::sev::SevAttester;
use reference_kbs::{Session, SessionState};

use std::env;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Measurement {
    workload_id: String,
    launch_measurement: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Secret {
    key_id: String,
    secret: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Workload {
    workload_id: String,
    launch_measurement: String,
    tee_config: String,
    passphrase: String,
    affinity: Vec<String>,
}

#[get("/")]
fn index() -> Result<String, Unauthorized<String>> {
    //Ok("Hello, world!".to_string())
    Err(Unauthorized(None))
}

fn get_workload() -> Workload {
    let json = env::var("WORKLOAD").expect("WORKLOAD environment variable not defined");
    let object: Workload = serde_json::from_str(&json).expect("WORKLOAD is not a valid JSON");
    return object;
}

#[post("/auth", format = "application/json", data = "<request>")]
async fn auth(
    state: &State<SessionState>,
    cookies: &CookieJar<'_>,
    request: Json<Request>,
) -> Result<Value, BadRequest<String>> {
    let session_id = Uuid::new_v4().to_simple().to_string();

    let mut attester: Box<dyn Attester> = match request.tee {
        Tee::Sev => {
            let sev_request: SevRequest = serde_json::from_str(&request.extra_params)
                .map_err(|e| BadRequest(Some(e.to_string())))?;

            let workload = get_workload();
            if sev_request.workload_id != workload.workload_id {
                return Err(BadRequest(Some("Invalid workload".to_string())));
            }

            let cek = sev_request.chain.sev.cek.to_string();
            if workload.affinity.len() > 0 && workload.affinity.iter().position(|s| *s == cek).is_none() {
                return Err(BadRequest(Some("Wrong CEC".to_string())));
            }

            Box::new(SevAttester::new(
                sev_request.workload_id.clone(),
                session_id.clone(),
                sev_request.build,
                sev_request.chain,
                Some(workload.tee_config),
            )) as Box<dyn Attester>
        }
        _ => return Err(BadRequest(Some("Unsupported TEE".to_string()))),
    };

    let challenge = attester
        .challenge()
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    let session = Session::new(session_id, attester.workload_id().clone(), attester);
    cookies.add(Cookie::new("session_id", session.id()));

    state
        .sessions
        .write()
        .unwrap()
        .insert(session.id(), Arc::new(Mutex::new(session)));
    Ok(json!(challenge))
}

#[post("/attest", format = "application/json", data = "<attestation>")]
async fn attest(
    state: &State<SessionState>,
    cookies: &CookieJar<'_>,
    attestation: Json<Attestation>,
) -> Result<(), BadRequest<String>> {
    let session_id = cookies
        .get("session_id")
        .ok_or_else(|| BadRequest(Some("Missing cookie".to_string())))?
        .value();

    // We're just cloning an Arc, looks like a false positive to me...
    #[allow(clippy::significant_drop_in_scrutinee)]
    let session_lock = match state.sessions.read().unwrap().get(session_id) {
        Some(s) => s.clone(),
        None => return Err(BadRequest(Some("Invalid cookie".to_string()))),
    };

    let workload = get_workload();
    assert_eq!(session_lock.lock().unwrap().workload_id(), workload.workload_id);

    let mut session = session_lock.lock().unwrap();
    session
        .attester()
        .attest(&attestation, &workload.launch_measurement)
        .map_err(|e| BadRequest(Some(e.to_string())))?;
    session.approve();

    Ok(())
}

#[get("/key/<key_id>")]
async fn key(
    state: &State<SessionState>,
    cookies: &CookieJar<'_>,
    key_id: &str,
) -> Result<Value, Unauthorized<String>> {
    let session_id = cookies
        .get("session_id")
        .ok_or_else(|| Unauthorized(Some("Missing cookie".to_string())))?
        .value();

    // We're just cloning an Arc, looks like a false positive to me...
    #[allow(clippy::significant_drop_in_scrutinee)]
    let session_lock = match state.sessions.read().unwrap().get(session_id) {
        Some(s) => s.clone(),
        None => return Err(Unauthorized(Some("Invalid cookie".to_string()))),
    };

    if !session_lock.lock().unwrap().is_valid() {
        return Err(Unauthorized(Some("Invalid session".to_string())));
    }

    let workload = get_workload();
    assert_eq!(session_lock.lock().unwrap().workload_id(), workload.workload_id);
    if key_id.to_string() != workload.workload_id {
        return Err(Unauthorized(Some("Invalid key".to_string())));
    }
    
    let mut session = session_lock.lock().unwrap();
    let secret = session
        .attester()
        .encrypt_secret(workload.passphrase.as_bytes())
        .unwrap();
    Ok(secret)
}

#[launch]
fn rocket() -> _ {
    get_workload(); // will panic if the WORKLOAD environment variable is not defined
    rocket::build()
        .mount(
            "/kbs/v0",
            routes![index, auth, attest, key],
        )
        .manage(SessionState {
            sessions: RwLock::new(HashMap::new()),
        })
}
