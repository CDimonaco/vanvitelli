use crate::events::policy::EventsPolicy;
use anyhow::{anyhow, Result};
use log::{info, warn};
use trento_contracts::events::{event_data_from_event, event_type_from_raw_bytes};
use trento_contracts::stubs::execution_requested::ExecutionRequested;

pub struct ProtobufEventsPolicy {
    host_id: String,
}

const REQUEST_EXECUTION_EVENT_TYPE: &str = "Trento.Checks.V1.ExecutionRequested";

impl ProtobufEventsPolicy {
    pub fn new(host_id: &str) -> Result<ProtobufEventsPolicy> {
        if host_id.len() == 0 {
            return Err(anyhow!("missing host_id, cannot create Policy"));
        }
        Ok(ProtobufEventsPolicy {
            host_id: host_id.to_owned(),
        })
    }
}

#[async_trait::async_trait]
impl EventsPolicy for ProtobufEventsPolicy {
    async fn handle_event(&self, raw_event: Vec<u8>) -> Result<()> {
        let event_type = event_type_from_raw_bytes(&raw_event)?;

        match event_type.as_str() {
            REQUEST_EXECUTION_EVENT_TYPE => {
                let mut request_execution_event = ExecutionRequested::new();
                event_data_from_event(&raw_event, &mut request_execution_event)?;

                info!(
                    "Execution requested event: execution_id {}, group_id {}",
                    request_execution_event.execution_id, request_execution_event.group_id
                );
            }
            _ => {
                warn!("unrecognized event type {}, skipping", event_type);
            }
        }
        Ok(())
    }
}
