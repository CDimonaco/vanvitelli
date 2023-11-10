#[cfg(test)]
use mockall::automock;

mod facts;
mod registry;
pub(crate) use facts::*;

#[async_trait::async_trait]
#[cfg_attr(test, automock)]
pub trait Gatherer: Sync + Send {
    async fn gather(&self, fact_request: FactsGatheringRequest) -> FactsGathered;
    fn name(&self) -> String;
}
