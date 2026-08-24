import { readFileSync } from "node:fs";

const workflowPath = ".github/workflows/r002-target-tv-deployment.yml";
const workflow = readFileSync(workflowPath, "utf8");

function requireMatch(pattern, description) {
  if (!pattern.test(workflow)) {
    throw new Error(`missing ${description}`);
  }
}

function forbidMatch(pattern, description) {
  if (pattern.test(workflow)) {
    throw new Error(`forbidden ${description}`);
  }
}

requireMatch(/^on:\n  workflow_dispatch:\n/m, "workflow_dispatch-only trigger");
forbidMatch(/^\s+(pull_request|push):/m, "automatic pull_request/push target trigger");
requireMatch(/runs-on:\s*\[self-hosted, linux, ARM64, ubuntu-arm64, target-device\]/, "target runner labels");
requireMatch(/^permissions:\n  contents: read\n/m, "read-only contents permission");
requireMatch(/candidate_sha:[\s\S]*?required: true[\s\S]*?type: string/, "required candidate SHA input");
requireMatch(/hold_minutes:[\s\S]*?required: true[\s\S]*?type: choice[\s\S]*?options:\n\s+- 45\n\s+- 75\n\s+- 90/, "bounded hold duration choices");
requireMatch(/\^\[0-9a-fA-F\]\{40\}\$/, "full candidate SHA validation");
requireMatch(/R001_BIND_ADDR/, "explicit bind configuration");
requireMatch(/R001_PORT/, "dedicated bind port configuration");
requireMatch(/18788/, "fixed test port");
requireMatch(/\.trusted-workflow/, "trusted workflow checkout");
requireMatch(/\.candidate/, "separate candidate checkout");
requireMatch(/git -C .*rev-parse HEAD/, "candidate checkout identity verification");
requireMatch(/id -u/, "non-root validation");
requireMatch(/default_route_iface|ip -4 route show default/, "normal-route LAN address derivation");
requireMatch(/a == 10|a == 172 && b >= 16|a == 192 && b == 168/, "private IPv4 validation");
requireMatch(/\/healthz/, "health self-smoke");
requireMatch(/\/display/, "display self-smoke/entry");
requireMatch(/\/control/, "control self-smoke/entry");
requireMatch(/trap .*cleanup|trap cleanup/, "cleanup trap");
requireMatch(/actions\/upload-artifact@v4/, "diagnostic artifact upload");
requireMatch(/timeout-minutes:/, "job timeout");
forbidMatch(/\bsudo\b|apt-get|apt install/i, "privileged package installation");
forbidMatch(/\/var\/lib\/web-media-gateway|GITHUB_TOKEN|secrets\.|cookie|tailscale|ssh-key/i, "production or long-term secret path");

console.log("r002 target deployment workflow static validation: PASS");
