# Bellboy Kubernetes Deployment

## 배포 순서

### 1. Docker 이미지 빌드 및 푸시

```bash
# 이미지 빌드
docker build -t your-registry/bellboy:latest .

# 이미지 푸시
docker push your-registry/bellboy:latest
```

### 2. Secret 설정

`k8s/secret.yaml` 파일을 열어서 실제 Slack 토큰으로 변경:

```yaml
stringData:
  slack-token: "xoxb-your-actual-token-here"
  slack-channel: "C05Q0DSCT0A"
```

### 3. Kubernetes 리소스 배포

```bash
# RBAC 권한 생성 (ClusterRole, ServiceAccount 등)
kubectl apply -f k8s/rbac.yaml

# Secret 생성
kubectl apply -f k8s/secret.yaml

# Deployment 생성
kubectl apply -f k8s/deployment.yaml
```

### 4. 확인

```bash
# Pod 상태 확인
kubectl get pods -l app=bellboy

# 로그 확인
kubectl logs -f deployment/bellboy
```

## 주요 설정

### 1. 단일 Pod 보장

- `replicas: 1` - 항상 1개의 Pod만 실행
- `strategy.type: Recreate` - 업데이트 시 기존 Pod 종료 후 새 Pod 시작 (동시에 2개가 뜨지 않음)

### 2. 모든 네임스페이스 감시

- `ClusterRole`에 `apps/deployments`에 대한 `get`, `list`, `watch` 권한 부여
- 코드에서 `Api::all(client)` 사용으로 모든 네임스페이스 감시

### 3. RBAC 권한

- **ServiceAccount**: `bellboy`
- **ClusterRole**: deployments 리소스에 대한 읽기 권한
- **ClusterRoleBinding**: ServiceAccount와 ClusterRole 연결

## 환경 변수

- `SLACK_TOKEN`: Slack Bot 토큰 (Secret에서 주입)
- `SLACK_CHANNEL`: Slack 채널 ID (Secret에서 주입)
- `LANGUAGE`: 언어 설정 (`ko` 또는 `en`)
- `RUST_LOG`: 로그 레벨 (`info`, `debug` 등)

## 트러블슈팅

### Pod가 시작되지 않는 경우

```bash
kubectl describe pod -l app=bellboy
kubectl logs -l app=bellboy
```

### 권한 오류가 발생하는 경우

ClusterRole과 ClusterRoleBinding이 제대로 생성되었는지 확인:

```bash
kubectl get clusterrole bellboy
kubectl get clusterrolebinding bellboy
```

### Slack 알림이 오지 않는 경우

1. Secret이 제대로 설정되었는지 확인
2. 로그에서 Slack API 오류 확인
3. Slack Bot 토큰의 권한 확인 (`chat:write`, `chat:write.public`)
