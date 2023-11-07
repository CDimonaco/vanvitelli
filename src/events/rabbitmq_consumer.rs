use crate::events::policy::EventsPolicy;
use amqprs::{
    channel::{BasicAckArguments, Channel},
    consumer::AsyncConsumer,
    BasicProperties, Deliver,
};
use log::{debug, error};

pub struct RabbitMqConsumer {
    policy: Box<dyn EventsPolicy>,
}

impl RabbitMqConsumer {
    pub fn new(events_policy: impl EventsPolicy + 'static) -> RabbitMqConsumer {
        RabbitMqConsumer {
            policy: Box::new(events_policy),
        }
    }
}

#[async_trait::async_trait]
impl AsyncConsumer for RabbitMqConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        debug!("consume delivery {} on channel {}", deliver, channel);

        match self.policy.handle_event(content).await {
            Ok(_) => {
                debug!("processed event {} - {}", deliver, channel)
            }
            Err(err) => {
                error!("error during event processing {}", err)
            }
        }

        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .expect("unable to ack rabbitmq message, fatal");
    }
}
