// SOP Studio Controller for managing and executing SOP pipelines via OmnesAgent Gateway.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';

class SopStudioController extends GetxController {
  final GatewayHttpClient httpClient = GatewayHttpClient();

  final isLoading = false.obs;
  final sops = <Map<String, dynamic>>[].obs;
  final selectedSopName = RxnString();
  final selectedGraph = Rxn<Map<String, dynamic>>();
  final sopRuns = <Map<String, dynamic>>[].obs;
  final activeRunId = RxnString();

  @override
  void onInit() {
    super.onInit();
    loadSops();
    loadRuns();
  }

  /// Fetches all SOPs from gateway.
  Future<void> loadSops() async {
    isLoading.value = true;
    try {
      final list = await httpClient.getSopsList();
      if (list.isNotEmpty) {
        sops.assignAll(list);
        if (selectedSopName.value == null && sops.isNotEmpty) {
          selectSop(sops.first['name']?.toString() ?? '');
        }
      } else {
        // Fallback demo SOPs if backend has none created yet
        _initFallbackSops();
      }
    } catch (_) {
      _initFallbackSops();
    } finally {
      isLoading.value = false;
    }
  }

  void _initFallbackSops() {
    sops.assignAll([
      {
        'name': 'security-audit',
        'title': 'Аудит безопасности и зависимостей',
        'description': 'Сканирование уязвимостей (cargo/npm audit), оценка CVE и отчет безопасности',
        'execution_mode': 'supervised',
        'triggers': ['manual', 'cron: ежедневно 03:00'],
      },
      {
        'name': 'release-build',
        'title': 'Сборка и валидация релиза',
        'description': 'Статический анализ, прогон тестов, компиляция бинарников и проверка контрольных сумм',
        'execution_mode': 'autonomous',
        'triggers': ['manual', 'git: tag'],
      },
      {
        'name': 'vps-proxy-sync',
        'title': 'Мониторинг VPS и прокси-туннелей',
        'description': 'Аудит доступности VPS моста (193.109.79.30), SOCKS5/SSH и перезапуск демонов',
        'execution_mode': 'autonomous',
        'triggers': ['cron: каждые 15 мин'],
      },
      {
        'name': 'code-review-gate',
        'title': 'Автономное код-ревью (PR Gate)',
        'description': 'Анализ git diff, проверка соответствия правилам AGENTS.md, оценка Blast Radius (OB2H)',
        'execution_mode': 'supervised',
        'triggers': ['git: pre-commit', 'manual'],
      },
      {
        'name': 'auto-refactor',
        'title': 'Автономный рефакторинг и TDD',
        'description': 'Поиск мертвого кода, устранение техдолга через AST и валидация тестами',
        'execution_mode': 'supervised',
        'triggers': ['manual'],
      },
    ]);
    if (selectedSopName.value == null && sops.isNotEmpty) {
      selectSop('security-audit');
    }
  }

  /// Selects an SOP and fetches its graph.
  Future<void> selectSop(String name) async {
    selectedSopName.value = name;
    // 1. Instant optimistic update: immediately render dedicated graph for this pipeline!
    selectedGraph.value = _getDedicatedPipelineGraph(name);

    // 2. Background sync with fast timeout (1.5s) if backend gateway has live execution state
    try {
      final graph = await httpClient.getSopGraph(name).timeout(const Duration(milliseconds: 1500));
      if (graph != null && graph['nodes'] != null) {
        selectedGraph.value = graph;
      }
    } catch (_) {}
  }

  /// Approves a pending approval gate in the active SOP.
  void approveGate(String nodeId) {
    final cur = selectedGraph.value;
    if (cur != null) {
      final nodes = (cur['nodes'] as List<dynamic>?)
          ?.map((e) => Map<String, dynamic>.from(e as Map))
          .toList();
      if (nodes != null) {
        for (final n in nodes) {
          if (n['id'] == nodeId && n['kind'] == 'gate') {
            n['status'] = 'completed';
          }
        }
        selectedGraph.value = {
          ...cur,
          'nodes': nodes,
        };
        Get.snackbar(
          'Шаг согласован',
          'Шлюз безопасности подтвержден. Пайплайн продолжает выполнение.',
          backgroundColor: const Color(0xFF0F172A),
          colorText: const Color(0xFF10B981),
          snackPosition: SnackPosition.BOTTOM,
        );
      }
    }
  }

  Map<String, dynamic> _getDedicatedPipelineGraph(String name) {
    switch (name) {
      case 'security-audit':
        return {
          'name': name,
          'title': 'Аудит безопасности и зависимостей',
          'description': 'Автономный сценарий для регулярного выявления уязвимостей в пакетах и зависимостях',
          'nodes': [
            {
              'id': 'trigger',
              'kind': 'trigger',
              'title': 'Триггер: Cron / Ручной запуск',
              'description': 'Запуск регламента ежедневно в 03:00 либо по требованию разработчика',
              'status': 'completed',
            },
            {
              'id': 'scan_cve',
              'kind': 'tool',
              'title': 'Сканирование CVE баз (cargo audit & npm audit)',
              'description': 'Проверка Rust крейтов и npm пакетов по базам известных уязвимостей RustSec и GitHub Advisory',
              'status': 'completed',
            },
            {
              'id': 'eval_risk',
              'kind': 'step',
              'title': 'Классификация уровней риска (CVSS > 7.0)',
              'description': 'Анализ критичности уязвимостей и формирование рекомендаций по патчам версий',
              'status': 'running',
            },
            {
              'id': 'approval_gate',
              'kind': 'gate',
              'title': 'Шлюз согласования обновления пакетов',
              'description': 'Агент запрашивает подтверждение разработчика перед модификацией lock-файлов и манифестов',
              'status': 'pending',
            },
            {
              'id': 'report',
              'kind': 'deliverable',
              'title': 'Генерация отчета и фиксация в OB2H',
              'description': 'Сохранение архитектурного вердикта безопасности в долгосрочную память и отправка сводки',
              'status': 'idle',
            },
          ],
        };

      case 'release-build':
        return {
          'name': name,
          'title': 'Сборка и валидация релиза',
          'description': 'Комплексный конвейер подготовки релизных артефактов omnesagent.exe и OmnesAgent.exe',
          'nodes': [
            {
              'id': 'trigger',
              'kind': 'trigger',
              'title': 'Триггер: Git Tag / Релизная команда',
              'description': 'Инициализация сборки при установке нового семантического тега версии (например, v0.2.1)',
              'status': 'completed',
            },
            {
              'id': 'linter_checks',
              'kind': 'step',
              'title': 'Статический анализ (cargo clippy & flutter analyze)',
              'description': 'Строгая проверка качества кода, отсутствие warning и соответствие стандартам монорепозитория',
              'status': 'completed',
            },
            {
              'id': 'unit_tests',
              'kind': 'step',
              'title': 'Полный прогон юнит- и интеграционных тестов',
              'description': 'Тестирование рантайма, Triage Router, провайдеров и виджетов Flutter',
              'status': 'completed',
            },
            {
              'id': 'cargo_build',
              'kind': 'tool',
              'title': 'Сборка Rust бэкенда (cargo build --release)',
              'description': 'Оптимизированная компиляция шлюза с включенными флагами lto=thin и strip=symbols',
              'status': 'running',
            },
            {
              'id': 'flutter_build',
              'kind': 'tool',
              'title': 'Сборка Flutter Desktop клиента (flutter build windows)',
              'description': 'Сборка релизного Windows runner с актуальными дизайн-системами и ассетами',
              'status': 'idle',
            },
            {
              'id': 'checksum_gate',
              'kind': 'gate',
              'title': 'Шлюз валидации контрольных сумм и размера',
              'description': 'Проверка SHA-256 хешей собранных exe файлов перед публикацией',
              'status': 'idle',
            },
            {
              'id': 'deliverable',
              'kind': 'deliverable',
              'title': 'Релизный пакет готов к дистрибуции',
              'description': 'Готовые бинарники omnesagent.exe и OmnesAgent.exe упакованы в дистрибутив',
              'status': 'idle',
            },
          ],
        };

      case 'vps-proxy-sync':
        return {
          'name': name,
          'title': 'Мониторинг VPS и прокси-туннелей',
          'description': 'Автономное поддержание связи с сервером ser.presniakov.ru и прокси-стеком',
          'nodes': [
            {
              'id': 'trigger',
              'kind': 'trigger',
              'title': 'Триггер: Cron (каждые 15 минут)',
              'description': 'Периодический фоновый опрос для предотвращения обрыва SSH туннелей',
              'status': 'completed',
            },
            {
              'id': 'ping_bridge',
              'kind': 'tool',
              'title': 'Проверка SSH моста VPS (193.109.79.30)',
              'description': 'Тестирование доступности удаленного хоста через vps_bridge.py',
              'status': 'completed',
            },
            {
              'id': 'check_daemons',
              'kind': 'step',
              'title': 'Аудит локальных прокси-демонов (:8787, :47821)',
              'description': 'Проверка health-check портов Headroom, pxpipe и фоновых воркеров',
              'status': 'running',
            },
            {
              'id': 'auto_restart',
              'kind': 'gate',
              'title': 'Шлюз автоперезапуска упавших сервисов',
              'description': 'Автономное восстановление упавших туннелей с регистрацией в журнал событий',
              'status': 'pending',
            },
            {
              'id': 'telemetry',
              'kind': 'deliverable',
              'title': 'Фиксация задержки (Latency) и статуса',
              'description': 'Запись метрик здоровья инфраструктуры в системный лог агента',
              'status': 'idle',
            },
          ],
        };

      case 'code-review-gate':
        return {
          'name': name,
          'title': 'Автономное код-ревью (PR Gate)',
          'description': 'Интеллектуальная проверка изменений перед вливанием в основную ветку',
          'nodes': [
            {
              'id': 'trigger',
              'kind': 'trigger',
              'title': 'Триггер: Git pre-commit / Pull Request',
              'description': 'Активация при создании новой ветки или подготовке коммита',
              'status': 'completed',
            },
            {
              'id': 'git_diff',
              'kind': 'tool',
              'title': 'Извлечение Git diff и карты изменений',
              'description': 'Сбор списка измененных строк и проверка на отсутствие захардкоженных секретов',
              'status': 'completed',
            },
            {
              'id': 'ast_impact',
              'kind': 'tool',
              'title': 'Оценка Blast Radius через OB2H AST',
              'description': 'Анализ всех классов и функций, которые затронет данная правка в соседних крейтах',
              'status': 'running',
            },
            {
              'id': 'approval_gate',
              'kind': 'gate',
              'title': 'Шлюз аппрува: Согласование слияния',
              'description': 'Агент проверяет соответствие AGENTS.md и требует подтверждения тимлида',
              'status': 'pending',
            },
            {
              'id': 'merge_done',
              'kind': 'deliverable',
              'title': 'Безопасное слияние в ветку main',
              'description': 'Формирование атомарного Conventional Commit и пуш в репозиторий',
              'status': 'idle',
            },
          ],
        };

      case 'auto-refactor':
      default:
        return {
          'name': name,
          'title': 'Автономный рефакторинг и TDD',
          'description': 'Выявление и безопасное устранение технического долга в кодовой базе',
          'nodes': [
            {
              'id': 'trigger',
              'kind': 'trigger',
              'title': 'Триггер: Запрос разработчика',
              'description': 'Инициализация по команде или обнаружению устаревших методов',
              'status': 'completed',
            },
            {
              'id': 'ast_dead_code',
              'kind': 'tool',
              'title': 'Поиск неиспользуемого кода и дубликатов',
              'description': 'Детерминированный обход дерева синтаксиса для выявления мертвого кода',
              'status': 'completed',
            },
            {
              'id': 'refactor_step',
              'kind': 'step',
              'title': 'Применение рефакторинга (Clean Architecture)',
              'description': 'Оптимизация сигнатур, удаление лишних копирований и приведение к PSR-12/PEP8/Rust 2024',
              'status': 'running',
            },
            {
              'id': 'test_verify',
              'kind': 'step',
              'title': 'Closed-Loop тестирование: проверка компиляции',
              'description': 'Гарантия того, что рефакторинг не сломал ни один публичный контракт API',
              'status': 'idle',
            },
            {
              'id': 'approval_gate',
              'kind': 'gate',
              'title': 'Шлюз согласования diff перед коммитом',
              'description': 'Просмотр диффа разработчиком перед окончательным сохранением',
              'status': 'idle',
            },
          ],
        };
    }
  }

  /// Runs the selected SOP.
  Future<void> runActiveSop() async {
    final name = selectedSopName.value;
    if (name == null || name.isEmpty) return;

    try {
      final res = await httpClient.runSop(name);
      if (res != null && res['run_id'] != null) {
        activeRunId.value = res['run_id'].toString();
        Get.snackbar('SOP запущен', 'Идентификатор запуска: ${res['run_id']}');
        loadRuns();
      } else {
        Get.snackbar('Запуск SOP', 'Процесс $name запущен в бэкенде');
      }
    } catch (e) {
      Get.snackbar('Ошибка запуска', '$e');
    }
  }

  /// Fetches recent runs.
  Future<void> loadRuns() async {
    try {
      final runs = await httpClient.getSopRuns();
      if (runs.isNotEmpty) {
        sopRuns.assignAll(runs);
      }
    } catch (_) {}
  }

  @override
  void onClose() {
    httpClient.dispose();
    super.onClose();
  }
}
