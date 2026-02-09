# bellboy

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
