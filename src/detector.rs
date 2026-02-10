use k8s_openapi::api::apps::v1::Deployment;
use crate::state::{DeploymentState, StateManager};

#[derive(Debug)]
pub enum DeploymentEvent {
    DeploymentStarted {
        namespace: String,
        name: String,
        old_generation: i64,
        new_generation: i64,
        old_replicas: Option<i32>,
        new_replicas: Option<i32>,
    },
    DeploymentCompleted {
        namespace: String,
        name: String,
        generation: i64,
        replicas: i32,
        replica_changed: Option<(i32, i32)>, // (old, new)
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
            // 초기 상태를 현재 값으로 설정하여 잘못된 완료 알림 방지
            current.last_completed_generation = current.generation;
            current.last_scaled_replicas = current.replicas;
        }
        Some(prev) => {
            // 변경 여부 확인
            let is_replica_change = current.replicas != prev.replicas;
            let is_generation_change = current.generation > prev.generation;
            let is_pod_template_change = current.pod_template_hash != prev.pod_template_hash;

            // 1. Deployment spec 변경 감지 (generation 증가)
            if is_generation_change {
                // Pod template이 변경되었으면 배포 이벤트, 아니면 replica 이벤트
                if is_pod_template_change {
                    // Pod template 변경 (이미지, 환경변수, 리소스 등) -> 배포 이벤트
                    let (old_replicas, new_replicas) = if is_replica_change {
                        (Some(prev.replicas), Some(current.replicas))
                    } else {
                        (None, None)
                    };

                    events.push(DeploymentEvent::DeploymentStarted {
                        namespace: namespace.clone(),
                        name: name.clone(),
                        old_generation: prev.generation,
                        new_generation: current.generation,
                        old_replicas,
                        new_replicas,
                    });

                    // replica 변경도 배포와 함께 처리됨
                    if is_replica_change {
                        current.last_scaled_replicas = current.replicas;
                    }
                } else {
                    // Pod template 동일하고 replica만 변경 -> replica 이벤트
                    events.push(DeploymentEvent::ReplicaScaleStarted {
                        namespace: namespace.clone(),
                        name: name.clone(),
                        old_replicas: prev.replicas,
                        new_replicas: current.replicas,
                    });
                    // replica만 변경된 경우 배포 완료 알림이 가지 않도록 generation 기록
                    current.last_completed_generation = current.generation;
                }
            } else if is_replica_change {
                // generation 변경 없이 replica만 변경 (이런 경우는 거의 없음)
                events.push(DeploymentEvent::ReplicaScaleStarted {
                    namespace: namespace.clone(),
                    name: name.clone(),
                    old_replicas: prev.replicas,
                    new_replicas: current.replicas,
                });
            }

            // 2. Deployment 완료 확인
            if is_deployment_complete(&current)
                && current.generation > current.last_completed_generation
            {
                // 배포 시작 시 replica가 변경되었는지 확인
                let replica_changed = if is_generation_change && is_replica_change {
                    Some((prev.replicas, current.replicas))
                } else {
                    None
                };

                events.push(DeploymentEvent::DeploymentCompleted {
                    namespace: namespace.clone(),
                    name: name.clone(),
                    generation: current.generation,
                    replicas: current.replicas,
                    replica_changed,
                });
                // 완료된 generation 기록
                current.last_completed_generation = current.generation;
            }

            // 3. Replica 변경 완료 확인
            if current.replicas != current.last_scaled_replicas
                && is_replicas_ready(&current)
                && current.observed_generation == current.generation
                && current.generation == current.last_completed_generation // 배포가 아닌 경우만
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

    // Pod template을 JSON으로 직렬화하여 해시 생성
    let pod_template_hash = spec
        .and_then(|s| serde_json::to_string(&s.template).ok())
        .unwrap_or_default();

    DeploymentState {
        namespace: metadata.namespace.clone().unwrap_or_default(),
        name: metadata.name.clone().unwrap_or_default(),
        generation: metadata.generation.unwrap_or(0),
        replicas: spec.and_then(|s| s.replicas).unwrap_or(1),
        ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
        available_replicas: status.and_then(|s| s.available_replicas).unwrap_or(0),
        updated_replicas: status.and_then(|s| s.updated_replicas).unwrap_or(0),
        observed_generation: status.and_then(|s| s.observed_generation).unwrap_or(0),
        pod_template_hash,
        last_completed_generation: 0, // 초기값, detect_changes에서 업데이트
        last_scaled_replicas: 0,      // 초기값, detect_changes에서 업데이트
    }
}
