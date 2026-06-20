---
type: source
title: designing-stealthy-portable-cli-agents
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# designing-stealthy-portable-cli-agents

Ingested source summary (2026-06-08).

## Entities
- [Nmap](/entities/nmap.md) (TOOL)
- [Python](/entities/python.md) (TOOL)
- [API gateways](/entities/api-gateways.md) (SYSTEM)
- [Ansible](/entities/ansible.md) (TOOL)
- [NVIDIA GPUs](/entities/nvidia-gpus.md) (SYSTEM)
- [Jenkins](/entities/jenkins.md) (TOOL)
- [Kismet](/entities/kismet.md) (TOOL)
- [Docker container](/entities/docker-container.md) (SYSTEM)
- [CLI agent](/entities/cli-agent.md) (CONCEPT)
- [torch.cuda.is_available()](/entities/torch-cuda-is-available.md) (CONCEPT)
- [Kubernetes](/entities/kubernetes.md) (TOOL)
- [Ollama server](/entities/ollama-server.md) (SYSTEM)
- [WAF (Web Application Firewall)](/entities/waf-web-application-firewall.md) (SYSTEM)
- [Windows Defender](/entities/windows-defender.md) (SYSTEM)
- [SIEM](/entities/siem.md) (SYSTEM)
- [Apple Silicon (MPS)](/entities/apple-silicon-mps.md) (SYSTEM)
- [GitLab CI](/entities/gitlab-ci.md) (TOOL)
- [wireless intrusion prevention systems (WIPS)](/entities/wireless-intrusion-prevention-systems-wips.md) (SYSTEM)
- [Terraform](/entities/terraform.md) (TOOL)

## Relations
- CLI agent → RELATED_TO → Windows Defender
- CLI agent → USES → Python
- Python → USES → torch.cuda.is_available()
- torch.cuda.is_available() → RELATED_TO → NVIDIA GPUs
