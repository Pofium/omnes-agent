/// Execution mode for the agent runtime (DeepSeek Harness / OmnesAgent profiles).
/// [icon] holds a `UiIcon` key (`design_system/ui_icons.dart`) — emoji are
/// forbidden in the UI (AGENTS.md, "UI Conventions — Icons").
enum AgentExecutionMode {
  fast(
    'fast',
    'Speed / Fast',
    'bolt',
    'Приоритет локального S1 (Laya), жесткое сжатие контекста, мгновенный ответ',
  ),
  deepCode(
    'deep_code',
    'Deep Code',
    'code',
    'Глубокий KAG AST граф, автопрогон проверок компилятора',
  ),
  architect(
    'architect',
    'Architect / CoT',
    'brain',
    'Максимальный бюджет рассуждений, двухфазная верификация',
  ),
  ralphLoop(
    'ralph_loop',
    'Ralph Loop',
    'repeat',
    'Автономный мульти-итеративный цикл с фиксацией вердиктов',
  );

  final String id;
  final String label;
  final String icon;
  final String description;

  const AgentExecutionMode(this.id, this.label, this.icon, this.description);

  static AgentExecutionMode fromId(String id) {
    for (final mode in AgentExecutionMode.values) {
      if (mode.id == id) return mode;
    }
    return AgentExecutionMode.fast;
  }
}
