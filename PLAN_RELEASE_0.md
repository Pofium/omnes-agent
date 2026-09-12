# Релиз 0 — Подготовка репозитория (README, issues, тег v0.1.0, gitignore)

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Подготовить репозиторий `Pofium/omnes-agent` к первому собственному релизу «Релиз 0» (тег `v0.1.0`): чистый корень, актуальные README (ru/en) с призывом заводить issues, корневые CONTRIBUTING/CODE_OF_CONDUCT, шаблоны `.github/`, CHANGELOG, аннотированный тег и черновик GitHub Release, к которому Планы 1–2 прикрепят артефакты.

**Architecture:** Репо — форк ZeroClaw (база ≈ v0.8.5). Инфраструктура релизов上游 частично унаследована (`backend/release-plz.toml`, `backend/scripts/release/*`), но на корне нет `.github/` вообще (ни workflows, ни шаблонов issues), версия workspace = `0.8.4`, из тегов есть только `archive/mobile-android-2026-09`. План наводит «витринный» порядок на корне, не трогая backend-инфраструктуру.

**Tech Stack:** git, GitHub (gh CLI 2.80.0 установлен), Markdown, conventional commits.

---

## Контекст (проверено в репо, 2026-09-11)

| Факт | Где |
|---|---|
| Версия workspace `0.8.4`, MSRV `1.96.0` | `backend/Cargo.toml` `[workspace.package]` |
| Remote: `https://github.com/Pofium/omnes-agent.git`, ветка `main` | `git remote -v` |
| Нет каталога `.github/` (ни шаблонов, ни workflows) | `ls .github` → пусто |
| `LICENSE` (MIT) есть на корне; `CONTRIBUTING.md` на корне НЕТ (есть `backend/CONTRIBUTING.md`, `backend/CODE_OF_CONDUCT.md`) | корень |
| License в Cargo: `MIT OR Apache-2.0`, при этом корневой `LICENSE` — только MIT | `backend/Cargo.toml` / корень |
| `repository` в Cargo указывает на старый URL `omnesagent-labs/omnesagent` (не `Pofium/omnes-agent`) | `backend/Cargo.toml` |
| Теги: только `archive/mobile-android-2026-09` | `git tag` |
| Untracked-мусор: `err.txt`, `out.txt`, `Untitled document.md`, `PLAN.md`, `frontend/desktop/assets/app_icon.ico` | `git status --short` |
| Удалены в worktree (не закоммичено): `PLAN_2026-09-07.md`, `PLAN_DESKTOP_MOBILE_SPLIT.md`, `PLAN_FRONTEND_WEB_MIGRATION.md`, `PLAN_ZEROCLAW_SYNC_2026-09-08.md`, `Ralph Orchestrator (Omnes Agent).md` | `git status --short` |
| `.gitignore` не покрывает `err.txt`, `out.txt`, `*.lnk` | корень `.gitignore` |
| README.md / README.en.md: нет ссылок на Releases/Issues, нет секции установки из релизов, нет призыва заводить issues | корень |
| `backend/AGENTS.md` требует conventional commits, PR-флоу, без AI-футеров в коммитах | `backend/AGENTS.md` |
| `gh` CLI доступен и авторизован | `gh --version` → 2.80.0 |

**Правило безопасности:** ничего не удалять без явного подтверждения пользователя (удалённые `PLAN_*.md` — сначала уточнить: архивировать в `docs/archive/` или оставить удалёнными).

---

## Решения

1. **Схема версий — `0.1.0`** (рекомендация): собственная нумерация Omnes Agent, независимая от унаследованной `0.8.4` (форк ZeroClaw). Аудит-методика fork-sync сверяется по коммитам, а не по номерам версий — конфликтов не будет. Правка — одна строка `[workspace.package] version`.
   - Альтернатива (если пользователь хочет наследовать нумерацию ZeroClaw): оставить `0.8.4`, тег `v0.8.4`. Уточнить перед Task 3, если явно не подтверждено.
2. **CHANGELOG-next.md** — это унаследованные заметки ZeroClaw v0.8.4. Его не коммитить как собственный changelog; собственный `CHANGELOG.md` создать с нуля (Keep a Changelog). `CHANGELOG-next.md` остаётся как есть (upstream-артефакт, полезен для fork-audit) — НЕ удалять.
3. Правила `backend/AGENTS.md` (conventional commits, без AI-атрибуции) соблюдаются во всех коммитах плана.

---

## Task 1: Гигиена корня и .gitignore

**Objective:** Рабочее дерево без случайных файлов, gitignore покрывает типовой мусор.

**Files:**
- Modify: `.gitignore`
- Untracked-файлы корня: `err.txt`, `out.txt`, `Untitled document.md`, `PLAN.md`

**Step 1:** Спросить пользователя про каждый файл (справка: `err.txt`/`out.txt` — логи прошлых сессий; `PLAN.md` — рабочий план ADE; `Untitled document.md` — неизвестно). По умолчанию: переместить в `C:\Projects\Omnes-agent\.hermes\plans\archive\` (не удалять).
**Step 2:** Добавить в конец `.gitignore`:

```gitignore
# ==========================================
# Session artifacts & logs (agent workspace)
# ==========================================
err.txt
out.txt
*.lnk
.hermes/
```

(`.hermes/` — рабочие планы Hermes; планы релиза при необходимости переносятся в `docs/plans/` и коммитятся осознанно.)
**Step 3:** Закоммитить удаление `PLAN_*.md` / `Ralph Orchestrator (Omnes Agent).md`, если пользователь подтвердил (файлы уже удалены в worktree — нужен фиксирующий коммит).

**Verify:** `git status --short` → чисто (только ожидаемые изменения). `git check-ignore err.txt out.txt` → оба игнорируются.

**Commit:** `chore(repo): clean root, extend gitignore for session artifacts`

---

## Task 2: Трекнуть иконку приложения

**Objective:** `frontend/desktop/assets/app_icon.ico` не должен потеряться — он нужен Плану 1 (иконка установщика и ярлыков).

**Files:**
- Add: `frontend/desktop/assets/app_icon.ico` (сейчас untracked)

**Step 1:** `git add frontend/desktop/assets/app_icon.ico && git commit -m "chore(desktop): track application icon asset"`
**Step 2:** Проверить, какой .ico использует Windows-runner (`frontend/desktop/windows/runner/resources/`), зафиксировать вывод для Плана 1: `ls frontend/desktop/windows/runner/resources/`.

**Verify:** `git ls-files frontend/desktop/assets/` содержит `app_icon.ico`.

---

## Task 3: Версия workspace → 0.1.0

**Objective:** Единая версия релиза 0 для Cargo-воркспейса (в неё будут смотреть План 1 и План 2).

**Files:**
- Modify: `backend/Cargo.toml` (строка `version = "0.8.4"` в `[workspace.package]`)
- Modify: `backend/Cargo.lock` (регенерация)

**Step 1:** Заменить `version = "0.8.4"` на `version = "0.1.0"` в `[workspace.package]`.
**Step 2:** Выровнять лицензионные метаданные: заменить `license = "MIT OR Apache-2.0"` на `license = "MIT"` (корневой `LICENSE` — только MIT; dual-лицензия потребовала бы добавить в корень полный текст Apache-2.0 — для релиза 0 избыточно).
**Step 3:** Исправить `repository = "https://github.com/omnesagent-labs/omnesagent"` → `repository = "https://github.com/Pofium/omnes-agent"`.
**Step 4:** `cd backend && cargo check -p omnesagent` — обновит `Cargo.lock` и подтвердит сборку манифеста. Expected: `Finished` без ошибок (проверка только манифеста, полный билд не нужен).

**Verify:** `grep -A2 '\[workspace.package\]' backend/Cargo.toml | grep version` → `0.1.0`; `grep -n 'license\|repository' backend/Cargo.toml` → `MIT` и `Pofium/omnes-agent`.

**Commit:** `chore(release): bump workspace version to 0.1.0, align license and repository metadata`

---

## Task 4: README.md (ru) — актуализация + призыв к issues

**Objective:** README отражает текущую структуру (desktop + web + shared, скрипты, порт 42617) и активно приглашает заводить issues.

**Files:**
- Modify: `README.md`

**Шаги:**
1. Секция «Архитектура монорепозитория»: исправить дерево — `frontend/desktop`, `frontend/web`, `frontend/shared`, `scripts/` (`vps_install.sh`, `omnesagent-update.sh`), `web/`; убрать строку про единый `frontend/lib/` (структура уже другая).
2. Секция «Быстрый старт»: переупорядочить по трём путям установки:
   - **Windows (из коробки):** `OmnesAgent-Setup-x64.exe` со страницы Releases — «Task: готовится, см. План 1» (заменить заглушку на реальную ссылку после Плана 1);
   - **VPS (веб-версия):** скачать архив релиза → `tar xzf` → `sudo ./install.sh` — «Task: готовится, см. План 2»;
   - **Из исходников:** текущее содержимое (`.\build.ps1`, `.\run.ps1`) — оставить как есть.
3. Добавить бейджи в шапку:

```markdown
[![Release](https://img.shields.io/github/v/release/Pofium/omnes-agent)](https://github.com/Pofium/omnes-agent/releases)
[![Issues](https://img.shields.io/github/issues/Pofium/omnes-agent)](https://github.com/Pofium/omnes-agent/issues)
```

4. Добавить секцию перед «Лицензия»:

```markdown
## 💬 Обратная связь и вклад

Нашли баг или нужна фича? Заведите issue — это лучший способ повлиять на roadmap:
- 🐞 [Сообщить об ошибке](https://github.com/Pofium/omnes-agent/issues/new?template=bug_report.yml)
- 💡 [Запросить функцию](https://github.com/Pofium/omnes-agent/issues/new?template=feature_request.yml)
- 🔒 Уязвимости безопасности — только через [Security Advisories](https://github.com/Pofium/omnes-agent/security/advisories/new), не в публичных issues.
```

**Verify:** Проверить ссылки `gh api repos/Pofium/omnes-agent --jq .html_url`; глазами рендер на GitHub (после push в Task 8); `grep -n "issues/new" README.md` находит 2 ссылки.

**Commit:** `docs(readme): update structure, install paths, issue callouts (ru)`

---

## Task 5: README.en.md — синхронизация

**Objective:** Английская версия получает те же изменения Task 4 (структура, установка, бейджи, issue-призыв).

**Files:**
- Modify: `README.en.md`

**Шаги:** Зеркально Task 4, все новые секции перевести на английский (ссылки идентичны).

**Verify:** `diff <(grep -c '^##' README.md) <(grep -c '^##' README.en.md)` — совпадает количество секций верхнего уровня.

**Commit:** `docs(readme): sync English version with Russian (structure, install, issues)`

---

## Task 6: Корневые CONTRIBUTING.md и CODE_OF_CONDUCT.md + шаблоны .github/

**Objective:** Стандартный контрибьюторский контур: куда писать, как заводить issue, как оформлять PR.

**Files:**
- Create: `CONTRIBUTING.md` (корень; краткий, ссылается на `backend/CONTRIBUTING.md` и `backend/AGENTS.md`)
- Create: `CODE_OF_CONDUCT.md` (копия из `backend/CODE_OF_CONDUCT.md`; исходник в backend оставить — на него могут ссылаться upstream-файлы)
- Create: `.github/SECURITY.md`
- Create: `.github/ISSUE_TEMPLATE/bug_report.yml`
- Create: `.github/ISSUE_TEMPLATE/feature_request.yml`
- Create: `.github/ISSUE_TEMPLATE/config.yml` (`blank_issues_enabled: false`, contact_links: Security Advisories)
- Create: `.github/PULL_REQUEST_TEMPLATE.md`

**SECURITY.md — минимум:** поддерживаемая версия (таблица: `v0.1.x` — поддерживается, ниже — нет); способ сообщения об уязвимостях — только через [Security Advisories](https://github.com/Pofium/omnes-agent/security/advisories/new), НЕ в публичных issues; что включать в отчёт (версия, шаги воспроизведения, логи без секретов); ожидаемый срок первого ответа (например, 7 дней). Без email-контакта, пока его нет — Advisories достаточно.

**Ключевое в CONTRIBUTING:** ветка по умолчанию — `main` (не `master`, как в унаследованных правилах); conventional commits; PR без AI-футеров; призыв начинать с issue.

**bug_report.yml — минимум полей:** `description` (textarea), `component` (dropdown: Backend/Gateway, Backend/KAG, Desktop (Flutter), Web (Flutter), Установка Windows, Установка VPS, Документация), `version` (строка, placeholder `v0.1.0`), `os`, `logs` (textarea, optional).

**Verify:** `gh api repos/Pofium/omnes-agent/contents/.github/ISSUE_TEMPLATE` после push; валидация YAML: `python -c "import yaml,glob;[yaml.safe_load(open(f,encoding='utf-8')) for f in glob.glob('.github/**/*.y*ml',recursive=True)]"`; `test -f .github/SECURITY.md` (GitHub подхватит её как Security Policy автоматически).

**Commit:** `chore(gh): add contributing guide, CoC, issue and PR templates`

---

## Task 7: CHANGELOG.md

**Objective:** Собственный changelog релиза 0.

**Files:**
- Create: `CHANGELOG.md`

**Содержание (формат Keep a Changelog, заголовок 0.1.0):** первый собственный релиз Omnes Agent; перечислить реально сделанное в форке: интеграция KAG/AST-памяти, десктопная рабочая станция (Flutter Windows) с авто-запуском шлюза, веб-клиент (ADE), скрипты `build.ps1`/`run.ps1`/`check.ps1`, `scripts/vps_install.sh`, русификация интерфейса. Ссылки на Планы 1–2 (установщики) как «планируется в следующих релизах» — либо заполнить после их выполнения.

**Verify:** Формат дат/ссылок консистентен; `CHANGELOG-next.md` не тронут.

**Commit:** `docs(changelog): add CHANGELOG.md for release 0.1.0`

---

## Task 8: Тег v0.1.0 и GitHub Release (draft)

**Objective:** Зафиксировать релиз 0 тегом и создать черновик Release, к которому Планы 1–2 прикрепят артефакты.

**Шаги:**
1. Убедиться, что все коммиты Task 1–7 в `main` и `git status` чист.
2. Аннотированный тег:
   ```bash
   git tag -a v0.1.0 -m "Omnes Agent Release 0 (v0.1.0)"
   git push origin main && git push origin v0.1.0
   ```
3. Черновик релиза:
   ```bash
   gh release create v0.1.0 --draft --title "Omnes Agent v0.1.0 — Release 0" --notes-file <(cat CHANGELOG.md)
   ```

**Verify:** `gh release view v0.1.0` существует (draft); `gh api repos/Pofium/omnes-agent/tags` содержит `v0.1.0`; README на GitHub рендерится, бейдж Release показывает `v0.1.0`.

**Важно:** Release публиковать (переводить из draft) только после прикрепления артефактов Планом 1 (`OmnesAgent-Setup-0.1.0-x64.exe`) и/или Планом 2 (`omnesagent-0.1.0-linux-amd64.tar.gz` + `SHA256SUMS`).

---

## Task 9: Базовый CI-воркфлоу `.github/workflows/ci.yml` + полировка репозитория

**Objective:** PR и пуши в `main` проверяются автоматически (компиляция, тесты, анализы) — до Плана 2, который добавит только релизный workflow. Плюс витринные настройки самого репозитория.

**Files:**
- Create: `.github/workflows/ci.yml`

**Скелет:**
```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:

permissions:
  contents: read

jobs:
  backend:
    runs-on: ubuntu-24.04
    timeout-minutes: 45
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable   # MSRV 1.96; при необходимости пин @1.96
      - uses: Swatinem/rust-cache@v2
        with: { workspaces: "backend" }
      - run: cargo fmt --check
        working-directory: backend
      - run: cargo clippy --workspace -- -D warnings
        working-directory: backend
      - run: cargo test --workspace
        working-directory: backend
  frontend:
    runs-on: ubuntu-24.04
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with: { channel: stable, cache: true }
      - run: flutter analyze
        working-directory: frontend/desktop
      - run: flutter test
        working-directory: frontend/desktop
```

**Замечания:** клапми/тесты могут выявить существующие проблемы форка — если воркфлоу красный на `main`, ослабить до `cargo check` + `cargo test -p omnesagent` и завести issue «починить CI» (красный CI на витрине хуже отсутствующего). Кэш обязателен: первый cold-run без него 20+ минут. `web/` и `frontend/shared` — добавить во flutter-job позже, когда появится их тестовая база.

**Полировка репозитория (шаг 2, опционально):**
```bash
gh repo edit Pofium/omnes-agent \
  --description "Omnes Agent — self-hosted AI agent (Rust gateway + Flutter desktop/web)" \
  --add-topic rust --add-topic flutter --add-topic ai-agent --add-topic self-hosted
```
Branch protection на `main` (require CI pass) — включить руками в Settings после первых зелёных прогонов.

**Verify:** push ветки с этим файлом → вкладка Actions показывает оба job'а зелёными (или заведён issue на красноту); `gh repo view Pofium/omnes-agent --json description,repositoryTopics` отражает изменения.

**Commit:** `ci: add basic CI workflow (cargo fmt/clippy/test, flutter analyze/test)`

---

## Риски и открытые вопросы

- **Номер версии** `0.1.0` vs наследование `0.8.4` — подтвердить у пользователя до Task 3 (влияет на имена артефактов в Планах 1–2).
- Удаление `PLAN_*.md` из индекса — только после явного подтверждения (память: «НИКОГДА не удалять без проверки»).
- `release-plz.toml` и `backend/scripts/release/bump-version.sh` остаются нетронутыми: upstream-инструментарий, в Релизе 0 не используется (ручной тег).
- Бейджи Issues/Releases могут 404 до первого публичного релиза/шаблонов — это норма, исчезнет после Task 6/8.
- Если репо приватное — shield-бейджи не отрисуются; проверить видимость репо: `gh repo view Pofium/omnes-agent --json visibility`.

## Зависимости

- План 1 (Windows installer) прикрепляет `OmnesAgent-Setup-*.exe` к release `v0.1.0` и вписывает реальную ссылку в README (Task 4, заглушка).
- План 2 (VPS installer) прикрепляет `omnesagent-*.tar.gz` и ссылку в README.
