use serde::{Deserialize, Serialize};
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureRecord, FutureProducer};
use std::time::Duration;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Mutex; 
use serde_json::Value;
use chrono::Utc;
use serde_json::json;
use crate::cores::schemas::{CompletionsResponse, CompletionsStreamResponse};

use crate::GLOBAL_CONFIG;

// Info for kafka 
#[derive(Deserialize, Serialize, Debug)]
pub struct Kafka {
    pub timestamp: i64,
    pub fields: FieldsInfo,
    pub tags: TagsInfo
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FieldsInfo {
    pub completion_tokens: u32,
    pub prompt_tokens: u32,
    pub total_tokens: u32
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TagsInfo {
    pub user_name: String,
    pub model_name: String,
}

static MESSAGE_QUEUE: OnceLock<Arc<Mutex<VecDeque<String>>>> = OnceLock::new();

// Start to send message to kafka server, can add error log to show error. 
pub async fn start_kafka_sender() {
    let config = &*GLOBAL_CONFIG;
    let brokers = &config.brokers;
    let topic = &config.topic;

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .expect("Failed to create Kafka producer");

    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;

        let queue = MESSAGE_QUEUE.get_or_init(|| Arc::new(Mutex::new(VecDeque::new())));
        let mut queue = queue.lock().await;
        let messages: Vec<String> = queue.drain(..).collect();

        if messages.is_empty() {
            continue;
        }

        for message in messages {
            let future_record: FutureRecord<(), [u8]> = FutureRecord::to(topic)
                .payload(message.as_bytes());

            let delivery_future = match producer.send_result(future_record) {
                Ok(delivery_future) => delivery_future,
                Err((e, _)) => {
                    eprintln!("Failed to send message to Kafka: {}", e);
                    continue;
                }
            };

            let delivery_status = tokio::time::timeout(Duration::from_secs(5), delivery_future).await;

            match delivery_status {
                Ok(Ok(Ok((_partition, _offset)))) => (),
                Ok(Ok(Err((e, _)))) => eprintln!("Failed to deliver message to Kafka: {}", e),
                Ok(Err(_)) => eprintln!("Message delivery canceled"),
                Err(_) => eprintln!("Message delivery timed out"),
            }
        }
    }
}

// Save messages for sending to kafka
pub async fn send_kafka_message_non_stream(chat_response: &CompletionsResponse, res: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let timestamp = Utc::now().timestamp();
    let kafka = Kafka {
        timestamp: timestamp,
        fields: FieldsInfo {
            completion_tokens: chat_response.usage.prompt_tokens,
            prompt_tokens: chat_response.usage.completion_tokens,
            total_tokens: chat_response.usage.total_tokens,
        },
        tags: TagsInfo {
            user_name: "example_user".to_string(),
            model_name: chat_response.model.to_string(),
        },
    };
    let kafka_json = serde_json::to_string(&kafka).unwrap();

    let result = send_to_kafka(&kafka_json).await;
    match result {
        Ok(()) => {
            Ok(res)
        },
        Err(e) => {
            Ok(json!({
                "error": "Failed to send message to Kafka queue",
                "details": format!("{:?}", e)
            }))
        }
    }
}

pub async fn send_kafka_message_stream(chat_response: &CompletionsStreamResponse) -> Result<(), String> {
    let timestamp = Utc::now().timestamp();
    let kafka = Kafka {
        timestamp: timestamp,
        fields: FieldsInfo {
            completion_tokens: chat_response.usage.as_ref().unwrap().prompt_tokens,
            prompt_tokens: chat_response.usage.as_ref().unwrap().completion_tokens,
            total_tokens: chat_response.usage.as_ref().unwrap().total_tokens,
        },
        tags: TagsInfo {
            user_name: "example_user".to_string(),
            model_name: chat_response.model.to_string(),
        },
    };
    let kafka_json = serde_json::to_string(&kafka).unwrap();

    let result = send_to_kafka(&kafka_json).await;
    match result {
        Ok(()) => {
            Ok(())
        },
        Err(e) => {
            Err(format!("Failed to send message to Kafka: {:?}", e))
        }
    }
}

pub async fn send_to_kafka(log_message: &str) -> Result<(), rdkafka::error::KafkaError> {
    let queue = MESSAGE_QUEUE.get_or_init(|| Arc::new(Mutex::new(VecDeque::new())));
    let mut queue = queue.lock().await;

    queue.push_back(log_message.to_string());

    Ok(())
}