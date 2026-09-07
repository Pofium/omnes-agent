// Controller for managing token consumption and cost statistics.

import 'package:get/get.dart';
import '../../core/gateway/gateway_config.dart';
import 'models/cost_summary.dart';
import 'stats_repository.dart';

class StatsController extends GetxController {
  final StatsRepository _repo;

  StatsController({StatsRepository? repo}) : _repo = repo ?? StatsRepository();

  final Rx<CostSummary?> costSummary = Rx<CostSummary?>(null);
  final RxBool isLoading = false.obs;
  final RxString activeAgent = ''.obs;
  final RxString selectedTimeframe = 'all'.obs; // 'all', 'today', 'week', 'month'

  @override
  void onInit() {
    super.onInit();
    loadStats();
  }

  /// Loads cost statistics for selected period.
  Future<void> loadStats() async {
    isLoading.value = true;
    try {
      activeAgent.value = await GatewayConfig.getActiveAgent();
      DateTime? from;
      final now = DateTime.now();

      if (selectedTimeframe.value == 'today') {
        from = DateTime(now.year, now.month, now.day);
      } else if (selectedTimeframe.value == 'week') {
        from = now.subtract(const Duration(days: 7));
      } else if (selectedTimeframe.value == 'month') {
        from = now.subtract(const Duration(days: 30));
      }

      final summary = await _repo.getCostSummary(
        agentAlias: activeAgent.value,
        from: from,
      );
      costSummary.value = summary;
    } catch (_) {
    } finally {
      isLoading.value = false;
    }
  }

  void setTimeframe(String timeframe) {
    if (selectedTimeframe.value == timeframe) return;
    selectedTimeframe.value = timeframe;
    loadStats();
  }

  @override
  void onClose() {
    _repo.dispose();
    super.onClose();
  }
}
