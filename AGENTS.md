## Commands

- **Compile check:** `cargo check`
- **Build all:** `cargo build`

## Release workflow

1. **Install tools** (one‑time): `cargo install git-cliff`
2. **Make sure commits follow [Conventional Commits](https://www.conventionalcommits.org/):**
   - `feat:` — новая фича (Added)
   - `fix:` — багфикс (Fixed)
   - `refactor:` — рефакторинг (Changed)
   - `perf:` — оптимизация (Changed)
   - `docs:` — документация (пропускается в changelog)
   - `ci:` / `chore:` / `test:` — служебные (пропускаются)
   - breaking change — добавь `!` после типа: `feat!: remove old API`
3. **Выпуск релиза:**
   ```powershell
   # alpha bump (0.6.0-alpha.4 → 0.6.0-alpha.5)
   .\scripts\release.ps1 alpha

   # patch bump (0.6.0 → 0.6.1)
   .\scripts\release.ps1 patch

   # minor bump (0.6.0 → 0.7.0)
   .\scripts\release.ps1 minor
   ```
   Скрипт обновит версию в Cargo.toml, сгенерирует CHANGELOG.md (через git-cliff), сделает коммит и тег.
4. **Проверить и запушить:**
   ```powershell
   git show HEAD
   git push origin main --tags
   ```
