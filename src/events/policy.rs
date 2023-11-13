use std::collections::HashMap;

use anyhow::{anyhow, Result};
use log::{info, warn};
use trento_contracts::events::{event_data_from_event, event_type_from_raw_bytes};
use trento_contracts::stubs::facts_gathering_requested::{
    FactsGatheringRequested, FactsGatheringRequestedTarget,
};

use crate::gatherers::{FactRequest, FactsGatheringRequest};

pub struct EventsPolicy {
    agent_id: String,
}

const FACTS_GATHERING_REQUEST_EVENT_TYPE: &str = "Trento.Checks.V1.FactsGatheringRequested";

impl EventsPolicy {
    pub fn new(agent_id: &str) -> Result<EventsPolicy> {
        if agent_id.len() == 0 {
            return Err(anyhow!("missing agent_id, cannot create Policy"));
        }
        Ok(EventsPolicy {
            agent_id: agent_id.to_owned(),
        })
    }
}

impl EventsPolicy {
    pub async fn handle_event(&self, raw_event: Vec<u8>) -> Result<()> {
        let event_type = event_type_from_raw_bytes(&raw_event)?;

        match event_type.as_str() {
            FACTS_GATHERING_REQUEST_EVENT_TYPE => {
                let mut facts_request_event = FactsGatheringRequested::new();
                event_data_from_event(&raw_event, &mut facts_request_event)?;

                let facts_request_for_agent: Vec<&FactsGatheringRequestedTarget> =
                    facts_request_event
                        .targets
                        .iter()
                        .filter(|t| t.agent_id == self.agent_id)
                        .collect();

                if facts_request_for_agent.is_empty() {
                    info!(
                        "execution requested for other agents, skipping execution with id: {} - host_id: {}",
                        facts_request_event.execution_id,
                        self.agent_id
                    );

                    return Ok(());
                }

                info!(
                    "execution requested event: execution_id {}, group_id {}",
                    facts_request_event.execution_id, facts_request_event.group_id
                );
            }
            _ => {
                warn!("unrecognized event type {}, skipping", event_type);
            }
        }
        Ok(())
    }
}

fn map_fact_gathering_request_from_event(
    event_requests: Vec<&FactsGatheringRequestedTarget>,
    execution_id: String,
    group_id: String,
) -> FactsGatheringRequest {
    let fact_requests: Vec<FactRequest> = event_requests
        .iter()
        .flat_map(|target| {
            target
                .fact_requests
                .iter()
                .map(|event_request| FactRequest {
                    argument: event_request.argument.to_owned(),
                    check_id: event_request.check_id.to_owned(),
                    gatherer: event_request.gatherer.to_owned(),
                    name: event_request.name.to_owned(),
                })
        })
        .collect();

    let mut fact_requests_for_gatherer: HashMap<String, Vec<FactRequest>> = HashMap::new();

    for request in fact_requests {
        let mut gatherer_requests: Vec<FactRequest> = fact_requests_for_gatherer
            .get(&request.gatherer)
            .get_or_insert(&vec![])
            .to_vec();
        gatherer_requests.push(request.clone());

        fact_requests_for_gatherer.insert(request.gatherer, gatherer_requests);
    }

    FactsGatheringRequest {
        execution_id: execution_id,
        group_id: group_id,
        facts_requests_by_gatherer: fact_requests_for_gatherer,
    }
}

#[cfg(test)]
mod test {
    use trento_contracts::stubs::facts_gathering_requested::FactRequest;

    use super::*;

    #[test]
    fn test_fact_gathering_request_from_event() {
        let execution_id = "exec1";
        let group_id = "group1";

        let first_target = FactsGatheringRequestedTarget {
            agent_id: "agent_1".to_owned(),
            fact_requests: vec![
                FactRequest {
                    argument: "arg1".to_owned(),
                    check_id: "check1".to_owned(),
                    gatherer: "test_gat".to_owned(),
                    name: "fact1".to_owned(),
                    ..Default::default()
                },
                FactRequest {
                    argument: "arg2".to_owned(),
                    check_id: "check1".to_owned(),
                    gatherer: "test_gat".to_owned(),
                    name: "fact2".to_owned(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let second_target = FactsGatheringRequestedTarget {
            agent_id: "agent_1".to_owned(),
            fact_requests: vec![
                FactRequest {
                    argument: "arg1".to_owned(),
                    check_id: "check1".to_owned(),
                    gatherer: "test_gat3".to_owned(),
                    name: "fact3".to_owned(),
                    ..Default::default()
                },
                FactRequest {
                    argument: "arg1".to_owned(),
                    check_id: "check1".to_owned(),
                    gatherer: "test_gat4".to_owned(),
                    name: "fact4".to_owned(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let targets: Vec<&FactsGatheringRequestedTarget> = vec![
            &first_target,
            &second_target
        ];

        let fact_requests: HashMap<String, Vec<super::FactRequest>> = vec![
            (
                "test_gat".to_owned(),
                vec![
                    super::FactRequest {
                        argument: "arg1".to_owned(),
                        check_id: "check1".to_owned(),
                        gatherer: "test_gat".to_owned(),
                        name: "fact1".to_owned(),
                    },
                    super::FactRequest {
                        argument: "arg2".to_owned(),
                        check_id: "check1".to_owned(),
                        gatherer: "test_gat".to_owned(),
                        name: "fact2".to_owned(),
                    },
                ],
            ),
            (
                "test_gat4".to_owned(),
                vec![super::FactRequest {
                    argument: "arg1".to_owned(),
                    check_id: "check1".to_owned(),
                    gatherer: "test_gat4".to_owned(),
                    name: "fact4".to_owned(),
                }],
            ),
            (
                "test_gat3".to_owned(),
                vec![super::FactRequest {
                    argument: "arg1".to_owned(),
                    check_id: "check1".to_owned(),
                    gatherer: "test_gat3".to_owned(),
                    name: "fact3".to_owned(),
                }],
            ),
        ]
        .into_iter()
        .collect();
        let expected_request = FactsGatheringRequest {
            execution_id: execution_id.to_owned(),
            group_id: group_id.to_owned(),
            facts_requests_by_gatherer: fact_requests,
        };

        let result = map_fact_gathering_request_from_event(targets.clone(), execution_id.to_owned(), group_id.to_owned());


        assert_eq!(result.execution_id, expected_request.execution_id);
        assert_eq!(result.group_id, expected_request.group_id);
        assert_eq!(result.facts_requests_by_gatherer.get("test_gat").unwrap(), expected_request.facts_requests_by_gatherer.get("test_gat").unwrap());
        assert_eq!(result.facts_requests_by_gatherer.get("test_gat3").unwrap(), expected_request.facts_requests_by_gatherer.get("test_gat3").unwrap());
        assert_eq!(result.facts_requests_by_gatherer.get("test_gat4").unwrap(), expected_request.facts_requests_by_gatherer.get("test_gat4").unwrap());

    }
}
