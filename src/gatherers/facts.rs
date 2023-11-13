use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FactGatheringErrors {
    // the errors
}

pub struct Fact {
    pub name: String,
    pub check_id: String,
    pub value: serde_json::value::Value,
    pub error: Option<FactGatheringErrors>,
}

pub struct FactsGathered {
    pub agent_id: String,
    pub exeuction_id: String,
    pub facts_gathered: Vec<Fact>,
    pub group_id: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FactRequest {
    pub argument: String,
    pub check_id: String,
    pub gatherer: String,
    pub name: String,
}

#[derive(PartialEq, Eq, Debug)]
pub struct FactsGatheringRequest {
    pub execution_id: String,
    pub group_id: String,
    pub facts_requests_by_gatherer: HashMap<String, Vec<FactRequest>>,
}
