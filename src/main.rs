#[macro_use]
extern crate log;

mod events;

use crate::events::{EventsPolicy, RabbitMqConsumer};

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicConsumeArguments, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
};
use tokio::sync::Notify;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    env_logger::init();

    info!("Hello, vanvitelli!");

    // open a connection to RabbitMQ server
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5674,
        "wanda",
        "wanda",
    ))
    .await
    .expect("unable to open a rabbitmq connection, fatal.");

    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .expect("unable to attach callback to rabbitmq connection, fatal.");

    info!("Connected to rabbitmq!");

    let channel = connection.open_channel(None).await.unwrap();
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .expect("unable to attach channel callback to rabbitmq connection, fatal.");

    // declare a server-named transient queue
    let (queue_name, _, _) = channel
        .queue_declare(QueueDeclareArguments::default())
        .await
        .unwrap()
        .expect("unable to create a transient queue in rabbitmq connection, fatal.");

    // bind the queue to exchange
    let routing_key = "executions";
    let exchange_name = "trento.checks";
    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            exchange_name,
            routing_key,
        ))
        .await
        .expect("unable to bind the queue in the rabbitmq connection, fatal.");

    let args = BasicConsumeArguments::new(&queue_name, "basic_consumer")
        .manual_ack(true)
        .finish();

    let policy = EventsPolicy::new("host_id")
        .expect("unable to create protobuf event policy, fatal");
    let rabbit_consumer = RabbitMqConsumer::new(policy);

    channel
        .basic_consume(rabbit_consumer, args)
        .await
        .expect("unable to consume from rabbitmq queue, fatal.");

    info!("consume forever..., ctrl+c to exit");
    let guard = Notify::new();
    guard.notified().await;
}
