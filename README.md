# bellboy

![](https://img.shields.io/badge/language-Rust-red) ![](https://img.shields.io/badge/version-0.2.0-brightgreen) [![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/myyrakle/bellboy/blob/master/LICENSE)

- A fast and simple deployment notification system
- Automatically sends Deployment completion notifications and Scale adjustment notifications to Slack.

## Details

- It supports only minimal features. It's intended for detecting "normal deployments.". not failures.
- If you require notifications of failures, we recommend Prometheus AlertManager.
- Languages ​​supported include Korean and English.

## Setup

It can be installed via helm.

```bash
helm repo add bellboy https://myyrakle.github.io/bellboy/
helm install bellboy bellboy/bellboy --set slack.token="SLACK TOKEN" --set slack.channel="CHANNEL ID"
```

### Configuration Options

- `slack.token`: Slack Bot Token (required)
- `slack.channel`: Slack Channel ID (required)
- `language`: Language for notifications (`ko` or `en`, default: `ko`)
- `watchNamespace`: Specific namespace to watch (empty = watch all namespaces)
- `logLevel`: Log level (default: `info`)

### Examples

Watch all namespaces:

```bash
helm install bellboy bellboy/bellboy \
  --set slack.token="xoxb-..." \
  --set slack.channel="C...."
```

Watch specific namespace only:

```bash
helm install bellboy bellboy/bellboy \
  --set slack.token="xoxb-..." \
  --set slack.channel="C...." \
  --set watchNamespace="production"
```

Use English language:

```bash
helm install bellboy bellboy/bellboy \
  --set slack.token="xoxb-..." \
  --set slack.channel="C...." \
  --set language="en"
```

Then, whenever the Deployment changes, a message like the following will be sent:
<img width="485" height="214" alt="image" src="https://github.com/user-attachments/assets/6a50573c-3270-4096-9b41-d446405b5297" />
