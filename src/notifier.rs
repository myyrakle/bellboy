use crate::detector::DeploymentEvent;
use serde::Serialize;

#[derive(Clone, Copy, Debug)]
pub enum Language {
    Korean,
    English,
}

impl Language {
    pub fn from_env(lang_code: &str) -> Self {
        match lang_code.to_lowercase().as_str() {
            "en" | "english" => Language::English,
            _ => Language::Korean, // 기본값: 한국어
        }
    }
}

#[derive(Clone)]
pub struct NotifierConfig {
    pub language: Language,
    pub slack_token: Option<String>,
    pub slack_channel: Option<String>,
}

impl NotifierConfig {
    pub fn from_env() -> Self {
        let language = std::env::var("LANGUAGE")
            .ok()
            .map(|s| Language::from_env(&s))
            .unwrap_or(Language::Korean);

        let slack_token = std::env::var("SLACK_TOKEN").ok();
        let slack_channel = std::env::var("SLACK_CHANNEL").ok();

        Self {
            language,
            slack_token,
            slack_channel,
        }
    }

    pub fn has_slack_config(&self) -> bool {
        self.slack_token.is_some() && self.slack_channel.is_some()
    }
}

fn format_message(event: &DeploymentEvent, language: Language) -> String {
    match (event, language) {
        (
            DeploymentEvent::DeploymentStarted {
                namespace,
                name,
                old_generation,
                new_generation,
            },
            Language::Korean,
        ) => {
            format!(
                "[배포 시작] {}/{}: 배포가 시작됩니다 (revision: {} -> {})",
                namespace, name, old_generation, new_generation
            )
        }
        (
            DeploymentEvent::DeploymentStarted {
                namespace,
                name,
                old_generation,
                new_generation,
            },
            Language::English,
        ) => {
            format!(
                "[Deploy Started] {}/{}: Deployment started (revision: {} -> {})",
                namespace, name, old_generation, new_generation
            )
        }

        (
            DeploymentEvent::DeploymentCompleted {
                namespace,
                name,
                generation,
                replicas,
            },
            Language::Korean,
        ) => {
            format!(
                "[배포 완료] {}/{}: 배포가 완료되었습니다 (revision: {}, replicas: {})",
                namespace, name, generation, replicas
            )
        }
        (
            DeploymentEvent::DeploymentCompleted {
                namespace,
                name,
                generation,
                replicas,
            },
            Language::English,
        ) => {
            format!(
                "[Deploy Completed] {}/{}: Deployment completed (revision: {}, replicas: {})",
                namespace, name, generation, replicas
            )
        }

        (
            DeploymentEvent::ReplicaScaleStarted {
                namespace,
                name,
                old_replicas,
                new_replicas,
            },
            Language::Korean,
        ) => {
            format!(
                "[스케일 시작] {}/{}: replica 수정 {}->{}가 시작됩니다",
                namespace, name, old_replicas, new_replicas
            )
        }
        (
            DeploymentEvent::ReplicaScaleStarted {
                namespace,
                name,
                old_replicas,
                new_replicas,
            },
            Language::English,
        ) => {
            format!(
                "[Scale Started] {}/{}: Scaling from {} to {} replicas",
                namespace, name, old_replicas, new_replicas
            )
        }

        (
            DeploymentEvent::ReplicaScaleCompleted {
                namespace,
                name,
                replicas,
            },
            Language::Korean,
        ) => {
            format!(
                "[스케일 완료] {}/{}: replica 수정이 완료되었습니다 (replicas: {})",
                namespace, name, replicas
            )
        }
        (
            DeploymentEvent::ReplicaScaleCompleted {
                namespace,
                name,
                replicas,
            },
            Language::English,
        ) => {
            format!(
                "[Scale Completed] {}/{}: Scaling completed (replicas: {})",
                namespace, name, replicas
            )
        }
    }
}

#[derive(Serialize)]
struct SlackMessage {
    channel: String,
    text: String,
}

async fn send_to_slack(
    message: &str,
    token: &str,
    channel: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let slack_message = SlackMessage {
        channel: channel.to_string(),
        text: message.to_string(),
    };

    let response = client
        .post("https://slack.com/api/chat.postMessage")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&slack_message)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("Slack API error: {}", error_text).into());
    }

    Ok(())
}

pub async fn notify(event: DeploymentEvent, config: &NotifierConfig) {
    let message = format_message(&event, config.language);

    // stdout 출력
    log::info!("{}", message);

    // Slack 전송
    if config.has_slack_config() {
        if let (Some(token), Some(channel)) = (&config.slack_token, &config.slack_channel) {
            if let Err(e) = send_to_slack(&message, token, channel).await {
                eprintln!("Failed to send Slack notification: {}", e);
            }
        }
    }
}
