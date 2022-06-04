use warp::{Filter, Reply, Rejection};
use std::sync::{Arc, Mutex};
use tokio::{task};
use crate::state;

pub fn set_up_web_server(state: Arc<Mutex<state::State>>) {
  let state_filter = warp::any().map(move || state.clone());
  let state_route = warp::get()
      .and(warp::path!("state"))
      .and(warp::path::end())
      .and(state_filter)
      .and_then(get_devices_state);

  task :: spawn(async move {
      warp::serve(state_route)
      .run(([127, 0, 0, 1], 3030))
      .await;
  });
}

async fn get_devices_state(state: Arc<Mutex<state::State>>) -> Result<impl warp::Reply, warp::Rejection> {
  let state_unlocked = state.lock().unwrap();
  Ok(warp::reply::json(
    &state_unlocked.get_devices_state()
  ))
}