use k8s_openapi::api::apps::v1::Deployment;
use crate::state::{DeploymentState, StateManager};

#[derive(Debug)]
pub enum DeploymentEvent {
    DeploymentStarted {
        namespace: String,
        name: String,
        old_generation: i64,
        new_generation: i64,
    },
    DeploymentCompleted {
        namespace: String,
        name: String,
        generation: i64,
        replicas: i32,
    },
    ReplicaScaleStarted {
        namespace: String,
        name: String,
        old_replicas: i32,
        new_replicas: i32,
    },
    ReplicaScaleCompleted {
        namespace: String,
        name: String,
        replicas: i32,
    },
}

pub async fn detect_changes(
    deployment: &Deployment,
    state_manager: &StateManager,
) -> Vec<DeploymentEvent> {
    let mut events = Vec::new();

    let namespace = deployment
        .metadata
        .namespace
        .as_deref()
        .unwrap_or("default")
        .to_string();
    let name = deployment
        .metadata
        .name
        .as_deref()
        .unwrap_or("unknown")
        .to_string();
    let key = format!("{}/{}", namespace, name);

    let mut current = extract_deployment_state(deployment);
    let previous = state_manager.get(&key).await;

    // 이전 상태에서 last_completed_generation, last_scaled_replicas 복사
    if let Some(ref prev) = previous {
        current.last_completed_generation = prev.last_completed_generation;
        current.last_scaled_replicas = prev.last_scaled_replicas;
    }

    match previous {
        None => {
            // 처음 발견한 Deployment - 상태만 저장하고 이벤트 발생 안 함
        }
        Some(prev) => {
            // 1. Deployment spec 변경 감지 (generation 증가)
            if current.generation > prev.generation {
                events.push(DeploymentEvent::DeploymentStarted {
                    namespace: namespace.clone(),
                    name: name.clone(),
                    old_generation: prev.generation,
                    new_generation: current.generation,
                });
            }

            // 2. Deployment 완료 확인
            // 배포가 완료되었고, 이 generation에 대해 아직 완료 이벤트를 발생시키지 않은 경우
            if is_deployment_complete(&current)
                && current.generation > current.last_completed_generation
            {
                events.push(DeploymentEvent::DeploymentCompleted {
                    namespace: namespace.clone(),
                    name: name.clone(),
                    generation: current.generation,
                    replicas: current.replicas,
                });
                // 완료된 generation 기록
                current.last_completed_generation = current.generation;
            }

            // 3. Replica 수 변경 감지
            if current.replicas != prev.replicas {
                events.push(DeploymentEvent::ReplicaScaleStarted {
                    namespace: namespace.clone(),
                    name: name.clone(),
                    old_replicas: prev.replicas,
                    new_replicas: current.replicas,
                });
            }

            // 4. Replica 변경 완료 확인
            // 이 replicas 수에 대해 아직 완료 이벤트를 발생시키지 않았고,
            // 모든 pods가 준비되었고, 배포가 진행 중이 아닌 경우
            if current.replicas != current.last_scaled_replicas
                && is_replicas_ready(&current)
                && current.observed_generation == current.generation
            {
                events.push(DeploymentEvent::ReplicaScaleCompleted {
                    namespace: namespace.clone(),
                    name: name.clone(),
                    replicas: current.replicas,
                });
                // 완료된 replicas 기록
                current.last_scaled_replicas = current.replicas;
            }
        }
    }

    // 상태 업데이트
    state_manager.update(key, current).await;

    events
}

fn is_deployment_complete(state: &DeploymentState) -> bool {
    state.observed_generation == state.generation
        && state.ready_replicas == state.replicas
        && state.available_replicas == state.replicas
        && state.updated_replicas == state.replicas
}

fn is_replicas_ready(state: &DeploymentState) -> bool {
    state.ready_replicas == state.replicas && state.available_replicas == state.replicas
}

fn extract_deployment_state(deployment: &Deployment) -> DeploymentState {
    let metadata = &deployment.metadata;
    let spec = deployment.spec.as_ref();
    let status = deployment.status.as_ref();

    DeploymentState {
        namespace: metadata.namespace.clone().unwrap_or_default(),
        name: metadata.name.clone().unwrap_or_default(),
        generation: metadata.generation.unwrap_or(0),
        replicas: spec.and_then(|s| s.replicas).unwrap_or(1),
        ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
        available_replicas: status.and_then(|s| s.available_replicas).unwrap_or(0),
        updated_replicas: status.and_then(|s| s.updated_replicas).unwrap_or(0),
        observed_generation: status.and_then(|s| s.observed_generation).unwrap_or(0),
        last_completed_generation: 0, // 초기값, detect_changes에서 업데이트
        last_scaled_replicas: 0,      // 초기값, detect_changes에서 업데이트
    }
}
