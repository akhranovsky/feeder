use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use codec::{AudioFrame, CodecParams};

use super::{AdId, AdsProvider};

pub struct AdsPlanner {
    ads_provider: Arc<AdsProvider>,
    codec_params: CodecParams,
    plan: Vec<AdId>,
    next_item: AtomicUsize,
}

impl AdsPlanner {
    pub async fn new(
        ads_provider: Arc<AdsProvider>,
        codec_params: CodecParams,
    ) -> anyhow::Result<Self> {
        let content = ads_provider.content().await?;

        let plan = arrange_plan(content);

        Ok(Self {
            ads_provider,
            codec_params,
            plan,
            next_item: AtomicUsize::default(),
        })
    }

    pub async fn next(&self) -> anyhow::Result<Vec<AudioFrame>> {
        let next_item = self.next_item.fetch_add(1, Ordering::Relaxed) % self.plan.len();
        assert!(next_item < self.plan.len());
        let next_id = self.plan[next_item];

        Ok((*self
            .ads_provider
            .get(next_id, self.codec_params)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No track"))?)
        .clone())
    }
}

fn arrange_plan(content: Vec<(AdId, String)>) -> Vec<AdId> {
    assert!(!content.is_empty());
    dbg!(&content);
    content.into_iter().map(|(id, _)| id).collect()
}

#[cfg(test)]
impl AdsPlanner {
    pub async fn testing(track: Vec<AudioFrame>) -> Self {
        let ads_provider = Arc::new(AdsProvider::testing(track).await);
        Self::new(ads_provider, super::CODEC_PARAMS).await.unwrap()
    }
}
