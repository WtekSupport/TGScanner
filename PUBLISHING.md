# Publishing & deployment (GitHub)

Public repository: **[github.com/WtekSupport/TGScanner](https://github.com/WtekSupport/TGScanner)**  
Maintainer: **WtekSupport** · License: **AGPL-3.0-or-later**

Release numbering starts at **v1.0.1** — see [CHANGELOG.md](CHANGELOG.md).

## What gets published

| Included in git | Never commit |
|-----------------|--------------|
| Source (`src/`), `Cargo.toml`, `Cargo.lock` | `.tguser_*`, `output/` |
| Docs, CI (`.github/workflows/`) | API ID/hash, phone, session blobs |
| `LICENSE`, `README.md`, templates | `tgscanner-crash.log`, `.env` |
| | `target/` (build output) |

Before every push, run:

```bash
git status
git diff --cached
```

If anything under `.tguser_*`, `output/`, or `target/` appears, **stop** and fix `.gitignore`.

## Repository setup (one time)

1. Create **public** repo `TGScanner` under org/user **WtekSupport** (empty — no auto README).
2. **Settings → General → Features:** Issues ✓  
3. **Settings → Security:** Private vulnerability reporting ✓  
4. **Settings → Actions → General:** Allow GitHub Actions ✓  
5. **Settings → Branches → Branch protection** for `main` (see below) — **only you** push commits to `main`.

### Protect `main` (only owner pushes)

Goal: nobody except **`WtekSupport`** can push commits to `main` (including force-push and branch delete).

**Option A — GitHub UI**

1. Repo → **Settings → Branches → Branch protection rules → Add rule**
2. **Branch name pattern:** `main`
3. Enable:
   - **Restrict who can push to matching branches** → add user **`WtekSupport`** only (remove everyone else)
   - **Do not allow bypassing the above settings** (recommended)
   - **Block force pushes**
   - **Block deletions**
4. Optional: **Require status checks to pass** → select CI job `build` (after Actions run once)
5. Save

Also: **Settings → Collaborators** — do not grant **Write** or **Admin** to others unless they should never touch `main` anyway (protection blocks their pushes).

**Option B — script (after `gh auth login`)**

```powershell
.\scripts\setup-branch-protection.ps1
```

To allow a different GitHub username, edit `$PushUsers` in the script.

## First push

```bash
git init -b main
git add .
git commit -m "Release TGScanner 1.0.1"
git remote add origin https://github.com/WtekSupport/TGScanner.git
git push -u origin main
```

## CI (every push / PR)

Workflow [`.github/workflows/ci.yml`](.github/workflows/ci.yml):

- Builds **release** on Windows, Linux, macOS (`cargo build --release --locked`)
- Uploads artifacts: `tgscanner-windows-x86_64`, `tgscanner-linux-x86_64`, `tgscanner-macos-universal`

Check **Actions** tab after push.

## Official release (binaries for users)

Automated workflow [`.github/workflows/release.yml`](.github/workflows/release.yml) runs when you push a version tag:

```bash
# After bumping version in Cargo.toml + CHANGELOG.md
git add Cargo.toml CHANGELOG.md
git commit -m "Prepare release v1.0.2"
git tag -a v1.0.2 -m "TGScanner v1.0.2"
git push origin main
git push origin v1.0.2
```

The **Release** job builds all three platforms and attaches binaries to the GitHub Release page.

Manual alternative: download artifacts from a green **CI** run and attach them in **Releases → Draft new release**.

## Making the app discoverable

- Pin the latest **Release** on the repo home page.
- README badges link to CI and license (already in [README.md](README.md)).
- Add release notes from [CHANGELOG.md](CHANGELOG.md) when drafting the GitHub release.
- Users download **Windows:** `tgscanner-windows-x86_64.exe` — no installer; run from a folder where the app can write `.tguser_*` and `output/`.

## Security checklist

- [ ] No secrets in git history (`git log -p` search for `api_hash`, phone numbers)
- [ ] [SECURITY.md](SECURITY.md) policy visible
- [ ] Dependabot enabled ([`.github/dependabot.yml`](.github/dependabot.yml))
- [ ] Issues/PRs: remind reporters not to paste credentials

## Local release build (Windows)

```bash
cargo build --release --locked
# Binary: target/release/tgscanner.exe
```

Do not copy `target/release/.tguser_*` when sharing the exe — those are local runtime files.

## Commits without third-party co-authors

Make release commits from **Git Bash** or **cmd** outside the IDE so the message stays exactly:

`Release TGScanner v1.0.1 for public GitHub deployment.`

If a `Co-authored-by:` trailer appears in history, amend and force-push from a normal terminal (not an IDE-integrated shell), then move tag `v1.0.1` to the new commit.
