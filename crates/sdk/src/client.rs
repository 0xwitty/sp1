use crate::{
    local::{LocalProver, LocalProverBuilder},
    opts::ProofOpts,
    proof::SP1ProofWithPublicValues,
    prover::Prover,
    request::DynProofRequest,
    SP1VerificationError,
};

#[cfg(feature = "network-v2")]
use crate::network_v2::{NetworkProver, NetworkProverBuilder, DEFAULT_PROVER_NETWORK_RPC};

use anyhow::Result;
use sp1_core_executor::{ExecutionError, ExecutionReport};
use sp1_core_machine::io::SP1Stdin;
use sp1_primitives::io::SP1PublicValues;
use sp1_prover::{SP1ProvingKey, SP1VerifyingKey};
use std::{env, sync::Arc};

pub struct None;

pub struct ProverClient {
    inner: Box<dyn Prover>,
}

pub struct ProverClientBuilder<T> {
    inner_builder: T,
}

#[allow(clippy::new_without_default)]
impl ProverClient {
    pub fn builder() -> ProverClientBuilder<None> {
        ProverClientBuilder { inner_builder: None }
    }

    #[deprecated(note = "Use ProverClient::builder() instead")]
    pub fn new() -> Self {
        Self::create_from_env()
    }

    fn create_from_env() -> Self {
        #[cfg(feature = "network-v2")]
        match env::var("SP1_PROVER").unwrap_or("local".to_string()).as_str() {
            "network" => {
                let rpc_url = env::var("PROVER_NETWORK_RPC")
                    .unwrap_or_else(|_| DEFAULT_PROVER_NETWORK_RPC.to_string());
                let private_key = env::var("SP1_PRIVATE_KEY").unwrap_or_default();

                let network_prover = NetworkProver::new(rpc_url, private_key);
                ProverClient { inner: Box::new(network_prover) }
            }
            _ => {
                let local_prover = LocalProver::new();
                ProverClient { inner: Box::new(local_prover) }
            }
        }

        #[cfg(not(feature = "network-v2"))]
        {
            let local_prover = LocalProver::new();
            ProverClient { inner: Box::new(local_prover) }
        }
    }

    pub async fn setup(&self, elf: Arc<[u8]>) -> Arc<SP1ProvingKey> {
        self.inner.setup(elf).await
    }

    pub async fn execute(
        &self,
        elf: Arc<[u8]>,
        stdin: SP1Stdin,
    ) -> Result<(SP1PublicValues, ExecutionReport), ExecutionError> {
        self.inner.execute(elf, stdin).await
    }

    pub fn prove<'a>(&'a self, pk: &'a Arc<SP1ProvingKey>, stdin: SP1Stdin) -> DynProofRequest<'a> {
        DynProofRequest::new(&*self.inner, pk, stdin, ProofOpts::default())
    }

    pub async fn verify(
        &self,
        proof: Arc<SP1ProofWithPublicValues>,
        vk: Arc<SP1VerifyingKey>,
    ) -> Result<(), SP1VerificationError> {
        self.inner.verify(proof, vk).await
    }
}

impl ProverClientBuilder<None> {
    pub fn local(self) -> ProverClientBuilder<LocalProverBuilder> {
        ProverClientBuilder { inner_builder: LocalProver::builder() }
    }
    
    #[cfg(feature = "network-v2")]
    pub fn network(self) -> ProverClientBuilder<NetworkProverBuilder> {
        ProverClientBuilder { inner_builder: NetworkProver::builder() }
    }

    pub fn from_env(self) -> ProverClient {
        ProverClient::create_from_env()
    }
}

impl<T: BuildableProver> ProverClientBuilder<T> {
    pub fn build(self) -> ProverClient {
        ProverClient { inner: self.inner_builder.build_prover() }
    }
}

#[cfg(feature = "network-v2")]
impl ProverClientBuilder<NetworkProverBuilder> {
    pub fn rpc_url(mut self, url: String) -> Self {
        self.inner_builder = self.inner_builder.rpc_url(url);
        self
    }

    pub fn private_key(mut self, key: String) -> Self {
        self.inner_builder = self.inner_builder.private_key(key);
        self
    }
}

pub trait BuildableProver {
    fn build_prover(self) -> Box<dyn Prover>;
}

impl BuildableProver for LocalProverBuilder {
    fn build_prover(self) -> Box<dyn Prover> {
        Box::new(self.build())
    }
}

#[cfg(feature = "network-v2")]
impl BuildableProver for NetworkProverBuilder {
    fn build_prover(self) -> Box<dyn Prover> {
        Box::new(self.build())
    }
}
