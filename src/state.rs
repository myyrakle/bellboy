use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct DeploymentState {
    pub namespace: String,
    pub name: String,
    pub generation: i64,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub updated_replicas: i32,
    pub observed_generation: i64,
    pub last_completed_generation: i64, // 마지막으로 완료 이벤트를 발생시킨 generation
    pub last_scaled_replicas: i32,      // 마지막으로 스케일 완료 이벤트를 발생시킨 replicas
}

#[derive(Clone)]
pub struct StateManager {
    states: Arc<Mutex<HashMap<String, DeploymentState>>>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<DeploymentState> {
        let states = self.states.lock().await;
        states.get(key).cloned()
    }

    pub async fn update(&self, key: String, state: DeploymentState) {
        let mut states = self.states.lock().await;
        states.insert(key, state);
    }

    #[allow(dead_code)]
    pub async fn remove(&self, key: &str) {
        let mut states = self.states.lock().await;
        states.remove(key);
    }
}
