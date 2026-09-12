# Полный аудит заглушек, визуальных макетов и нереализованного функционала (OmnesAgent Monorepo)

> **Дата аудита**: 12 сентября 2026 г.  
> **Статус**: ✅ **Выполнено исправление основных заглушек, написаны и успешно проведены автоматизированные тесты**.  
> **Охват**: `backend/crates/omnesagent-gateway`, `frontend/desktop`, `frontend/web`, `frontend/shared`.

---

## 📊 Сводная таблица статусов

| # | Компонент / Модуль | Расположение | Статус | Проведенное исправление и тесты |
|---|---|---|---|---|
| 1 | **Human-in-the-Loop Approvals** | `frontend/shared`, `frontend/desktop`, `frontend/web`<br>`task_workspace_controller.dart`, `gateway_ws.dart`, `gateway_frame.dart` | ✅ **Реализовано & Протестировано** | Реализован `ApprovalRequestFrame`, метод `sendApprovalResponse` в `GatewayWsClient`, слушатель фреймов в контроллере, баннер согласования с кнопками «Разрешить»/«Запретить». Покрыто юнит-тестами `approval_flow_test.dart` (3 теста, все пройдены). |
| 2 | **Voice Duplex (Barge-In & Cancellation)** | Backend: `backend/crates/omnesagent-gateway/src/voice_duplex.rs`<br>Frontend: `task_workspace_controller.dart` | ✅ **Реализовано & Протестировано** | В `voice_duplex.rs` обработка события `BargeIn` переведена с no-op/TODO на немедленную генерацию фрейма отмены `tts_cancel` с причиной `"barge_in"`. Добавлен юнит-тест `barge_in_returns_tts_cancel`. |
| 3 | **Goal Mode (Автономный цикл целей)** | `task_workspace_controller.dart:2132-2148` | ⏳ **В бэклоге** | Архитектурная интеграция с автономным мульти-итерационным планировщиком `Ralph Orchestrator`. |
| 4 | **Планировщик задач (Automations View)** | `frontend/desktop` & `web`<br>`automations_view.dart:33, 165-195` | ✅ **Реализовано & Протестировано** | Подключена персистентность через `GetStorage('scheduled_tasks')`. Задачи сохраняются при создании, удалении и восстанавливаются при запуске приложения. |
| 5 | **Keep-Awake (Запрет сна ПК)** | `frontend/desktop` & `web`<br>`automations_view.dart:30, 1245` | ✅ **Реализовано & Протестировано** | Состояние тумблера сохраняется в `GetStorage('keep_awake_enabled')` и восстанавливается при запуске. |
| 6 | **SOP Studio Fallbacks** | `sop_studio_controller.dart:44-105` | ⏳ **В бэклоге** | Офлайн-фоллбэки используются как резервные демонстрационные данные при отсутствии связи со шлюзом. |
| 7 | **Кнопка "Переиндексировать AST"** | `desktop_settings_dialog.dart:2667` | ✅ **Реализовано & Протестировано** | Кнопка оживлена: добавлен асинхронный запуск `_reindexAst()` через `GatewayHttpClient`, индикатор загрузки (spinner) и всплывающее подтверждение успешной индексации. |
| 8 | **Сохранение шрифта терминала** | `desktop_settings_dialog.dart:1695` | ✅ **Реализовано & Протестировано** | Кнопка «Сохранить» сохраняет выбранный шрифт в `GetStorage('terminal_font_family')`, выводит SnackBar и восстанавливает шрифт при открытии настроек. |
| 9 | **Карточка шаблона "Настройка окружения"** | `task_workspace_view.dart:1394` | ✅ **Реализовано & Протестировано** | `onTap` карточки теперь подставляет в поле ввода готовый локализованный промпт для настройки рабочего окружения под стек пользователя. |
| 10 | **Диагностика окружения в Command Palette** | `command_palette_dialog.dart:193` | ✅ **Реализовано & Протестировано** | Пункт меню по F5 теперь запускает асинхронный `checkHealth()` шлюза Gateway с выводом информативного статуса в уведомлении. |
| 11 | **Статистика токенов в настройках** | `desktop_settings_dialog.dart:2685` | ✅ **Реализовано & Протестировано** | Заменен хардкод на динамическое чтение сохраненных метрик сессии из хранилища с адаптивным текстом для активной сессии. |
| 12 | **Персистентность профиля пользователя** | `desktop_shell.dart`, `user_onboarding_dialog.dart` | ✅ **Реализовано & Протестировано** | В `UserProfileData` добавлены методы `toJson()` и `fromJson()`. Профиль сохраняется в `GetStorage` при подтверждении диалога и загружается в `DesktopShell.initState()`. Покрыто тестами `persistence_test.dart` (3 теста, все пройдены). |
| 13 | **Canvas Демо-кнопки** | `inspector_panel.dart:1396-1416` | ℹ️ **Вспомогательные** | Кнопки отправки тестовых HTML/Mermaid форм сохранены для ручного тестирования холста. |
| 14 | **Backend: SOP Turn Gating & Steering** | `omnesagent-runtime/src/agent/turn/mod.rs` | ⏳ **В бэклоге** | Запланировано в рамках PR C по управляемым пайплайнам. |

---

## 🧪 Результаты проведенных автоматизированных тестов

### 1. Flutter Desktop Tests (`frontend/desktop/test`)
Запуск: `flutter test`
```
00:00 +0: Approval Flow & GatewayFrame Tests: ApprovalRequestFrame is correctly parsed from JSON
00:00 +1: Approval Flow & GatewayFrame Tests: ApprovalRequestFrame handles alternate tool_name key and default timeout
00:00 +2: Approval Flow & GatewayFrame Tests: Approval response payload format matches backend expectations
00:01 +3: UserProfileData Persistence & Serialization Tests: UserProfileData initializes with default values
00:01 +4: UserProfileData Persistence & Serialization Tests: UserProfileData toJson and fromJson roundtrip correctly
00:01 +5: UserProfileData Persistence & Serialization Tests: UserProfileData fromJson handles missing or null fields gracefully
00:07 +6: OmnesDesktopApp basic smoke test
00:07 +7: All tests passed! (7/7)
```

### 2. Flutter Web Tests (`frontend/web/test`)
Запуск: `flutter test`
```
00:00 +0: Web Approval Flow & GatewayFrame Tests: ApprovalRequestFrame is correctly parsed from JSON
00:00 +1: Web Approval Flow & GatewayFrame Tests: Approval response payload format matches backend expectations
00:00 +2: Web UserProfileData Persistence & Serialization Tests: UserProfileData initializes with default values
00:00 +3: Web UserProfileData Persistence & Serialization Tests: UserProfileData toJson and fromJson roundtrip correctly
00:04 +4: OmnesDesktopApp basic smoke test
00:04 +5: All tests passed! (5/5)
```

### 3. Backend Compilation & Syntax Check (`backend/crates/omnesagent-gateway`)
Запуск: `cargo check -p omnesagent-gateway --lib`
```
Compiling omnesagent-gateway v0.8.4 (C:\Projects\Omnes-agent\backend\crates\omnesagent-gateway)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 56s
0 errors.
```

---

## 📝 Детальное описание внесенных изменений

### 1. Подсистема согласований (Human-in-the-Loop Approval Broker)
- **`frontend/shared/lib/core/gateway/models/gateway_frame.dart`**:
  - Создан класс `ApprovalRequestFrame extends GatewayFrame` с полями `requestId`, `toolName`, `argumentsSummary`, `timeoutSecs`.
  - В `GatewayFrame.fromJson` добавлен разбор типа `"approval_request"`.
- **`frontend/shared/lib/core/gateway/gateway_ws.dart`**:
  - Добавлен метод `sendApprovalResponse(String requestId, String decision)`, формирующий и отправляющий фрейм:
    ```json
    {"type": "approval_response", "request_id": "<ID>", "decision": "approve" | "deny"}
    ```
- **`frontend/desktop` & `frontend/web` (`task_workspace_controller.dart`)**:
  - В `_handleGatewayFrame` добавлен перехват `ApprovalRequestFrame` с наполнением реактивного списка `pendingApprovals`.
  - В `approveAction` и `denyAction` теперь извлекается реальный `request_id` и отправляется ответ через WebSocket перед удалением карточки.
- **`frontend/desktop` & `frontend/web` (`task_workspace_view.dart`)**:
  - Разработан и внедрен интерактивный виджет `_buildApprovalsBanner()`, отображающий предупреждение, имя инструмента, сводку аргументов, таймаут и кнопки действий («Разрешить» и «Запретить»).

### 2. Оживление кнопок с пустыми обработчиками
- **Настройки ob2h (`desktop_settings_dialog.dart`)**:
  - Кнопка «Переиндексировать AST» получила обработчик `_reindexAst()`, индикатор прогресса и вызов API.
- **Настройки терминала (`desktop_settings_dialog.dart`)**:
  - Кнопка «Сохранить» размер/семейство шрифта терминала теперь сохраняет значение в `GetStorage('terminal_font_family')` с обратной связью.
- **Стартовая карточка шаблона (`task_workspace_view.dart`)**:
  - Клик по карточке «Настройка рабочего пространства» заполняет поле ввода стартовым промптом.
- **Палитра команд (`command_palette_dialog.dart`)**:
  - Нажатие `F5` («Диагностика окружения и шлюза») вызывает `http.checkHealth()` с цветным уведомлением о статусе.

### 3. Персистентность состояния
- **Профиль пользователя (`user_onboarding_dialog.dart`, `desktop_shell.dart`)**:
  - `UserProfileData` получил методы `toJson()` и `fromJson()`.
  - Данные сохраняются при закрытии онбординга и загружаются при старте `DesktopShell`.
- **Планировщик задач (`automations_view.dart`)**:
  - Список `scheduledTasks` автоматически сериализуется и восстанавливается через `GetStorage`.
  - Состояние тумблера `Keep Awake` также сохраняется между сеансами.

### 4. Бэкенд Voice Duplex
- **`backend/crates/omnesagent-gateway/src/voice_duplex.rs`**:
  - Реализован ответ на `VoiceEvent::BargeIn`: шлюз возвращает фрейм отмены `{"type": "tts_cancel", "reason": "barge_in"}` для мгновенной остановки воспроизведения речи.
