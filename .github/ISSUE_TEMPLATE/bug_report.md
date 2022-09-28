---
name: Bug report
about: Create a report to help us improve
title: ''
labels: ''
assignees: ''

---

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Please provide as much information as possible on how to trigger the issue. If we can recreate the issue, it is much more likely to be quickly addressed. Ideally, this would include details on how to recreate the problem using a fresh install on GKE or minikube. K8s clusters and installs tend to differ in unique ways, so if you can't provide recreation instructions please provide as many details on your configuration as possible.

**Expected behavior**
A clear and concise description of what you expected to happen.

**Output/Logs**
If applicable, please include CLI output or logs (if large please attach or put in gist/pastebin).
Logs for Trow are normally obtained from Kubernetes e.g. `kubectl -n kube-public trow-deploy...`

**Trow Info**
 - Install method (quick-install, Helm, Kustomize)
 - Version/Image Name (e.g. `extrality/trow:0.3`, can be obtained from `kubectl describe`)

**Kubernetes**
 - Kubernetes distro/host (EKS, GKE, Kind, Minikube etc)
 - Kubernetes version (e.g. 1.19)
 - Container Runtime (e.g. Docker, can be found via `kubectl get nodes -o wide`)


**Additional context**
Add any other context about the problem here.
