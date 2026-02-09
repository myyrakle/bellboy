use crate::detector::DeploymentEvent;

pub fn notify(event: DeploymentEvent) {
    match event {
        DeploymentEvent::DeploymentStarted {
            namespace,
            name,
            old_generation,
            new_generation,
        } => {
            println!(
                "[배포 시작] {}/{}: 배포가 시작됩니다 (generation: {} -> {})",
                namespace, name, old_generation, new_generation
            );
        }
        DeploymentEvent::DeploymentCompleted {
            namespace,
            name,
            generation,
            replicas,
        } => {
            println!(
                "[배포 완료] {}/{}: 배포가 완료되었습니다 (generation: {}, replicas: {})",
                namespace, name, generation, replicas
            );
        }
        DeploymentEvent::ReplicaScaleStarted {
            namespace,
            name,
            old_replicas,
            new_replicas,
        } => {
            println!(
                "[스케일 시작] {}/{}: replica 수정 {}->{}가 시작됩니다",
                namespace, name, old_replicas, new_replicas
            );
        }
        DeploymentEvent::ReplicaScaleCompleted {
            namespace,
            name,
            replicas,
        } => {
            println!(
                "[스케일 완료] {}/{}: replica 수정이 완료되었습니다 (replicas: {})",
                namespace, name, replicas
            );
        }
    }
}
