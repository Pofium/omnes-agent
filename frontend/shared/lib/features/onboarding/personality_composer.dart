// Deterministic composer for OmnesAgent personality files based on onboarding answers.
import 'personality_templates.dart';

/// User answers gathered during the interactive first-run questionnaire.
class OnboardingAnswers {
  final String userName;
  final String role;
  final String primaryStack;
  final String language;
  final String autonomyStyle;
  final String agentTone;
  final String agentName;
  final DateTime createdDate;
  final bool enableAstMemory;

  OnboardingAnswers({
    required this.userName,
    required this.role,
    required this.primaryStack,
    required this.language,
    required this.autonomyStyle,
    required this.agentTone,
    this.agentName = 'omnes',
    DateTime? createdDate,
    this.enableAstMemory = true,
  }) : createdDate = createdDate ?? DateTime.now();

  OnboardingAnswers copyWith({
    String? userName,
    String? role,
    String? primaryStack,
    String? language,
    String? autonomyStyle,
    String? agentTone,
    String? agentName,
    DateTime? createdDate,
    bool? enableAstMemory,
  }) {
    return OnboardingAnswers(
      userName: userName ?? this.userName,
      role: role ?? this.role,
      primaryStack: primaryStack ?? this.primaryStack,
      language: language ?? this.language,
      autonomyStyle: autonomyStyle ?? this.autonomyStyle,
      agentTone: agentTone ?? this.agentTone,
      agentName: agentName ?? this.agentName,
      createdDate: createdDate ?? this.createdDate,
      enableAstMemory: enableAstMemory ?? this.enableAstMemory,
    );
  }
}

/// Pure deterministic composer producing personality files.
/// Zero LLM calls, zero network I/O, strictly compliant with MAX_FILE_CHARS = 20,000.
class PersonalityComposer {
  /// Maximum characters per file allowed by the backend runtime (omnesagent-runtime/agent/personality.rs:8).
  static const int maxFileChars = 20000;

  /// Composes the 4 core personality files (SOUL.md, IDENTITY.md, USER.md, MEMORY.md).
  static Map<String, String> compose(OnboardingAnswers answers) {
    final soul = renderSoulTemplate(
      userName: answers.userName.trim(),
      agentName: answers.agentName.trim(),
      tone: answers.agentTone,
      autonomy: answers.autonomyStyle,
      language: answers.language,
    );

    final identity = renderIdentityTemplate(
      agentName: answers.agentName.trim(),
      userName: answers.userName.trim(),
      role: answers.role.trim(),
      primaryStack: answers.primaryStack.trim(),
      autonomy: answers.autonomyStyle,
      createdDate: answers.createdDate,
      language: answers.language,
    );

    final user = renderUserTemplate(
      userName: answers.userName.trim(),
      role: answers.role.trim(),
      primaryStack: answers.primaryStack.trim(),
      language: answers.language,
      autonomy: answers.autonomyStyle,
      enableAstMemory: answers.enableAstMemory,
      createdDate: answers.createdDate,
    );

    final memory = renderMemoryTemplate(
      userName: answers.userName.trim(),
      role: answers.role.trim(),
      primaryStack: answers.primaryStack.trim(),
      autonomy: answers.autonomyStyle,
      createdDate: answers.createdDate,
      language: answers.language,
    );

    return {
      'SOUL.md': _clamp(soul),
      'IDENTITY.md': _clamp(identity),
      'USER.md': _clamp(user),
      'MEMORY.md': _clamp(memory),
    };
  }

  static String _clamp(String text) {
    if (text.length > maxFileChars) {
      return text.substring(0, maxFileChars);
    }
    return text;
  }
}
