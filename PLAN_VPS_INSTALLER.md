# План 2 — Архив и guided-установщик веб-части для VPS (omnesagent-*-linux-amd64.tar.gz + install.sh)

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Релизный архив `omnesagent-<ver>-linux-amd64.tar.gz`, который скачивают со страницы GitHub Release, распаковывают и запускают `sudo ./install.sh`. Установщик в интерактивном режиме ведёт по шагам (домен → порт → SSL → проверка) до полной конфигурации, пока веб-приложение реально не поднимется и не ответит health-check'ом; повторный запуск = обновление. На VPS НЕ требуется ни Rust, ни Flutter, ни сборка — только предсобранные бинарник и web-ассеты.

**Architecture:** Архив содержит предсобранный `bin/omnesagent` (linux x86_64), `web_dist/` (Flutter web, canvaskit) и guided-скрипт `install.sh`. Скрипт наследует проверенную логику существующего `scripts/vps_install.sh` (шаги 1–3, 6, 11–15: пакеты, swap, юзер, config.toml, systemd, nginx, certbot, ufw, self-check), но **убирает** шаги 4–5 (Rust/Flutter toolchains) и 7–9 (clone/build) — вместо них распаковка уже готовых артефактов в atomic-release лэйаут `/opt/omnesagent/releases/<ver>`.

**Tech Stack:** bash, systemd, nginx, certbot, ufw, GitHub Actions (опционально для CI-сборки), tar/gzip, sha256sum.

---

## Контекст (проверено в репо, 2026-09-11)

| Факт | Где |
|---|---|
| Существует `scripts/vps_install.sh` — 15 шагов, build-from-source (ставит Rust + Flutter на VPS — тяжело и долго) | `scripts/vps_install.sh` |
| Лэйаут, который он создаёт: `/opt/omnesagent/{bin,web_dist,releases,source}`, данные `/var/lib/omnesagent/data`, юзер `omnesagent`, atomic-симлинк `releases/current` | `vps_install.sh:13-14,154-190` |
| Конфиг шлюза: `[gateway] host=127.0.0.1 port=42617 web_dist_dir=...`, `[data] dir=...`, env `OMNESAGENT_CONFIG` | `vps_install.sh:197-206` |
| systemd unit с security-сэндбоксом (ProtectSystem=strict, ReadWritePaths, NoNewPrivileges) | `vps_install.sh:213-238` |
| nginx: proxy_pass 42617, WS-upgrade для `/ws/`, `client_max_body_size 100M`, gzip c wasm | `vps_install.sh:250-281` |
| Certbot через `--nginx`; ufw: 22/80/443 (+ custom port) | `vps_install.sh:287-308` |
| Self-check: systemd active, `curl /api/health` → 200, `/api/auth/me` → 401/200, креды в `$DATA_DIR/.install-credentials` | `vps_install.sh:311-338` |
| Существует `scripts/omnesagent-update.sh` — update-from-source; останется для source-пути, guided-инсталлятор делает свой update | `scripts/omnesagent-update.sh` |
| Web-ассеты: `flutter build web --release --web-renderer canvaskit` → локально уже собраны (`web/dist/`: `main.dart.js`, `flutter_bootstrap.js`, `canvaskit/`, …), в git НЕ трекаются (`dist/` в .gitignore) | `web/dist/` |
| Бэкенд: `cargo build --release --bin omnesagent`, workspace-версия из `backend/Cargo.toml` (после Плана 0 = `0.1.0`), MSRV 1.96 | `backend/Cargo.toml` |
| `.github/` отсутствует — workflows нет ни одного | корень репо |
| Gateway статика: шлюз сам отдаёт `web_dist_dir` (static_files в gateway) | `vps_install.sh:201` |

---

## Task 1: Скрипт сборки архива `scripts/make_release.sh`

**Objective:** Воспроизводимая упаковка релизного артефакта на Linux x86_64.

**Files:**
- Create: `scripts/make_release.sh`

**Интерфейс:**
```bash
./scripts/make_release.sh [--web-dist <dir>] [--out <dir>]
```
- `--web-dist`: путь к готовому `flutter build web` (по умолчанию пытается собрать сам, если есть flutter; иначе обязателен).
- Логика: версию взять из `backend/Cargo.toml` (`awk` по `[workspace.package] version`); собрать `cargo build --release --bin omnesagent` (cwd `backend/`); staging:

```
omnesagent-<ver>-linux-amd64/
├── install.sh            (копия scripts/install_vps.sh, Task 2; скрипт должен ПАДАТЬ с
│                          понятной ошибкой, если scripts/install_vps.sh ещё не создан)
├── uninstall.sh          (копия scripts/uninstall_vps.sh, Task 5)
├── bin/omnesagent        (release-бинарник)
├── web_dist/             (содержимое web-dist)
├── VERSION               (одна строка: <ver>)
└── README-VPS.txt        (5-7 строк: как установить, как удалить, как обновить)
```
- `tar czf omnesagent-<ver>-linux-amd64.tar.gz` + `sha256sum ... > SHA256SUMS`; распечатать оба пути и хэш.
- `shellcheck scripts/make_release.sh` — без warnings (shellcheck поставить: `apt install shellcheck` или скачать бинарник).

**Проверка бинарника:** `ldd bin/omnesagent` — зафиксировать список so-зависимостей; если линкуется `libssl` — в install.sh добавить `apt-get install -y libssl3` (обычно уже есть). Если сборка была под свежий glibc (например, Ubuntu 22.04, glibc 2.35) — задокументировать минимальную ОС: **Debian 12 / Ubuntu 22.04+**. Для макс. совместимости собирать в контейнере `rust:1-bullseye` (Debian 11, glibc 2.31).

**Verify:** распаковать tar.gz во временный каталог, `./bin/omnesagent --help` (или `--version`) не падает по glibc; `sha256sum -c SHA256SUMS` — OK.

**Commit:** `build(vps): release archive packaging script (bin + web_dist + guided installer)`

---

## Task 2: Guided-установщик `scripts/install_vps.sh`

**Objective:** Интерактивный пошаговый мастер: скачал → запустил → ответил на вопросы → веб-приложение работает. Плюс тихий режим через флаги и режим обновления.

**Files:**
- Create: `scripts/install_vps.sh`

**Логика (переиспользовать дословно блоки `vps_install.sh` — отмечены ссылками на строки):**

1. **Pre-flight (всё до первого вопроса пользователю, каждая проверка — с понятным сообщением и exit 1):**
   - root-проверка;
   - запуск из распакованного архива (рядом лежат `bin/`, `web_dist/`, `VERSION`) — иначе «запустите из распакованного архива релиза»;
   - архитектура: `uname -m` = `x86_64` (архив собирается только под amd64);
   - ОС/пакетный менеджер: детект `apt-get` (Debian/Ubuntu) или `dnf`/`yum` (RHEL: Rocky/Alma/Fedora); нет ни того, ни другого — ошибка с перечнем поддерживаемых ОС (Debian 11+/Ubuntu 20.04+/Rocky 9+);
   - glibc: `ldd --version` ≥ минимума из Task 1 (glibc 2.31 для bullseye-сборки) — иначе предложить обновить ОС;
   - systemd: `[ -d /run/systemd/system ]` — в контейнерах без systemd предложить ручной запуск `bin/omnesagent` с config.toml;
   - диск: ≥ 2 ГБ свободного места на пути `/opt` и каталога данных.
2. **Guided-вопросы (интерактив с дефолтами в []):**
   - Домен (пусто = доступ по IP); при домене — email для Let's Encrypt; SSL y/n; публичный порт (80/443 или кастомный); каталог данных (`/var/lib/omnesagent`).
   - Показ сводки «будет сделано» → подтверждение y/n.
   - Флаги для тихого режима: `--domain --email --port --no-ssl --data-dir` (те же имена, что в `vps_install.sh`).
3. **Пакеты — две ветки по результату детекта из pre-flight:**
   - **apt (Debian/Ubuntu):** `apt-get install -y nginx certbot python3-certbot-nginx curl jq ufw ca-certificates` — certbot/plugin только если SSL (`vps_install.sh:84-101`);
   - **dnf (RHEL/Rocky/Alma):** `dnf install -y nginx certbot python3-certbot-nginx curl jq ca-certificates` (+ `dnf install -y epel-release` перед certbot, если репозитория нет); **ufw на RHEL нет — вместо него firewalld** (см. шаг 7);
   - БЕЗ build-essential/rust/flutter — prebuilt-подход.
4. **Swap <2GB RAM** (`vps_install.sh:103-120`) — больше не критично (нет компиляции), но оставить как защитный минимум.
5. **Юзер и каталоги:** `useradd -r omnesagent`, `/opt/omnesagent/{releases/<ver>,bin,web_dist}` (`vps_install.sh:148-155`).
6. **Распаковка релиза:** копировать `bin/omnesagent` и `web_dist/` из архива в `releases/<ver>/`, atomic-симлинк `releases/current`, rsync web_dist, обновить `/opt/omnesagent/bin/omnesagent` (`vps_install.sh:185-189`). Если ставится поверх — прежний релиз остаётся в `releases/` (откат = переключение симлинка + рестарт).
7. **config.toml + systemd unit + nginx + certbot + firewall** — блоки из `vps_install.sh:195-308` почти без изменений; nginx-порт подставляется из ответов. Firewall — ветвление:
   - **apt/ufw:** разрешить 22/80/443 (+ кастомный порт) (`vps_install.sh:287-308`);
   - **dnf/firewalld:** `systemctl enable --now firewalld`; `firewall-cmd --permanent --add-service={http,https}` + `--add-service=ssh` (+ `--add-port=<порт>/tcp` для кастомного) → `firewall-cmd --reload`.
8. **Self-check (`vps_install.sh:311-338`):** systemctl active; health-check с retry-циклом — `for i in $(seq 1 30); do curl -sf http://127.0.0.1:42617/api/health && break; sleep 2; done` (одиночный curl ложно падает: сервису нужно пару секунд на подъём); unauth `/api/auth/me` → 401/200; если web_dist не отдаётся — отдельная проверка `curl -sf http://127.0.0.1:42617/` → 200 (страница Flutter).
9. **Финальный экран:** URL (https://домен или http://IP:порт), username `admin`, временный пароль из `.install-credentials`, команды статуса/логов, напоминание про смену пароля при первом входе.
10. **Update-режим:** если `/opt/omnesagent` уже существует и в архиве VERSION новее/равно установленной — пройти только шаги 6 и 8 (распаковка релиза, рестарт сервиса, self-check), вопросы не задавать (кроме подтверждения). При этом:
    - **забэкапить текущий `config.toml`** (`config.toml.bak-<дата>`) перед перезаписью;
    - **ре-рендерить systemd-unit и nginx-конфиг** идемпотентно из шаблонов нового архива (шаблоны между версиями могли измениться — старый unit с устаревшими `ReadWritePaths` способен сломать новый бинарник);
    - `.install-credentials` **не пересоздаётся** — админ-пароль остаётся прежним (не «обновляется» молча);
    - предыдущий релиз остаётся в `releases/` для отката.

**Правила качества:** `set -euo pipefail`; idempotent (повторный запуск не ломает); никаких секретов в stdout-логах кроме финальной выдачи пароля (он и так выводится только в конце); chmod 600 на `.install-credentials`; `shellcheck` чисто.

**Verify:**
```bash
docker run --rm -it -p 8080:80 debian:12 bash
# внутри: apt update && apt install -y systemd nginx curl jq ufw
# скопировать распакованный архив, ./install.sh --no-ssl --port 80
```
Expected: self-check зелёный, `curl -sf http://127.0.0.1/api/health` → 200, `curl http://127.0.0.1/` отдаёт HTML Flutter-приложения. (Docker без systemd — для теста юнит-логики запускать `omnesagent serve` вручную с config.toml; полный systemd-прогон делать на disposable VPS.)

**Commit:** `feat(vps): guided prebuilt release installer (install_vps.sh)`

---

## Task 3: CI-сборка релизных артефактов `.github/workflows/release.yml`

**Objective:** Тег → автоматически собранные архив (linux) и установщик (windows, План 1) прикрепляются к GitHub Release.

**Files:**
- Create: `.github/workflows/release.yml`

**Скелет:**
```yaml
name: Release
on:
  push:
    tags: ["v*"]

permissions:
  contents: write          # иначе GITHUB_TOKEN (restricted по умолчанию) не сможет создать релиз

concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: false

jobs:
  build-linux:
    runs-on: ubuntu-24.04            # 22.04-образы runner'ов на выводе — не закладываться на них
    timeout-minutes: 60
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with: { channel: stable, cache: true }
      - run: cd frontend/web && flutter build web --release --web-renderer canvaskit
      # cargo внутри контейнера: бинарник против glibc контейнера, а не раннера
      # (решение по итогу Task 1/ldd; rust:1-bullseye=glibc 2.31, если образ выведен —
      #  fallback rust:1-bookworm, glibc 2.36 → Debian 12 / Ubuntu 22.04+)
      - run: docker run --rm -v "$PWD:/w" -w /w/backend rust:1-bullseye cargo build --release --bin omnesagent
      - run: bash scripts/make_release.sh --web-dist frontend/web/build/web --out dist --skip-build
      - run: sha256sum dist/*.tar.gz > dist/SHA256SUMS
      - uses: softprops/action-gh-release@v2
        with: { files: "dist/*", draft: true }
  build-windows:
    runs-on: windows-2022
    timeout-minutes: 90
    steps:  # см. План 1: package_windows.ps1 + iscc
      - uses: actions/checkout@v4
      - run: ./scripts/package_windows.ps1
      - run: |
          Get-ChildItem dist/*.exe | Get-FileHash -Algorithm SHA256 |
            ForEach-Object { "{0}  {1}" -f $_.Hash.ToLower(), (Split-Path $_.Path -Leaf) } |
            Set-Content dist/SHA256SUMS-windows.txt
      - uses: softprops/action-gh-release@v2
        with: { files: "dist/*", draft: true }
```

**Замечания:** `make_release.sh` должен уметь `--skip-build` (бинарник уже собран контейнером в `backend/target/`) — иначе он пересоберёт его на раннере и смысла в контейнере нет; flutter-кэш обязателен (сборка web без кэша ~5+ мин); артефакты идут в **draft**-релиз, публикует человек.

**Verify:** push тестового тега `v0.1.0-rc1` → Actions зелёные → draft-релиз с tar.gz и SHA256SUMS. После проверки тег удалить.

**Commit:** `ci(release): build linux archive and windows installer on tag`

---

## Task 4: Публикация архива и README-инструкция

**Objective:** Пользователь может за 5 минут развернуть веб-версию из релиза.

**Шаги:**
1. `gh release upload v0.1.0 omnesagent-0.1.0-linux-amd64.tar.gz SHA256SUMS` (релиз-драфт из Плана 0).
2. README (ru/en), секция «VPS (веб-версия)» — заменить заглушку на:
   ```bash
   wget https://github.com/Pofium/omnes-agent/releases/download/v0.1.0/omnesagent-0.1.0-linux-amd64.tar.gz
   tar xzf omnesagent-0.1.0-linux-amd64.tar.gz
   cd omnesagent-0.1.0-linux-amd64
   sudo ./install.sh
   ```
   + таблица флагов тихого режима и строка про update (повторный запуск install.sh с новым архивом) + строка про удаление (`sudo ./uninstall.sh`, `--purge` — с данными).
3. Судьба старого `scripts/vps_install.sh`: оставить (путь build-from-source) и добавить в его шапку комментарий-ссылку «для предсобранного релиза см. install_vps.sh».

**Verify:** `gh release view v0.1.0` содержит tar.gz + SHA256SUMS (+ exe из Плана 1); ссылки в README кликабельны; `sha256sum -c` на скачанном файле сходится.

**Commit:** `docs(readme): vps quick-install from release archive (ru/en)`

---

## Task 5: Деинсталлятор `scripts/uninstall_vps.sh`

**Objective:** Симметрия с установщиком: чистое удаление сервиса и файлов одним скриптом. Данные пользователя по умолчанию СОХРАНЯЮТСЯ; полное удаление — только явным `--purge`.

**Files:**
- Create: `scripts/uninstall_vps.sh` (попадает в архив как `uninstall.sh` — Task 1)

**Логика:**
1. **Pre-flight:** root; если ни `/opt/omnesagent`, ни unit `omnesagent.service` не существует — «OmnesAgent не установлен, удалять нечего», exit 0.
2. **systemd:** `systemctl stop omnesagent || true`; `systemctl disable omnesagent || true`; удалить `/etc/systemd/system/omnesagent.service`; `systemctl daemon-reload`; `systemctl reset-failed`.
3. **nginx:** удалить сайт-конфиг из `sites-enabled`/`conf.d`, `nginx -t` → reload. **SSL-сертификат по умолчанию НЕ трогается** (принадлежит certbot и может использоваться другими сайтами); флаг `--remove-cert` → `certbot delete --cert-name <domain>` (если сертификатов несколько — спросить какой).
4. **Файлы:** `rm -rf /opt/omnesagent`; удалить юзера `omnesagent` (`userdel`), только если нет его процессов.
5. **Данные:** `/var/lib/omnesagent` (конфиг, SQLite, `.install-credentials`) сохраняется; в конце вывести, где лежат данные и как удалить вручную. Флаг **`--purge`** — удалить и их, но только после явного интерактивного подтверждения (попросить ввести `PURGE` целиком; в тихом режиме — отдельный флаг `--yes-i-know`).
6. **Качество:** `set -euo pipefail`; идемпотентный (повторный запуск не падает); `shellcheck` чисто; поддержка обеих веток пакетных менеджеров не нужна (ничего не доустанавливает), но firewalld/ufw правила, если добавлялись кастомные порты, не трогает (перечислить в выводе, что осталось).

**Verify:**
```bash
# на той же disposable VM: install → uninstall → install
sudo ./uninstall.sh                  # данные на месте; systemctl status omnesagent → not found; curl :42617 → refused
sudo ./uninstall.sh --purge          # /var/lib/omnesagent удалён
sudo ./install.sh                    # повторная установка работает
```

**Commit:** `feat(vps): guided uninstaller (uninstall_vps.sh)`

---

## Task 6: Полный прогон на disposable VPS

**Objective:** Реальная E2E-валидация guided-сценария на чистом сервере (не прод).

1. Взять дешёвый/временный VPS (Debian 12, root). **Обязательно прогнать и на RHEL-семействе** (Rocky 9/10): у пользователя целевые серверы обоих семейств (Debian 13 и Rocky 10; на Rocky — 702 MiB RAM, заодно проверит swap-блок).
2. Пройти сценарий пользователя: скачать релиз → распаковать → `sudo ./install.sh` → ответить на вопросы (сначала `--no-ssl` по IP, затем повторно с доменом+SSL).
3. Проверки: self-check скрипта зелёный; браузером открыть URL — Flutter-приложение грузится; вход admin + временный пароль; WS работает (чат без рестартов); `sudo journalctl -u omnesagent -f` без ошибок.
4. Update-тест: собрать/подложить архив той же версии → повторный `install.sh` → сервис перезапустился, данные на месте, релизов в `releases/` два, `config.toml.bak-<дата>` создан, пароль админа не изменился.
5. Откат-тест: переключить `releases/current` на предыдущий релиз + рестарт → старая версия работает.
6. Uninstall-тест: `uninstall.sh` (данные живы) → повторный install работает; `uninstall.sh --purge` → всё чисто.

**Verify:** чеклист пунктов 2–6 пройден на обоих семействах ОС; результат зафиксировать в комментариях к релизу/issue.

---

## Риски, компромиссы, открытые вопросы

- **glibc-совместимость** — главный риск prebuilt-подхода. Минимизация: сборка в `rust:1-bullseye` (glibc 2.31) → работает на Debian 11/12, Ubuntu 20.04+. Проверять `ldd` после каждой сборки (Task 1). Альтернатива на будущее: musl-static (сложнее с TLS-стеками — отложить).
- **RAM при cargo release-сборке в CI** — не проблема (runner 7GB), но при ручной сборке на VPS с малым RAM — оставшийся swap-блок из `vps_install.sh` полезен.
- **Certbot rate limits / DNS** — если домен не смотрит на VPS, certbot упадёт: скрипт должен WARNING-ом продолжать HTTP-режим (как в `vps_install.sh:287-293`), не валить установку.
- **systemd-сэндбокс**: `ProtectSystem=strict` + `ReadWritePaths` — при смене `--data-dir` пути в unit должны обновляться (уже так в шаблоне).
- **Flutter web canvaskit** грузит wasm с CDN? — по умолчанию локально из `canvaskit/` внутри web_dist; проверить на VPS без интернета к google-cdn (если тянет CDN — добавить `--web-renderer html` fallback или зафиксировать локальный canvaskit). **Открытый вопрос**, проверяется в Task 6.
- **ОС-матрица (apt + dnf):** исходный `vps_install.sh` был apt-only — неявное ограничение, несовместимое с реальностью пользователя (серверы на Debian 13 И Rocky 10). Добавленная dnf/firewalld-ветка удваивает поверхность тестирования: каждый инфраструктурный блок (пакеты, firewall, nginx-пути — на RHEL `conf.d`, не `sites-enabled`!) проверять на обоих семействах (Task 6).
- **Несколько VPS:** установка на каждый сервер полностью независима (общего состояния нет — у каждого свой config.toml, БД, креды). Обновление = повторный `install.sh` с новым архивом на каждом сервере отдельно (откат на каждом тоже независимый). Зафиксировать в README-VPS, чтобы не возникло ожидания «централизованного управления парком».
- Русификация текстов установщика: текущий `vps_install.sh` пишет финальный блок по-русски — сохранять тот же стиль; самоустойчивость: если терминал без UTF-8, не ломать вывод.
- `install_vps.sh` и `vps_install.sh` дублируют блоки systemd/nginx — сознательный компромисс (независимость скриптов); при изменении инфраструктуры править оба, либо позже вынести общий source.

## Зависимости
- План 0: тег `v0.1.0` и draft-релиз, версия `0.1.0`, шаблоны `.github/` (release.yml ляжет в созданный каталог).
- План 1: job `build-windows` в release.yml — только после того, как `package_windows.ps1` существует.
