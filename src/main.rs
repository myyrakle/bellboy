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
use notifier::{NotifierConfig, notify};
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

    // NAMESPACE 환경 변수로 특정 네임스페이스만 watch 가능
    let deployments: Api<Deployment> = match env::var("NAMESPACE") {
        Ok(namespace) => {
            log::info!("Watching namespace: {}", namespace);
            Api::namespaced(client, &namespace)
        }
        Err(_) => {
            log::info!("Watching all namespaces");
            Api::all(client)
        }
    };
    let state_manager = StateManager::new();
    let notifier_config = NotifierConfig::from_env();

    // Slack 설정 확인 및 로깅
    if notifier_config.has_slack_config() {
        log::info!("Slack notification enabled");
        log::info!("Language: {:?}", notifier_config.language);
    } else {
        log::info!("Slack notification disabled (set SLACK_TOKEN and SLACK_CHANNEL to enable)");
    }

    watcher(deployments, Default::default())
        .applied_objects()
        .try_for_each(|deployment| {
            let state_manager = state_manager.clone();
            let notifier_config = notifier_config.clone();
            async move {
                let events = detect_changes(&deployment, &state_manager).await;

                for event in events {
                    notify(event, &notifier_config).await;
                }

                Ok(())
            }
        })
        .await
        .expect("watch failed");
}
