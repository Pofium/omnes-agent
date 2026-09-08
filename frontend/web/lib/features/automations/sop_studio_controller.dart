// SOP Studio Controller for managing and executing SOP pipelines via OmnesAgent Gateway.

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
        'title': 'Аудит безопасности зависимостей',
        'description': 'Сканирование cargo audit / npm audit и генерация отчёта',
        'execution_mode': 'supervised',
        'triggers': ['manual', 'cron'],
      },
      {
        'name': 'release-build',
        'title': 'Сборка и проверка релизного бинарника',
        'description': 'Запуск flutter build windows, проверка exe и тестов',
        'execution_mode': 'autonomous',
        'triggers': ['manual'],
      },
      {
        'name': 'vps-proxy-sync',
        'title': 'Синхронизация прокси и туннелей',
        'description': 'Проверка SOCKS5/SSH моста и перезапуск упавших демонов',
        'execution_mode': 'autonomous',
        'triggers': ['cron'],
      },
    ]);
    if (selectedSopName.value == null && sops.isNotEmpty) {
      selectSop('security-audit');
    }
  }

  /// Selects an SOP and fetches its graph.
  Future<void> selectSop(String name) async {
    selectedSopName.value = name;
    try {
      final graph = await httpClient.getSopGraph(name);
      if (graph != null) {
        selectedGraph.value = graph;
      } else {
        _setMockGraph(name);
      }
    } catch (_) {
      _setMockGraph(name);
    }
  }

  void _setMockGraph(String name) {
    selectedGraph.value = {
      'name': name,
      'nodes': [
        {'id': 'trigger', 'kind': 'trigger', 'title': 'Триггер: Ручной запуск', 'status': 'completed'},
        {'id': 'scan_ast', 'kind': 'tool', 'title': 'AST анализ графа (ob2h)', 'status': 'completed'},
        {'id': 'verify_code', 'kind': 'step', 'title': 'Проверка компиляции и тестов', 'status': 'running'},
        {'id': 'approval_gate', 'kind': 'gate', 'title': 'Согласование деструктивных шагов', 'status': 'pending'},
        {'id': 'deploy', 'kind': 'step', 'title': 'Фиксация артефактов', 'status': 'idle'},
      ],
      'edges': [
        {'from': 'trigger', 'to': 'scan_ast'},
        {'from': 'scan_ast', 'to': 'verify_code'},
        {'from': 'verify_code', 'to': 'approval_gate'},
        {'from': 'approval_gate', 'to': 'deploy'},
      ],
    };
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
