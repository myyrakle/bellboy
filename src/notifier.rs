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
                old_replicas,
                new_replicas,
            },
            Language::Korean,
        ) => {
            let replica_info = match (old_replicas, new_replicas) {
                (Some(old), Some(new)) => format!(" (replicas: {} → {})", old, new),
                _ => String::new(),
            };
            format!(
                "🚀 [배포 시작] {}/{}: 배포가 시작됩니다 (revision: {} -> {}){}",
                namespace, name, old_generation, new_generation, replica_info
            )
        }
        (
            DeploymentEvent::DeploymentStarted {
                namespace,
                name,
                old_generation,
                new_generation,
                old_replicas,
                new_replicas,
            },
            Language::English,
        ) => {
            let replica_info = match (old_replicas, new_replicas) {
                (Some(old), Some(new)) => format!(" (replicas: {} → {})", old, new),
                _ => String::new(),
            };
            format!(
                "🚀 [Deploy Started] {}/{}: Deployment started (revision: {} -> {}){}",
                namespace, name, old_generation, new_generation, replica_info
            )
        }

        (
            DeploymentEvent::DeploymentCompleted {
                namespace,
                name,
                generation,
                replicas,
                replica_changed,
            },
            Language::Korean,
        ) => {
            let replica_info = match replica_changed {
                Some((old, new)) => format!("replicas: {} → {}", old, new),
                None => format!("replicas: {}", replicas),
            };
            format!(
                "✅ [배포 완료] {}/{}: 배포가 완료되었습니다 (revision: {}, {})",
                namespace, name, generation, replica_info
            )
        }
        (
            DeploymentEvent::DeploymentCompleted {
                namespace,
                name,
                generation,
                replicas,
                replica_changed,
            },
            Language::English,
        ) => {
            let replica_info = match replica_changed {
                Some((old, new)) => format!("replicas: {} → {}", old, new),
                None => format!("replicas: {}", replicas),
            };
            format!(
                "✅ [Deploy Completed] {}/{}: Deployment completed (revision: {}, {})",
                namespace, name, generation, replica_info
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
            if new_replicas > old_replicas {
                format!(
                    "📈 [스케일 업] {}/{}: {} → {} replicas 증가",
                    namespace, name, old_replicas, new_replicas
                )
            } else {
                format!(
                    "📉 [스케일 다운] {}/{}: {} → {} replicas 감소",
                    namespace, name, old_replicas, new_replicas
                )
            }
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
            if new_replicas > old_replicas {
                format!(
                    "📈 [Scale Up] {}/{}: {} → {} replicas",
                    namespace, name, old_replicas, new_replicas
                )
            } else {
                format!(
                    "📉 [Scale Down] {}/{}: {} → {} replicas",
                    namespace, name, old_replicas, new_replicas
                )
            }
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
                "✅ [스케일 완료] {}/{}: replica 수정이 완료되었습니다 (replicas: {})",
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
                "✅ [Scale Completed] {}/{}: Scaling completed (replicas: {})",
                namespace, name, replicas
            )
        }
    }
}

#[derive(Serialize)]
struct SlackMessage {
    channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<SlackAttachment>>,
}

#[derive(Serialize)]
struct SlackAttachment {
    color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<SlackField>>,
}

#[derive(Serialize)]
struct SlackField {
    title: String,
    value: String,
    short: bool,
}

fn create_slack_attachment(event: &DeploymentEvent, language: Language) -> SlackAttachment {
    let color = match event {
        DeploymentEvent::DeploymentStarted { .. } => "warning".to_string(),
        DeploymentEvent::DeploymentCompleted { .. } => "good".to_string(),
        DeploymentEvent::ReplicaScaleStarted { .. } => "warning".to_string(),
        DeploymentEvent::ReplicaScaleCompleted { .. } => "good".to_string(),
    };

    let (title, fields) = match event {
        DeploymentEvent::DeploymentStarted {
            namespace,
            name,
            old_generation,
            new_generation,
            old_replicas,
            new_replicas,
        } => {
            let title = match language {
                Language::Korean => "🚀 배포 시작",
                Language::English => "🚀 Deploy Started",
            };
            let mut fields = vec![
                SlackField {
                    title: "Deployment".to_string(),
                    value: format!("{}/{}", namespace, name),
                    short: true,
                },
                SlackField {
                    title: "Revision".to_string(),
                    value: format!("{} → {}", old_generation, new_generation),
                    short: true,
                },
            ];

            if let (Some(old), Some(new)) = (old_replicas, new_replicas) {
                fields.push(SlackField {
                    title: "Replicas".to_string(),
                    value: format!("{} → {}", old, new),
                    short: true,
                });
            }

            (title, fields)
        }
        DeploymentEvent::DeploymentCompleted {
            namespace,
            name,
            generation,
            replicas,
            replica_changed,
        } => {
            let title = match language {
                Language::Korean => "✅ 배포 완료",
                Language::English => "✅ Deploy Completed",
            };
            let replica_value = match replica_changed {
                Some((old, new)) => format!("{} → {}", old, new),
                None => replicas.to_string(),
            };
            let fields = vec![
                SlackField {
                    title: "Deployment".to_string(),
                    value: format!("{}/{}", namespace, name),
                    short: true,
                },
                SlackField {
                    title: "Revision".to_string(),
                    value: generation.to_string(),
                    short: true,
                },
                SlackField {
                    title: "Replicas".to_string(),
                    value: replica_value,
                    short: true,
                },
            ];
            (title, fields)
        }
        DeploymentEvent::ReplicaScaleStarted {
            namespace,
            name,
            old_replicas,
            new_replicas,
        } => {
            let title = if new_replicas > old_replicas {
                match language {
                    Language::Korean => "📈 스케일 업",
                    Language::English => "📈 Scale Up",
                }
            } else {
                match language {
                    Language::Korean => "📉 스케일 다운",
                    Language::English => "📉 Scale Down",
                }
            };
            let fields = vec![
                SlackField {
                    title: "Deployment".to_string(),
                    value: format!("{}/{}", namespace, name),
                    short: true,
                },
                SlackField {
                    title: "Replicas".to_string(),
                    value: format!("{} → {}", old_replicas, new_replicas),
                    short: true,
                },
            ];
            (title, fields)
        }
        DeploymentEvent::ReplicaScaleCompleted {
            namespace,
            name,
            replicas,
        } => {
            let title = match language {
                Language::Korean => "✅ 스케일 완료",
                Language::English => "✅ Scale Completed",
            };
            let fields = vec![
                SlackField {
                    title: "Deployment".to_string(),
                    value: format!("{}/{}", namespace, name),
                    short: true,
                },
                SlackField {
                    title: "Replicas".to_string(),
                    value: replicas.to_string(),
                    short: true,
                },
            ];
            (title, fields)
        }
    };

    SlackAttachment {
        color,
        text: Some(title.to_string()),
        fields: Some(fields),
    }
}

async fn send_to_slack(
    event: &DeploymentEvent,
    language: Language,
    token: &str,
    channel: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let attachment = create_slack_attachment(event, language);

    let slack_message = SlackMessage {
        channel: channel.to_string(),
        text: None,
        attachments: Some(vec![attachment]),
    };

    let response = client
        .post("https://slack.com/api/chat.postMessage")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&slack_message)
        .send()
        .await?;

    // Slack API 응답 확인
    let response_text = response.text().await?;

    // Slack API는 200을 반환하지만 error 필드로 에러를 표시할 수 있음
    let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
    if let Some(ok) = response_json.get("ok") {
        if ok == &serde_json::Value::Bool(false) {
            if let Some(error) = response_json.get("error") {
                return Err(format!("Slack API error: {}", error).into());
            }
        }
    }

    Ok(())
}

pub async fn notify(event: DeploymentEvent, config: &NotifierConfig) {
    let message = format_message(&event, config.language);

    // stdout 출력
    log::info!("{}", message);

    // Slack 전송 (Block Kit 사용)
    if config.has_slack_config() {
        if let (Some(token), Some(channel)) = (&config.slack_token, &config.slack_channel) {
            if let Err(e) = send_to_slack(&event, config.language, token, channel).await {
                eprintln!("Failed to send Slack notification: {}", e);
            }
        }
    }
}
