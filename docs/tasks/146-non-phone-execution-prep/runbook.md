# Non-phone execution runbook

This runbook implements #146 R3. Issue #146's latest execution report identifies
its reviewed Candidate, build run/job/artifact and Claim results. Until all are
accepted, this is a candidate runbook, not authority to publish #67/#68.

## Boundaries and fixed layout

Target alias: `tx-node`; observed Ubuntu 26.04, x86_64, glibc 2.43, system
`/usr/bin/python3` 3.14.4 and `/usr/bin/google-chrome`. Target has no pip;
no installation or compilation is performed there.

Dedicated locked-password/non-login account: `gateway-verify`, uid/gid 1001,
primary group only, home `/home/gateway-verify` mode 0700. It was created under
#146's narrow management authorization, without SSH keys or sudo entries.
Management SSH is root; all test/broker/browser runtime uses:

```sh
setpriv --reuid=1001 --regid=1001 --init-groups \
  --bounding-set=-all --inh-caps=-all --ambient-caps=-all --no-new-privs \
  env -i HOME=/home/gateway-verify PATH=/usr/bin:/bin PYTHONDONTWRITEBYTECODE=1 \
  /usr/bin/python3 /home/gateway-verify/artifact.py verify \
  --expected /home/gateway-verify/expected.json
```

No production files/profile/Vault inspection is part of this procedure. The
root connection only installs/launches files as the dedicated user; it never
runs the broker, browser or extractor as root.

The fixed bundle root is
`/home/gateway-verify/non-phone/runtime-80fb081b129f8f664124b84ddcc9698039e2cfd1`.
The compiled source root is its `source/` child. Do not relocate this path:
the frozen smoke binary embeds the worker path. Both binaries and runtime tests
are built from untouched `80fb081b129f8f664124b84ddcc9698039e2cfd1`, independently
of the wrapper/workflow Candidate.

## Hosted build and authenticated delivery

1. Push the exact reviewed Task Candidate on `worker/issue-146-*` to trigger
   `.github/workflows/non-phone-execution.yml`. It uses GitHub-hosted Ubuntu
   24.04, with <=35-minute build and <=10-minute other jobs.
2. `build.py` checks out frozen runtime at the fixed absolute path, uses the
   accepted offline bundle helper/lock, runs existing runtime and clean-build
   checks, enumerates test executables from Cargo JSON and packages binaries,
   worker, preinstalled wheel cache, lock/helper and provenance manifest.
3. Require `helpers`, `build` and `consumer` success. `consumer` runs on a fresh
   Runner without the runtime build tree; it exercises direct tests and the
   existing smoke's R008 loopback denial, never real site extraction. Missing
   or mutated worker/sandbox must fail before execution.
4. Use `download.py RUN_ID CANDIDATE_SHA NEW_OUTPUT_DIRECTORY` from the reviewed
   checkout in the authenticated Codex environment. It resolves the actual
   build job/artifact ID, checks run source/event/workflow, verifies the ZIP
   against GitHub's authenticated digest and binds the manifest hash. It emits
   `artifact.zip` and `expected.json`; never replace expected values with a
   checksum found only inside an unverified download.
5. Transfer those two files and the exact reviewed `artifact.py` into dedicated
   user-owned files in `/home/gateway-verify`. Transfer through stdin to
   `runuser -u gateway-verify` or equivalent; compare SHA256 independently
   before execution. Do not place GitHub or SSH credentials in this account.
6. Run `admit --archive /home/gateway-verify/artifact.zip --expected
   /home/gateway-verify/expected.json` with the same privilege-cleared prefix
   above. Admission requires a new destination, bounded file count/size,
   allowlisted regular paths and verified hashes. Traversal, links, wrong
   Candidate, missing or mutated assets fail closed. The receipt also freezes
   the target interpreter hash. Every subsequent launch revalidates all assets.

The cache is built/installed on Actions and restored read-only on target;
accepted `verify_bundle` and `verify_cache` checks prove frozen wheel/import
identity. There is no target pip, install fallback or cache regeneration hook.
A missing/corrupt cache requires new verified remote delivery.

## Runtime and browser proof / re-entry

Run `probe --expected /home/gateway-verify/expected.json` using the same
privilege-cleared prefix. It directly executes six named tests, requiring
one actual passed test per selector, covering actual yt-dlp handler, broker IPC,
network denial/no_new_privs, ambient fd denial, diagnostic containment and R008
pre-network rejection. It also starts the compiled smoke with a loopback URL;
R008 must reject that target before any connection. It never invokes cargo.

Transfer the exact reviewed `readiness.py` similarly and run it with the same
prefix. It creates an anonymous temporary profile, launches installed Chrome
headless with normal sandbox/autoplay settings, and accesses only its own
loopback fixture. JS POST proves both Host and Origin. Browser and fixture are
on tx-node; CDP does not listen anywhere. This executable CLI proof replaces
use of an unverified MCP/user-profile topology for this local fixture Claim.
It does not prove playback, autoplay, TV UX or Bilibili reachability.

For re-entry, repeat `verify`, `probe`, `readiness.py`, `verify`. No rebuild,
network package fetch or repeated extraction loop is needed. If a step fails,
retain only fixed result classes and the reviewed provenance. Do not print raw
browser/test stderr, DOM, account data or source/media output.

## Downstream launcher

Only after #146 Final Acceptance and #67 publication/preflight PASS may #67
use the same prefix with `artifact.py smoke --expected ... --source` and its
contract's exact frozen URL. The delivered launcher preserves the fixed sibling,
worker, offline runtime and normalized output boundary without invoking cargo.
#146 does not execute this operation. #68 must regenerate artifacts/manifests
for its own product Candidate; this package is not its product executable.

## Cleanup and recovery

The readiness helper stops its own process group and fixture, removes its own
temporary profile and confirms the fixture listener is closed. Probe test
processes are bounded and the existing runtime owns sandbox child cleanup.
After proof, verify no processes owned by uid 1001 remain. Do not inspect or
terminate other users' processes. Check only dedicated directories for leftover
fixture staging; never scan production homes/Vault.

Intentionally retain: the dedicated locked account, reviewed helper(s), verified
ZIP/control manifest, and admitted runtime/assets/cache/receipt. No daemon,
Runner, service, public listener or phone deployment is installed.

Reusing an admitted bundle requires `verify`; admission never overwrites it.
For replacement after a failed admission, remove only the owned `admission-*`
staging created by this tool. Replacing an already admitted fixed root requires
recording the old receipt and deliberately retiring that exact owned delivery;
do not recursively delete an unverified or symlinked destination. Artifact
expiration is resolved by another exact-Candidate hosted build, not a local build
or upstream version change.
