//! Test Utilities for the [crate::pipeline::DerivationPipeline]
//! as well as its stages and providers.

use alloc::{boxed::Box, sync::Arc};
use op_alloy_genesis::{RollupConfig, SystemConfig};
use op_alloy_protocol::{BlockInfo, L2BlockInfo};
use op_alloy_rpc_types_engine::OpAttributesWithParent;

use crate::stages::ChannelProvider;
// Re-export these types used internally to the test pipeline.
pub use crate::{
    batch::SingleBatch,
    errors::PipelineError,
    pipeline::{DerivationPipeline, PipelineBuilder, PipelineResult},
    stages::{
        AttributesProvider, AttributesQueue, BatchQueue, BatchStream, ChannelBank, ChannelReader,
        FrameQueue, L1Retrieval, L1Traversal,
    },
    test_utils::TestAttributesBuilder,
    traits::{
        test_utils::TestDAP, FlushableStage, NextAttributes, OriginAdvancer, OriginProvider,
        ResettableStage,
    },
};
pub use kona_providers::test_utils::{TestChainProvider, TestL2ChainProvider};

/// A fully custom [NextAttributes].
#[derive(Default, Debug, Clone)]
pub struct TestNextAttributes {
    /// The next [OpAttributesWithParent] to return.
    pub next_attributes: Option<OpAttributesWithParent>,
}

#[async_trait::async_trait]
impl FlushableStage for TestNextAttributes {
    /// Flushes the stage.
    async fn flush_channel(&mut self) -> PipelineResult<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl ResettableStage for TestNextAttributes {
    /// Resets the derivation stage to its initial state.
    async fn reset(&mut self, _: BlockInfo, _: &SystemConfig) -> PipelineResult<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl OriginProvider for TestNextAttributes {
    /// Returns the current origin.
    fn origin(&self) -> Option<BlockInfo> {
        Some(BlockInfo::default())
    }
}

#[async_trait::async_trait]
impl OriginAdvancer for TestNextAttributes {
    /// Advances the origin to the given block.
    async fn advance_origin(&mut self) -> PipelineResult<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl NextAttributes for TestNextAttributes {
    /// Returns the next valid [OpAttributesWithParent].
    async fn next_attributes(&mut self, _: L2BlockInfo) -> PipelineResult<OpAttributesWithParent> {
        self.next_attributes.take().ok_or(PipelineError::Eof.temp())
    }
}

/// An [L1Traversal] using test providers and sources.
pub type TestL1Traversal = L1Traversal<TestChainProvider>;

/// An [L1Retrieval] stage using test providers and sources.
pub type TestL1Retrieval = L1Retrieval<TestDAP, TestL1Traversal>;

/// A [FrameQueue] using test providers and sources.
pub type TestFrameQueue = FrameQueue<TestL1Retrieval>;

/// A [ChannelBank] using test providers and sources.
pub type TestChannelProvider = ChannelProvider<TestFrameQueue>;

/// A [ChannelReader] using test providers and sources.
pub type TestChannelReader = ChannelReader<TestChannelProvider>;

/// A [BatchStream] using test providers and sources.
pub type TestBatchStream = BatchStream<TestChannelReader, TestL2ChainProvider>;

/// A [BatchQueue] using test providers and sources.
pub type TestBatchQueue = BatchQueue<TestBatchStream, TestL2ChainProvider>;

/// An [AttributesQueue] using test providers and sources.
pub type TestAttributesQueue = AttributesQueue<TestBatchQueue, TestAttributesBuilder>;

/// A [DerivationPipeline] using test providers and sources.
pub type TestPipeline = DerivationPipeline<TestAttributesQueue, TestL2ChainProvider>;

/// Constructs a [DerivationPipeline] using test providers and sources.
pub fn new_test_pipeline() -> TestPipeline {
    PipelineBuilder::new()
        .rollup_config(Arc::new(RollupConfig::default()))
        .origin(BlockInfo::default())
        .dap_source(TestDAP::default())
        .builder(TestAttributesBuilder::default())
        .chain_provider(TestChainProvider::default())
        .l2_chain_provider(TestL2ChainProvider::default())
        .build()
}
