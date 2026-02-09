mod detector;
mod notifier;
mod state;

use detector::detect_changes;
use futures::TryStreamExt;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    runtime::{watcher, WatchStreamExt},
    Api, Client,
};
use notifier::notify;
use state::StateManager;

#[tokio::main]
async fn main() {
    let client = Client::try_default()
        .await
        .expect("Failed to create client");

    let deployments: Api<Deployment> = Api::all(client);
    let state_manager = StateManager::new();

    watcher(deployments, Default::default())
        .applied_objects()
        .try_for_each(|deployment| {
            let state_manager = state_manager.clone();
            async move {
                let events = detect_changes(&deployment, &state_manager).await;

                for event in events {
                    notify(event);
                }

                Ok(())
            }
        })
        .await
        .expect("watch failed");
}
