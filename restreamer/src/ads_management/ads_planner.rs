use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use chrono::{DateTime, Utc};
use codec::{AudioFrame, CodecParams};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{AdId, AdsProvider, ContentItem};

#[derive(Debug, Clone, Copy)]
struct ActiveItem {
    id: AdId,
    started: DateTime<Utc>,
}

pub struct AdsPlanner {
    client_id: Uuid,
    ads_provider: Arc<AdsProvider>,
    codec_params: CodecParams,
    plan: Vec<AdId>,
    cursor: AtomicUsize,
    active_item: Arc<RwLock<Option<ActiveItem>>>,
}

impl AdsPlanner {
    pub async fn new(
        ads_provider: Arc<AdsProvider>,
        codec_params: CodecParams,
    ) -> anyhow::Result<Self> {
        let content = ads_provider.content().await?;

        let plan = arrange_plan(content);

        Ok(Self {
            client_id: Uuid::new_v4(),
            ads_provider,
            codec_params,
            plan,
            cursor: AtomicUsize::default(),
            active_item: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn next(&self) -> anyhow::Result<Vec<AudioFrame>> {
        if self.active_item.read().await.is_some() {
            log::error!(
                "Client {} Track is not completed: {:?}",
                self.client_id,
                self.active_item
            );
        }

        let active_item = self.cursor.fetch_add(1, Ordering::Relaxed) % self.plan.len();
        assert!(active_item < self.plan.len());
        let active_id = self.plan[active_item];

        *self.active_item.write().await = Some(ActiveItem {
            id: active_id,
            started: Utc::now(),
        });

        self.ads_provider
            .report_started(self.client_id, active_id)
            .await?;

        Ok((*self
            .ads_provider
            .get(active_id, self.codec_params)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Client {}: failed to obtain track {active_id}",
                    self.client_id,
                )
            })?)
        .clone())
    }

    pub async fn finished(&self) {
        let active_item = self.active_item.write().await.take();

        if let Some(item) = active_item {
            if let Err(err) = self
                .ads_provider
                .report_finished(self.client_id, item.id, item.started)
                .await
            {
                log::error!(
                    "Client {}: failed to report finished: {err:#}",
                    self.client_id,
                );
            }
        }
    }
}

fn arrange_plan(content: Vec<ContentItem>) -> Vec<AdId> {
    assert!(!content.is_empty());
    dbg!(&content);
    content.into_iter().map(|item| item.id).collect()
}

#[cfg(test)]
impl AdsPlanner {
    pub async fn testing(track: Vec<AudioFrame>) -> Self {
        let ads_provider = Arc::new(AdsProvider::testing(track).await);
        Self::new(ads_provider, super::CODEC_PARAMS).await.unwrap()
    }
}
