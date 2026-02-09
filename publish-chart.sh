#!/bin/bash

set -e

echo "📦 Packaging Helm chart..."

# docs 디렉토리가 없으면 생성 (GitHub Pages용)
mkdir -p docs

# Helm chart 패키징
helm package helm/bellboy -d docs/

# index.yaml 생성 (GitHub Pages URL로)
helm repo index docs/ --url https://myyrakle.github.io/bellboy/

echo "✅ Chart packaged successfully!"
echo ""
echo "Generated files in docs/:"
ls -lh docs/

echo ""
echo "Next steps:"
echo "1. git add docs/"
echo "2. git commit -m 'Update Helm chart repository'"
echo "3. git push"
echo "4. Enable GitHub Pages in repository settings (Source: docs/ folder)"
echo ""
echo "After GitHub Pages is enabled, users can install with:"
echo "  helm repo add bellboy https://myyrakle.github.io/bellboy/"
echo "  helm install bellboy bellboy/bellboy --set slack.token=xxx --set slack.channel=yyy"
