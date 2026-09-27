/// Represents token usage breakdown across prompt components and compression savings.
class ContextBudgetSnapshot {
  final int systemTokens;
  final int kagAstTokens;
  final int memoryTokens;
  final int dialogHistoryTokens;
  final int toolOutputsTokens;
  final int compressionSavedTokens;
  final int totalUsedTokens;
  final int maxWindowTokens;
  final double savingsPercent;

  const ContextBudgetSnapshot({
    this.systemTokens = 0,
    this.kagAstTokens = 0,
    this.memoryTokens = 0,
    this.dialogHistoryTokens = 0,
    this.toolOutputsTokens = 0,
    this.compressionSavedTokens = 0,
    this.totalUsedTokens = 0,
    this.maxWindowTokens = 128000,
    this.savingsPercent = 0.0,
  });

  factory ContextBudgetSnapshot.fromJson(Map<String, dynamic> json) {
    return ContextBudgetSnapshot(
      systemTokens: json['system_tokens'] as int? ?? 0,
      kagAstTokens: json['kag_ast_tokens'] as int? ?? 0,
      memoryTokens: json['memory_tokens'] as int? ?? 0,
      dialogHistoryTokens: json['dialog_history_tokens'] as int? ?? 0,
      toolOutputsTokens: json['tool_outputs_tokens'] as int? ?? 0,
      compressionSavedTokens: json['compression_saved_tokens'] as int? ?? 0,
      totalUsedTokens: json['total_used_tokens'] as int? ?? 0,
      maxWindowTokens: json['max_window_tokens'] as int? ?? 128000,
      savingsPercent: (json['savings_percent'] as num?)?.toDouble() ?? 0.0,
    );
  }

  Map<String, dynamic> toJson() => {
    'system_tokens': systemTokens,
    'kag_ast_tokens': kagAstTokens,
    'memory_tokens': memoryTokens,
    'dialog_history_tokens': dialogHistoryTokens,
    'tool_outputs_tokens': toolOutputsTokens,
    'compression_saved_tokens': compressionSavedTokens,
    'total_used_tokens': totalUsedTokens,
    'max_window_tokens': maxWindowTokens,
    'savings_percent': savingsPercent,
  };
}
