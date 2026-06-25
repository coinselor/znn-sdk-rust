//! Accelerator embedded contract API.

use crate::abi::AbiValue;
use crate::api::PageQuery;
use crate::api::embedded::{dispatch, embedded_address, encode_call, nullable_array, page_params};
use crate::client::interfaces::Client;
use crate::client::websocket::WsClient;
use crate::embedded::constants::PROJECT_CREATION_FEE_IN_ZNN;
use crate::embedded::definitions::ACCELERATOR_DEFINITION;
use crate::error::Error;
use crate::model::embedded::accelerator::{Phase, Project, ProjectList};
use crate::model::embedded::common::{PillarVote, VoteBreakdown};
use crate::model::nom::account_block_template::AccountBlockTemplate;
use crate::primitives::address::Address;
use crate::primitives::hash::Hash;
use crate::primitives::token_standard::{TokenStandard, znn_token_standard};
use num_bigint::BigUint;
use serde_json::json;
use std::sync::Arc;

/// Accelerator API root.
pub struct AcceleratorApi<C: Client = WsClient> {
    client: Arc<C>,
}

impl<C: Client> AcceleratorApi<C> {
    /// Creates an accelerator API sharing `client`.
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }

    /// Returns all projects.
    pub async fn get_all(&self, page: PageQuery) -> Result<ProjectList, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.accelerator.getAll",
            &page_params(page),
        )
        .await?;
        ProjectList::from_json(&response)
    }

    /// Returns one project by id.
    pub async fn get_project_by_id(&self, id: &str) -> Result<Project, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.accelerator.getProjectById",
            &[json!(id)],
        )
        .await?;
        Project::from_json(&response)
    }

    /// Returns one phase by hash.
    pub async fn get_phase_by_hash(&self, hash: &Hash) -> Result<Phase, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.accelerator.getPhaseByHash",
            &[json!(hash.to_string())],
        )
        .await?;
        Phase::from_json(&response)
    }

    /// Returns pillar votes for hashes, preserving `null` entries.
    pub async fn get_pillar_votes(
        &self,
        name: &str,
        hashes: &[String],
    ) -> Result<Vec<Option<PillarVote>>, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.accelerator.getPillarVotes",
            &[json!(name), json!(hashes)],
        )
        .await?;
        nullable_array(&response, PillarVote::from_json)
    }

    /// Returns the vote breakdown.
    pub async fn get_vote_breakdown(&self, hash: &Hash) -> Result<VoteBreakdown, Error> {
        let response = dispatch(
            &*self.client,
            "embedded.accelerator.getVoteBreakdown",
            &[json!(hash.to_string())],
        )
        .await?;
        VoteBreakdown::from_json(&response)
    }

    /// Builds a create-project template.
    pub fn create_project(
        &self,
        name: &str,
        description: &str,
        url: &str,
        znn_funds: BigUint,
        qsr_funds: BigUint,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            ACCELERATOR_DEFINITION,
            "CreateProject",
            &[
                AbiValue::String(name.to_string()),
                AbiValue::String(description.to_string()),
                AbiValue::String(url.to_string()),
                AbiValue::UInt(znn_funds),
                AbiValue::UInt(qsr_funds),
            ],
        );
        AccountBlockTemplate::call_contract(
            accelerator_address(),
            znn_token_standard(),
            BigUint::from(PROJECT_CREATION_FEE_IN_ZNN),
            data,
        )
    }

    /// Builds an add-phase template.
    pub fn add_phase(
        &self,
        id: &Hash,
        name: &str,
        description: &str,
        url: &str,
        znn_funds: BigUint,
        qsr_funds: BigUint,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            ACCELERATOR_DEFINITION,
            "AddPhase",
            &[
                AbiValue::Hash(id.clone()),
                AbiValue::String(name.to_string()),
                AbiValue::String(description.to_string()),
                AbiValue::String(url.to_string()),
                AbiValue::UInt(znn_funds),
                AbiValue::UInt(qsr_funds),
            ],
        );
        AccountBlockTemplate::call_contract(
            accelerator_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds an update-phase template.
    pub fn update_phase(
        &self,
        id: &Hash,
        name: &str,
        description: &str,
        url: &str,
        znn_funds: BigUint,
        qsr_funds: BigUint,
    ) -> AccountBlockTemplate {
        let data = encode_call(
            ACCELERATOR_DEFINITION,
            "UpdatePhase",
            &[
                AbiValue::Hash(id.clone()),
                AbiValue::String(name.to_string()),
                AbiValue::String(description.to_string()),
                AbiValue::String(url.to_string()),
                AbiValue::UInt(znn_funds),
                AbiValue::UInt(qsr_funds),
            ],
        );
        AccountBlockTemplate::call_contract(
            accelerator_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a donation template.
    pub fn donate(&self, amount: BigUint, zts: TokenStandard) -> AccountBlockTemplate {
        let data = encode_call(ACCELERATOR_DEFINITION, "Donate", &[]);
        AccountBlockTemplate::call_contract(accelerator_address(), zts, amount, data)
    }

    /// Builds a vote-by-name template.
    pub fn vote_by_name(&self, name: &str, hash: &Hash, vote: u32) -> AccountBlockTemplate {
        let data = encode_call(
            ACCELERATOR_DEFINITION,
            "VoteByName",
            &[
                AbiValue::Hash(hash.clone()),
                AbiValue::String(name.to_string()),
                AbiValue::UInt(BigUint::from(vote)),
            ],
        );
        AccountBlockTemplate::call_contract(
            accelerator_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }

    /// Builds a vote-by-producer-address template.
    pub fn vote_by_prod_address(&self, hash: &Hash, vote: u32) -> AccountBlockTemplate {
        let data = encode_call(
            ACCELERATOR_DEFINITION,
            "VoteByProdAddress",
            &[
                AbiValue::Hash(hash.clone()),
                AbiValue::UInt(BigUint::from(vote)),
            ],
        );
        AccountBlockTemplate::call_contract(
            accelerator_address(),
            znn_token_standard(),
            BigUint::from(0u32),
            data,
        )
    }
}

/// Returns the accelerator contract address.
pub fn accelerator_address() -> Address {
    embedded_address("z1qxemdeddedxaccelerat0rxxxxxxxxxxp4tk22")
}
