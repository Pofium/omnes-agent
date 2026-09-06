// Repository for retrieving LLM cost and token statistics via /api/cost.

import '../../core/gateway/gateway_config.dart';
import '../../core/gateway/gateway_http.dart';
import 'models/cost_summary.dart';

class StatsRepository {
  final GatewayHttpClient _http;

  StatsRepository({GatewayHttpClient? http}) : _http = http ?? GatewayHttpClient();

  Future<CostSummary?> getCostSummary({
    String? agentAlias,
    DateTime? from,
    DateTime? to,
  }) async {
    final alias = (agentAlias != null && agentAlias.isNotEmpty)
        ? agentAlias
        : await GatewayConfig.getActiveAgent();

    final data = await _http.costSummary(
      agentAlias: alias,
      from: from?.toUtc().toIso8601String(),
      to: to?.toUtc().toIso8601String(),
    );

    if (data != null) {
      return CostSummary.fromJson(data);
    }
    return null;
  }

  void dispose() {
    _http.dispose();
  }
}
