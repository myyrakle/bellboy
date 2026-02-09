mod detector;
mod notifier;
mod state;

use std::env;

use detector::detect_changes;
use futures::TryStreamExt;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    Api, Client,
    runtime::{WatchStreamExt, watcher},
};
use notifier::notify;
use state::StateManager;

fn setup_logging() {
    unsafe {
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();
}

#[tokio::main]
async fn main() {
    setup_logging();

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
