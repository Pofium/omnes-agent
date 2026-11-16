# Proposal: ob2h v2.0.0+ Follow-up & OmnesProducts Rebrand Completion

Реализация доработок по спецификации [`PLAN_OB2H_V2_FOLLOWUP_AND_REBRAND_2026-11-16.md`](../../../PLAN_OB2H_V2_FOLLOWUP_AND_REBRAND_2026-11-16.md):
- **Ф21**: устойчивый JSON-парсинг ответов LLM в Dream (`clean_json_balanced`), прямое чтение daily/сессионных логов в Dream, безопасный `git_head_sha` без `.git`, правило `session_ingest` в конце долгих сессий, поддержка ассемблера в AST (`asm-gas` / `asm-intel`).
- **Ф22**: ETA для `warming_up_embeddings`, сопряжение с `ob2h secrets ask-pin` и non-TTY `is_terminal()`-гейт.
- **Ф23**: вычистка оставшихся ~472 хардкодов старого циана (`0xFF00D2FF` / `0xFF00E5FF` / `0xFF0072FF`) в 23 файлах `frontend/`, бандлинг шрифтов `Montserrat` / `Inter` / `JetBrains Mono`, мета-теги Web и проверка палитры `omnescode`.
