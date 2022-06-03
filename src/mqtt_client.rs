
use std::sync::{Arc, Mutex};

use rumqttc::{self, AsyncClient, MqttOptions, EventLoop,QoS, SubscribeFilter, Event, Incoming};
use tokio::{task};

use crate::{message_handler::MessageHandler, state::State};


pub struct MqttClient {
  pub client: AsyncClient,
  event_loop: EventLoop,
  handler: Arc<MessageHandler>,
}

impl MqttClient {
  pub async fn new(options: MqttOptions, cap: usize, topics: &Vec<String>, handler: Arc<MessageHandler>) -> Self {
    let (client, event_loop) = AsyncClient::new(options, cap);
    subscribe_to_topics(client.clone(), topics.clone()).await;
    MqttClient {
      client,
      event_loop,
      handler
    }
  }

  pub async fn run(& mut self, state: Arc<Mutex<State>>) {
    loop {
      let event = self.event_loop.poll().await;
      if let Ok(Event::Incoming(Incoming::Publish(publish))) = event {
        let handler = self.handler.clone();
        let state = state.clone();
        task::spawn(async move {
          handler.handle_message(publish.payload.clone(), &publish.topic, state);
        });
      }
    }
  }

}

async fn subscribe_to_topics(client: AsyncClient,  topics: Vec<String>)  {
  task::spawn(async move {
    requests(client, topics).await;
  }).await.unwrap()
}

async fn requests(client: AsyncClient,  topics: Vec<String>) -> (){
  let subscribe_filters = topics.iter().map(|topic| SubscribeFilter::new(topic.to_string(), QoS::AtLeastOnce)).collect::<Vec<_>>();
  client
      .subscribe_many(subscribe_filters)
      .await
      .unwrap();
}