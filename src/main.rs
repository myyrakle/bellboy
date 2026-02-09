use futures::TryStreamExt;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    Api, Client,
    runtime::{WatchStreamExt, watcher},
};

#[tokio::main]
async fn main() {
    let client = Client::try_default()
        .await
        .expect("Failed to create client");

    let deployments: Api<Deployment> = Api::all(client);

    watcher(deployments, Default::default())
        .applied_objects()
        .try_for_each(|deployment| async move {
            // 이미지 변경, replica 변경 감지 후 알림
            println!(
                "Deployment applied: {} with {} replicas",
                deployment.metadata.name.unwrap_or_default(),
                deployment
                    .spec
                    .as_ref()
                    .and_then(|spec| spec.replicas)
                    .unwrap_or(0)
            );

            Ok(())
        })
        .await
        .expect("watch failed");
}
