// Model representing token usage and costs from GET /api/cost.

class CostSummary {
  final int totalTokens;
  final double totalCostUsd;
  final int requestCount;
  final Map<String, ModelUsage> byModel;

  const CostSummary({
    required this.totalTokens,
    required this.totalCostUsd,
    required this.requestCount,
    required this.byModel,
  });

  factory CostSummary.fromJson(Map<String, dynamic> json) {
    final rawByModel = json['by_model'] as Map? ?? {};
    final models = <String, ModelUsage>{};

    int sumTokens = 0;
    double sumCost = 0.0;
    int sumReqs = 0;

    rawByModel.forEach((key, val) {
      if (val is Map) {
        final usage = ModelUsage.fromJson(Map<String, dynamic>.from(val));
        models[key.toString()] = usage;
        sumTokens += usage.totalTokens;
        sumCost += usage.costUsd;
        sumReqs += usage.requestCount;
      }
    });

    final totalTok = json['total_tokens'] != null ? (json['total_tokens'] as num).toInt() : sumTokens;
    final totalC = json['monthly_cost_usd'] != null
        ? (json['monthly_cost_usd'] as num).toDouble()
        : (json['session_cost_usd'] != null ? (json['session_cost_usd'] as num).toDouble() : sumCost);
    final reqCount = json['request_count'] != null ? (json['request_count'] as num).toInt() : sumReqs;

    return CostSummary(
      totalTokens: totalTok > 0 ? totalTok : sumTokens,
      totalCostUsd: totalC > 0 ? totalC : sumCost,
      requestCount: reqCount > 0 ? reqCount : sumReqs,
      byModel: models,
    );
  }
}

class ModelUsage {
  final String model;
  final int inputTokens;
  final int outputTokens;
  final int totalTokens;
  final double costUsd;
  final int requestCount;

  const ModelUsage({
    required this.model,
    required this.inputTokens,
    required this.outputTokens,
    required this.totalTokens,
    required this.costUsd,
    required this.requestCount,
  });

  factory ModelUsage.fromJson(Map<String, dynamic> json) {
    return ModelUsage(
      model: (json['model'] ?? '').toString(),
      inputTokens: json['input_tokens'] != null ? (json['input_tokens'] as num).toInt() : 0,
      outputTokens: json['output_tokens'] != null ? (json['output_tokens'] as num).toInt() : 0,
      totalTokens: json['total_tokens'] != null ? (json['total_tokens'] as num).toInt() : 0,
      costUsd: json['cost_usd'] != null ? (json['cost_usd'] as num).toDouble() : 0.0,
      requestCount: json['request_count'] != null ? (json['request_count'] as num).toInt() : 0,
    );
  }
}
