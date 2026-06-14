use crate::features::launcher::fabric::FabricLaunchable;
use crate::features::launcher::{LaunchContext, LaunchResult, Launchable};
use async_trait::async_trait;

#[derive(Default)]
pub struct IrisFabricLaunchable {
    inner: FabricLaunchable,
}

#[async_trait]
impl Launchable for IrisFabricLaunchable {
    async fn prepare(&self, context: &LaunchContext) -> Result<(), String> {
        self.inner.prepare(context).await
    }

    async fn launch(&self, context: &LaunchContext) -> Result<LaunchResult, String> {
        self.inner.launch(context).await
    }
}
