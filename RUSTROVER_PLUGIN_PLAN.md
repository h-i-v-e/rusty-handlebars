# RustRover Plugin Publication Plan

Last updated: 2026-07-27

Target release: `0.3.0`

Plugin XML ID: `dev.hive.rusty-handlebars`

## Goal

Publish the Rusty Handlebars plugin for RustRover through JetBrains
Marketplace, then verify that a user can discover, install, and use the
approved Marketplace build.

The implementation is complete. This document focuses only on the remaining
release, signing, Marketplace, and acceptance work.

## Current Status

| Area | Status | Notes |
| --- | --- | --- |
| RustRover plugin implementation | Complete | Located in `editors/jetbrains` |
| Coordinated version | Complete | Rust, VS Code, and RustRover are `0.3.0` |
| Local Rust, VS Code, and Gradle tests | Passing | Includes `buildPlugin` |
| Dedicated `.rhbs` editor | Complete | HTML/template highlighting, editing support, and Live Templates |
| Rust language-server integration | Complete | Standard LSP plus generated Rust and project reload requests |
| Native server build matrix | Configured | macOS ARM64/x64, Linux ARM64/x64, and Windows x64 |
| Universal plugin packaging | Configured | `.github/workflows/release-jetbrains.yml` |
| Minimum-version Plugin Verifier run | Passing | RustRover 2025.3.1 |
| Full Plugin Verifier matrix | Pending on final CI artifact | 2025.3.1, 2025.3.6, 2026.1.4, and 2026.2 |
| Cross-platform release artifact | Pending | Must be built by the release workflow from the final commit |
| Manual IDE acceptance | Pending | Must exercise the exact universal release candidate |
| Plugin signing | Pending | Gradle is configured, but the release workflow does not yet invoke signing |
| Marketplace vendor/account setup | Not confirmed | Requires the publisher to log in and complete Marketplace setup |
| First Marketplace upload | Pending | Must be performed manually |
| JetBrains approval | Pending | Every new plugin and update is reviewed |
| Installation from Marketplace | Pending | This is the final proof of publication |

The locally generated ZIP under `editors/jetbrains/build/distributions` is a
development archive. It does not contain all five native servers unless those
resources have first been staged. Do not upload that local archive to
Marketplace.

## Critical Path

```text
final commit
    ↓
universal CI artifact with five native servers
    ↓
Plugin Verifier + archive inspection
    ↓
manual acceptance of the exact artifact
    ↓
sign and verify the exact artifact
    ↓
create Marketplace vendor/listing and upload manually
    ↓
JetBrains review and approval
    ↓
install from Marketplace and complete final smoke test
```

Do not tag or publish a different ZIP from the one that passed archive
inspection, Plugin Verifier, and manual acceptance.

## 1. Finalize the Source Release

- [ ] Review the complete `0.3.0` diff.
- [ ] Commit the RustRover implementation, language-server hardening,
      packaging workflows, documentation, and coordinated version bump.
- [ ] Push the final commit to GitHub.
- [ ] Confirm CI passes on that exact commit.
- [ ] Replace `Unreleased` with the release date in the root and RustRover
      changelogs only when the candidate has passed acceptance.
- [ ] Record the final commit SHA in the release notes or release record.

Suggested commit message:

```text
feat: add RustRover support and prepare 0.3.0

- add native .rhbs editing and LSP integration for RustRover
- harden file URI handling and project-index reloading
- share native server builds across VS Code and JetBrains packages
- add universal RustRover packaging and verification workflows
- bump all coordinated packages and editors to 0.3.0
```

Cargo crate publication is independent of JetBrains Marketplace approval. It
is useful for a coordinated `0.3.0` release, but it is not a technical
prerequisite for uploading the RustRover plugin because the plugin embeds the
language-server executable.

## 2. Produce the Universal Release Candidate

Run the GitHub Actions workflow:

```text
Package RustRover plugin
```

Use `workflow_dispatch` on the final commit for the release candidate. Reserve
the `jetbrains-v0.3.0` tag for the accepted release rather than using a tag to
discover packaging problems.

The workflow must complete all of these jobs:

- [ ] build macOS ARM64 server;
- [ ] build macOS x64 server;
- [ ] build Linux musl ARM64 server;
- [ ] build Linux musl x64 server;
- [ ] build Windows x64 server;
- [ ] run `--version` for every native executable;
- [ ] generate SHA-256 checksums and build metadata;
- [ ] run the RustRover tests;
- [ ] build the universal plugin ZIP;
- [ ] inspect the ZIP for all five binaries exactly once;
- [ ] reject Cargo, Gradle, IDE sandbox, and signing material;
- [ ] run Plugin Verifier against every configured RustRover version.

Download the `rusty-handlebars-rustrover` artifact and record:

```text
commit SHA:
workflow run URL:
archive filename:
archive SHA-256:
archive size:
```

JetBrains Marketplace currently limits plugin uploads to 400 MB. Confirm the
universal archive remains comfortably below that limit.

## 3. Verify the Exact Candidate

### Automated gates

The final archive must pass:

```sh
cargo fmt -p rusty-handlebars-parser \
  -p rusty-handlebars-language-server -- --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings

cd editors/vscode
npm ci
npm run check
npm run compile

cd ../jetbrains
./gradlew test
./gradlew buildPlugin
./gradlew verifyPlugin
```

The release workflow, rather than a local `buildPlugin`, is the source of the
universal candidate because CI stages all native server resources.

Treat any of the following as a release blocker:

- missing dependencies or internal API usage from Plugin Verifier;
- a compatibility failure on any declared RustRover version;
- a native server that does not execute on its build runner;
- a missing or duplicate executable/checksum;
- generated directories or credentials inside the archive;
- an archive built from a different commit than the recorded candidate.

### Manual acceptance

Install the universal candidate with:

```text
Settings → Plugins → gear menu → Install Plugin from Disk
```

At minimum, test the oldest supported RustRover and the newest configured
RustRover. Exercise macOS ARM64 locally if that is the available development
machine, and obtain Linux x64 and Windows x64 acceptance before describing
those platforms as manually verified.

For each tested IDE/platform:

- [ ] install the exact candidate ZIP into a clean IDE profile;
- [ ] open a representative Cargo project;
- [ ] open or create a `.rhbs` file;
- [ ] confirm the bicycle icon and mixed HTML/template highlighting;
- [ ] confirm Live Templates, comments, quotes, and delimiter matching;
- [ ] introduce and clear a syntax diagnostic;
- [ ] request structural and project-field completion;
- [ ] test hover and navigation to a Rust field;
- [ ] test symbols, folding, block highlights, signature help, and selection;
- [ ] open generated Rust and confirm it is read-only and in memory;
- [ ] edit a Rust field and confirm project-index refresh;
- [ ] test the explicit reload and restart actions;
- [ ] configure a legacy `.hbs` glob and confirm unrelated `.hbs` files remain
      unclaimed;
- [ ] close the project and confirm the language-server process exits;
- [ ] confirm paths containing spaces and non-ASCII characters work;
- [ ] uninstall the plugin and confirm project files were not modified.

Record the IDE build, operating system, CPU architecture, and result for every
manual run.

## 4. Sign the Candidate

The Gradle build already reads these environment variables:

```text
CERTIFICATE_CHAIN
PRIVATE_KEY
PRIVATE_KEY_PASSWORD
PUBLISH_TOKEN
```

The remaining work is operational:

- [ ] choose or generate the certificate and private key;
- [ ] store the private key and password outside the repository;
- [ ] add the signing values as protected GitHub Actions secrets;
- [ ] restrict secret access to the release environment and authorized
      maintainers;
- [ ] update the RustRover release workflow to invoke `signPlugin`;
- [ ] run `verifyPluginSignature` on the signed output;
- [ ] upload the signed ZIP as a separate release artifact;
- [ ] calculate and record the signed archive SHA-256;
- [ ] install the signed ZIP once before Marketplace upload.

The current workflow builds and verifies an unsigned archive. Signing is not
complete merely because the `signing` block exists in `build.gradle.kts`.

Never commit a certificate, private key, signing password, Marketplace token,
or a decoded CI secret. JetBrains' current signing instructions are at:

<https://plugins.jetbrains.com/docs/intellij/plugin-signing.html>

## 5. Create the Marketplace Publisher and Listing

The first upload requires interactive Marketplace setup and must be performed
manually.

- [ ] log in to JetBrains Marketplace with the intended owner account;
- [ ] accept the JetBrains Marketplace Developer Agreement if required;
- [ ] create or select the Vendor profile;
- [ ] choose the Vendor ID carefully because it cannot be changed later;
- [ ] provide a working public vendor email and website;
- [ ] complete the Marketplace trader/non-trader declaration;
- [ ] confirm `Rusty Handlebars` is acceptable as the public name;
- [ ] confirm `dev.hive.rusty-handlebars` is accepted as the Plugin XML ID;
- [ ] select the open-source/MIT licensing option;
- [ ] provide the source URL:
      `https://github.com/h-i-v-e/rusty-handlebars`;
- [ ] provide the documentation and issue-tracker URLs;
- [ ] choose accurate Rust, template-language, and editor-related tags;
- [ ] state that processing is local and that template/Cargo data is not sent
      to an external service;
- [ ] state the supported RustRover versions and native platforms accurately;
- [ ] upload the distinct 40×40 SVG plugin logo;
- [ ] decide whether the first approved build should be hidden or use a
      Beta/custom release channel before promotion to Stable.

The public name must remain under JetBrains' 30-character limit and must not
include “Plugin”, “IntelliJ”, “JetBrains”, or a JetBrains product name. The
current `Rusty Handlebars` name and 40×40 SVG are designed to satisfy these
requirements, but Marketplace makes the final determination.

Current Marketplace upload guidance:

<https://plugins.jetbrains.com/docs/marketplace/uploading-a-new-plugin.html>

Current approval requirements:

<https://plugins.jetbrains.com/docs/marketplace/jetbrains-marketplace-approval-guidelines.html>

## 6. Perform the First Upload

- [ ] select the correct Vendor profile;
- [ ] upload the exact signed and verified `0.3.0` universal ZIP;
- [ ] confirm the plugin version, compatible products, and supported build
      range shown by Marketplace;
- [ ] confirm the listing uses the intended license, source, documentation,
      privacy, and support information;
- [ ] retain the upload confirmation and plugin page URL;
- [ ] record the Marketplace numeric plugin ID;
- [ ] create a permanent Marketplace token for later automated updates;
- [ ] store that token as the protected `PUBLISH_TOKEN` GitHub secret.

The project already configures Gradle's `publishPlugin` task, but JetBrains
requires the first upload to collect listing information interactively. Do not
enable automatic Marketplace publication until the first listing exists and
the token has been tested deliberately.

Every new plugin and every update is reviewed by JetBrains. Approval is not
guaranteed, and an uploaded archive must not be described as publicly
available until Marketplace shows it as approved. JetBrains currently suggests
contacting Marketplace support if a new plugin has no review update after
approximately 3–4 working days.

## 7. Respond to Review

- [ ] monitor the Marketplace owner email and plugin dashboard;
- [ ] answer any privacy, compatibility, licensing, or branding questions;
- [ ] implement requested corrections in source rather than patching only the
      ZIP;
- [ ] bump to `0.3.1` if a replacement archive is required after `0.3.0` has
      been accepted or otherwise made immutable by Marketplace;
- [ ] rebuild, re-run all gates, re-sign, and re-upload any corrected archive;
- [ ] record the approval date and final Marketplace URL.

Do not weaken dependency declarations or compatibility checks solely to pass
Marketplace review. Narrow the supported range if verification shows a real
platform incompatibility.

## 8. Verify Marketplace Installation

After approval:

- [ ] open a clean supported RustRover installation;
- [ ] find `Rusty Handlebars` through the Marketplace tab;
- [ ] install it without using a local ZIP;
- [ ] restart RustRover if requested;
- [ ] repeat the core `.rhbs`, diagnostics, completion, navigation, generated
      Rust, reload, and shutdown smoke tests;
- [ ] confirm the downloaded plugin selects the correct native server;
- [ ] confirm the displayed version is `0.3.0`;
- [ ] confirm the listing links, icon, description, license, and compatibility
      range are correct;
- [ ] promote/unhide the build if the first upload used a hidden or Beta
      channel;
- [ ] verify a second clean installation after promotion.

Marketplace publication is complete only after this installation succeeds.

## 9. Finish the Public Release

- [ ] create and push the `jetbrains-v0.3.0` tag at the recorded commit;
- [ ] create a GitHub release with the `0.3.0` release notes;
- [ ] attach the signed universal ZIP and its SHA-256;
- [ ] update the root and RustRover README files with the approved Marketplace
      URL;
- [ ] replace install-from-disk as the primary user installation path while
      retaining it as a fallback;
- [ ] mark `0.3.0` with its release date in both changelogs;
- [ ] announce only the platforms and RustRover versions actually verified;
- [ ] retain the workflow run, verifier reports, signed artifact checksum, and
      manual acceptance record.

## 10. Automate Later Updates

After the first listing and token are known to work:

- [ ] add an explicitly gated signing job to the RustRover release workflow;
- [ ] verify the signed archive in CI;
- [ ] configure `publishPlugin` to use the intended Stable or custom channel;
- [ ] require a protected GitHub release environment for Marketplace upload;
- [ ] prefer a manual approval gate before `publishPlugin`;
- [ ] prevent pull-request workflows from receiving signing or publishing
      secrets;
- [ ] keep the universal ZIP as a GitHub release artifact even when
      Marketplace publication succeeds;
- [ ] document rollback, hiding, and replacement procedures for a bad update.

Marketplace upload is an external state change. Do not make it an automatic
side effect of every tag until the release process has been exercised
successfully and explicit authorization is preserved.

## Definition of Done

The RustRover plugin is published only when all of these statements are true:

- [ ] the source and version are committed and tagged;
- [ ] the universal ZIP contains all five verified native servers;
- [ ] the complete Plugin Verifier matrix passes;
- [ ] the exact signed artifact passes manual acceptance;
- [ ] signing and signature verification pass;
- [ ] the Marketplace listing is complete and accurate;
- [ ] JetBrains has approved the plugin;
- [ ] `Rusty Handlebars` is visible through RustRover's Marketplace UI;
- [ ] a clean Marketplace installation succeeds;
- [ ] the installed plugin reports `0.3.0` and launches the correct server;
- [ ] public documentation links to the approved listing;
- [ ] no secrets or generated build directories are tracked by Git.

Until then, describe the plugin as implemented or release-candidate ready, not
as Marketplace-published.
