// Interactive first-run onboarding dialog with deterministic agent personality configuration.
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get_storage/get_storage.dart';

import '../../core/gateway/gateway_http.dart';
import '../../core/gateway/quickstart_submission.dart';
import '../../design_system/shadcn_colors.dart';
import '../../utils/desktop_i18n.dart';
import 'personality_composer.dart';

/// User profile data model.
class UserProfileData {
  String firstName;
  String lastName;
  String role;
  String primaryStack;
  String autonomyStyle;
  String agentTone;
  String language;
  bool enableAstMemory;

  UserProfileData({
    this.firstName = '',
    this.lastName = '',
    this.role = 'Tech Lead / AI Engineer',
    this.primaryStack = 'Rust / Dart / Python',
    this.autonomyStyle = 'Ask before changes (спрашивать перед правками)',
    this.agentTone = 'Дружелюбный',
    this.language = 'Русский',
    this.enableAstMemory = true,
  });

  String get fullName => '$firstName $lastName'.trim();
  String get initials => '${firstName.isNotEmpty ? firstName[0] : ''}${lastName.isNotEmpty ? lastName[0] : ''}'.toUpperCase();

  Map<String, dynamic> toJson() => {
    'firstName': firstName,
    'lastName': lastName,
    'role': role,
    'primaryStack': primaryStack,
    'autonomyStyle': autonomyStyle,
    'agentTone': agentTone,
    'language': language,
    'enableAstMemory': enableAstMemory,
  };

  factory UserProfileData.fromJson(Map<String, dynamic> json) => UserProfileData(
    firstName: json['firstName'] as String? ?? '',
    lastName: json['lastName'] as String? ?? '',
    role: json['role'] as String? ?? 'Tech Lead / AI Engineer',
    primaryStack: json['primaryStack'] as String? ?? 'Rust / Dart / Python',
    autonomyStyle: json['autonomyStyle'] as String? ?? 'Ask before changes (спрашивать перед правками)',
    agentTone: json['agentTone'] as String? ?? 'Дружелюбный',
    language: json['language'] as String? ?? 'Русский',
    enableAstMemory: json['enableAstMemory'] as bool? ?? true,
  );
}

class UserOnboardingDialog extends StatefulWidget {
  final UserProfileData initialProfile;
  final Function(UserProfileData) onSave;

  const UserOnboardingDialog({
    super.key,
    required this.initialProfile,
    required this.onSave,
  });

  static Future<UserProfileData?> show(
    BuildContext context, {
    required UserProfileData initialProfile,
    required Function(UserProfileData) onSave,
  }) {
    return showDialog<UserProfileData>(
      context: context,
      barrierColor: Colors.black.withOpacity(0.8),
      barrierDismissible: false,
      builder: (ctx) => Dialog(
        backgroundColor: Colors.transparent,
        insetPadding: const EdgeInsets.symmetric(horizontal: 32, vertical: 24),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(16),
          child: Container(
            constraints: const BoxConstraints(maxWidth: 760, maxHeight: 720),
            decoration: BoxDecoration(
              color: ShadcnColors.surface,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: ShadcnColors.border),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.5),
                  blurRadius: 30,
                  spreadRadius: 8,
                ),
                BoxShadow(
                  color: ShadcnColors.primary.withOpacity(0.08),
                  blurRadius: 40,
                  spreadRadius: 1,
                ),
              ],
            ),
            child: UserOnboardingDialog(
              initialProfile: initialProfile,
              onSave: onSave,
            ),
          ),
        ),
      ),
    );
  }

  @override
  State<UserOnboardingDialog> createState() => _UserOnboardingDialogState();
}

class _UserOnboardingDialogState extends State<UserOnboardingDialog> {
  int currentStep = 0;
  final int totalSteps = 5;
  late final String runId;

  // Controllers
  late final TextEditingController firstNameController;
  late final TextEditingController lastNameController;
  late final TextEditingController customRoleController;
  late final TextEditingController customStackController;

  // Step 3 (Model/Channel) Controllers
  late final TextEditingController modelNameController;
  late final TextEditingController apiKeyController;
  late final TextEditingController telegramTokenController;

  // State
  late String selectedRole;
  late String selectedStack;
  late String selectedAutonomy;
  late String selectedTone;
  late String selectedLanguage;
  late bool enableAstMemory;
  String selectedProviderType = 'anthropic';

  // Step 4 Personality preview map (filename -> text controller)
  final Map<String, TextEditingController> personalityControllers = {};
  String activePreviewFile = 'SOUL.md';

  // Validation / Error tracking
  String? nameError;
  String? backendErrorSummary;
  final Map<String, String> inlineErrors = {};
  bool isSubmitting = false;

  final List<String> standardRoles = [
    'Tech Lead / AI Engineer',
    'Backend Developer',
    'Frontend Developer',
    'Fullstack Engineer',
    'DevOps / SRE',
    'Data Scientist / ML Engineer',
    'Student / Enthusiast',
    'Другое…',
  ];

  final List<String> standardStacks = [
    'Rust / Dart / Python',
    'TypeScript / Node.js / React',
    'Go / Cloud Native',
    'Python / AI / ML',
    'C++ / Systems',
    'Java / Kotlin / Android',
    'Swift / iOS',
    'Другое…',
  ];

  final List<String> languages = [
    'Русский',
    'English',
    'Deutsch',
    'Español',
    'Français',
    '中文',
  ];

  @override
  void initState() {
    super.initState();
    runId = 'qs-${DateTime.now().millisecondsSinceEpoch}';
    firstNameController = TextEditingController(text: widget.initialProfile.firstName);
    lastNameController = TextEditingController(text: widget.initialProfile.lastName);
    customRoleController = TextEditingController();
    customStackController = TextEditingController();

    modelNameController = TextEditingController(text: 'claude-sonnet-4-5');
    apiKeyController = TextEditingController();
    telegramTokenController = TextEditingController();

    selectedRole = standardRoles.contains(widget.initialProfile.role)
        ? widget.initialProfile.role
        : 'Tech Lead / AI Engineer';
    selectedStack = standardStacks.contains(widget.initialProfile.primaryStack)
        ? widget.initialProfile.primaryStack
        : 'Rust / Dart / Python';
    selectedAutonomy = widget.initialProfile.autonomyStyle;
    selectedTone = widget.initialProfile.agentTone;
    selectedLanguage = widget.initialProfile.language;
    enableAstMemory = widget.initialProfile.enableAstMemory;

    _regeneratePersonalityDrafts();
  }

  @override
  void dispose() {
    firstNameController.dispose();
    lastNameController.dispose();
    customRoleController.dispose();
    customStackController.dispose();
    modelNameController.dispose();
    apiKeyController.dispose();
    telegramTokenController.dispose();
    for (final c in personalityControllers.values) {
      c.dispose();
    }
    super.dispose();
  }

  String get effectiveRole => selectedRole == 'Другое…' && customRoleController.text.trim().isNotEmpty
      ? customRoleController.text.trim()
      : selectedRole;

  String get effectiveStack => selectedStack == 'Другое…' && customStackController.text.trim().isNotEmpty
      ? customStackController.text.trim()
      : selectedStack;

  void _regeneratePersonalityDrafts() {
    final answers = OnboardingAnswers(
      userName: firstNameController.text.trim().isNotEmpty
          ? '${firstNameController.text.trim()} ${lastNameController.text.trim()}'.trim()
          : (widget.initialProfile.fullName.isNotEmpty ? widget.initialProfile.fullName : 'Пользователь'),
      role: effectiveRole,
      primaryStack: effectiveStack,
      language: selectedLanguage,
      autonomyStyle: selectedAutonomy,
      agentTone: selectedTone,
      agentName: 'omnes',
      enableAstMemory: enableAstMemory,
    );

    final files = PersonalityComposer.compose(answers);
    for (final entry in files.entries) {
      if (personalityControllers.containsKey(entry.key)) {
        personalityControllers[entry.key]!.text = entry.value;
      } else {
        personalityControllers[entry.key] = TextEditingController(text: entry.value);
      }
    }
  }

  void _onNext() {
    if (currentStep == 0) {
      if (firstNameController.text.trim().isEmpty) {
        setState(() {
          nameError = DesktopI18n.tr('Пожалуйста, введите ваше имя', 'Please enter your name');
        });
        return;
      } else {
        setState(() => nameError = null);
      }
    }

    if (currentStep == 2) {
      _regeneratePersonalityDrafts();
    }

    if (currentStep < totalSteps - 1) {
      setState(() {
        currentStep++;
        backendErrorSummary = null;
        inlineErrors.clear();
      });
    }
  }

  void _onBack() {
    if (currentStep > 0) {
      setState(() {
        currentStep--;
        backendErrorSummary = null;
        inlineErrors.clear();
      });
    }
  }

  Future<void> _onDismiss() async {
    try {
      final http = GatewayHttpClient();
      await http.dismissQuickstart(
        runId: runId,
        surface: 'web',
        lastStep: 'step_$currentStep',
      );
    } catch (_) {}
    if (mounted) {
      Navigator.of(context).pop(null);
    }
  }

  BuilderSubmissionDto _buildSubmission({bool usePlaceholderModel = false}) {
    final personalityFiles = personalityControllers.entries.map((e) {
      return QuickstartPersonalityFileDto(
        filename: e.key,
        content: e.value.text,
      );
    }).toList();

    final userFullName = '${firstNameController.text.trim()} ${lastNameController.text.trim()}'.trim();

    final hasApiKey = apiKeyController.text.trim().isNotEmpty;
    final modelProviderChoice = ModelProviderChoiceDto(
      providerType: selectedProviderType,
      alias: selectedProviderType,
      model: modelNameController.text.trim().isNotEmpty
          ? modelNameController.text.trim()
          : (selectedProviderType == 'anthropic' ? 'claude-sonnet-4-5' : 'gpt-4o'),
      fields: {
        'api_key': (hasApiKey && !usePlaceholderModel)
            ? apiKeyController.text.trim()
            : 'placeholder-api-key',
      },
    );

    final channels = <SelectorChoiceDto<ChannelQuickStartDto>>[];
    if (telegramTokenController.text.trim().isNotEmpty && !usePlaceholderModel) {
      channels.add(
        SelectorChoiceDto.fresh(
          ChannelQuickStartDto(
            channelType: 'telegram',
            alias: 'tg',
            fields: {'bot_token': telegramTokenController.text.trim()},
          ),
        ),
      );
    }

    return BuilderSubmissionDto(
      modelProvider: SelectorChoiceDto.fresh(modelProviderChoice),
      riskProfile: const SelectorChoiceDto.fresh('balanced'),
      runtimeProfile: const SelectorChoiceDto.fresh('balanced'),
      memory: const SelectorChoiceDto.fresh('sqlite'),
      channels: channels,
      peerGroups: const [],
      agent: AgentIdentityDto(
        name: 'omnes',
        systemPrompt: 'You are Omnes, the personal agent of $userFullName. Details live in your personality files.',
        personalityFile: null,
        personalityFiles: personalityFiles,
      ),
    );
  }

  Future<void> _submitProfile({bool skipModel = false}) async {
    setState(() {
      isSubmitting = true;
      backendErrorSummary = null;
      inlineErrors.clear();
    });

    final submission = _buildSubmission(usePlaceholderModel: skipModel);
    final http = GatewayHttpClient();

    try {
      // 1. Validate submission
      final valResult = await http.validateQuickstartSubmission(submission);
      if (!valResult.isOk) {
        setState(() {
          isSubmitting = false;
          backendErrorSummary = DesktopI18n.tr(
            'Ошибка валидации параметров агента:',
            'Agent configuration validation failed:',
          );
          for (final err in valResult.errors) {
            inlineErrors['${err.step}.${err.field}'] = err.message;
          }
        });
        return;
      }

      // 2. Apply submission atomically
      final applyResult = await http.submitQuickstart(submission);
      if (!applyResult.isApplied) {
        setState(() {
          isSubmitting = false;
          backendErrorSummary = DesktopI18n.tr(
            'Ошибка применения конфигурации шлюзом:',
            'Gateway failed to apply configuration:',
          );
          for (final err in applyResult.errors) {
            inlineErrors['${err.step}.${err.field}'] = err.message;
          }
        });
        return;
      }

      // 3. Save profile locally only after successful apply
      final updatedProfile = UserProfileData(
        firstName: firstNameController.text.trim(),
        lastName: lastNameController.text.trim(),
        role: effectiveRole,
        primaryStack: effectiveStack,
        autonomyStyle: selectedAutonomy,
        agentTone: selectedTone,
        language: selectedLanguage,
        enableAstMemory: enableAstMemory,
      );

      try {
        GetStorage().write('user_profile', updatedProfile.toJson());
        await http.memoryStore(
          key: 'user_profile',
          content: jsonEncode({
            'name': updatedProfile.fullName,
            'role': updatedProfile.role,
            'stack': updatedProfile.primaryStack,
            'autonomy': updatedProfile.autonomyStyle,
            'tone': updatedProfile.agentTone,
            'language': updatedProfile.language,
            'ast_memory': updatedProfile.enableAstMemory,
          }),
        );
      } catch (_) {}

      widget.onSave(updatedProfile);

      if (mounted) {
        Navigator.of(context).pop(updatedProfile);
      }
    } catch (e) {
      setState(() {
        isSubmitting = false;
        backendErrorSummary = '${DesktopI18n.tr("Сетевая ошибка:", "Network error:")} $e';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        _buildTopHeader(),
        _buildStepIndicator(),
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 20),
            child: _buildCurrentStepContent(),
          ),
        ),
        if (backendErrorSummary != null) _buildErrorBanner(),
        _buildBottomActions(),
      ],
    );
  }

  Widget _buildTopHeader() {
    return Container(
      height: 64,
      padding: const EdgeInsets.symmetric(horizontal: 24),
      decoration: const BoxDecoration(
        color: ShadcnColors.card,
        border: Border(bottom: BorderSide(color: ShadcnColors.border)),
      ),
      child: Row(
        children: [
          Container(
            width: 32,
            height: 32,
            decoration: BoxDecoration(
              gradient: const LinearGradient(
                colors: [ShadcnColors.primaryGradientStart, ShadcnColors.primaryGradientEnd],
              ),
              borderRadius: BorderRadius.circular(8),
              boxShadow: [
                BoxShadow(
                  color: ShadcnColors.primaryGlow,
                  blurRadius: 10,
                  spreadRadius: 1,
                ),
              ],
            ),
            child: const Center(
              child: Icon(FontAwesomeIcons.robot, size: 16, color: Colors.black87),
            ),
          ),
          const SizedBox(width: 12),
          Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                DesktopI18n.tr('Инициализация OmnesAgent', 'OmnesAgent Initialization'),
                style: const TextStyle(
                  fontSize: 15,
                  fontWeight: FontWeight.bold,
                  color: ShadcnColors.foreground,
                ),
              ),
              Text(
                DesktopI18n.tr('Интерактивный опросник первого запуска', 'Interactive First-Run Questionnaire'),
                style: const TextStyle(fontSize: 11, color: ShadcnColors.foregroundMuted),
              ),
            ],
          ),
          const Spacer(),
          TextButton(
            onPressed: isSubmitting ? null : _onDismiss,
            style: TextButton.styleFrom(foregroundColor: ShadcnColors.foregroundMuted),
            child: Text(DesktopI18n.tr('Отложить', 'Dismiss')),
          ),
        ],
      ),
    );
  }

  Widget _buildStepIndicator() {
    final stepTitles = [
      DesktopI18n.tr('Имя', 'Name'),
      DesktopI18n.tr('Профиль', 'Profile'),
      DesktopI18n.tr('Поведение', 'Behavior'),
      DesktopI18n.tr('Модель', 'Model'),
      DesktopI18n.tr('Превью', 'Preview'),
    ];

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
      decoration: const BoxDecoration(
        color: ShadcnColors.background,
        border: Border(bottom: BorderSide(color: ShadcnColors.borderSubtle)),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: List.generate(totalSteps, (index) {
          final isDone = index < currentStep;
          final isActive = index == currentStep;

          Color dotColor = ShadcnColors.cardElevated;
          Color textColor = ShadcnColors.foregroundSubtle;
          if (isActive) {
            dotColor = ShadcnColors.primary;
            textColor = ShadcnColors.primary;
          } else if (isDone) {
            dotColor = ShadcnColors.success;
            textColor = ShadcnColors.foreground;
          }

          return Expanded(
            child: Row(
              children: [
                Container(
                  width: 24,
                  height: 24,
                  decoration: BoxDecoration(
                    color: dotColor.withOpacity(0.18),
                    border: Border.all(color: dotColor, width: isActive ? 2 : 1),
                    shape: BoxShape.circle,
                  ),
                  child: Center(
                    child: isDone
                        ? const Icon(Icons.check, size: 14, color: ShadcnColors.success)
                        : Text(
                            '${index + 1}',
                            style: TextStyle(
                              fontSize: 11,
                              fontWeight: FontWeight.bold,
                              color: textColor,
                            ),
                          ),
                  ),
                ),
                const SizedBox(width: 8),
                Text(
                  stepTitles[index],
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
                    color: textColor,
                  ),
                ),
                if (index < totalSteps - 1)
                  Expanded(
                    child: Container(
                      height: 1,
                      margin: const EdgeInsets.symmetric(horizontal: 8),
                      color: isDone ? ShadcnColors.success.withOpacity(0.4) : ShadcnColors.borderSubtle,
                    ),
                  ),
              ],
            ),
          );
        }),
      ),
    );
  }

  Widget _buildCurrentStepContent() {
    switch (currentStep) {
      case 0:
        return _buildStep0Welcome();
      case 1:
        return _buildStep1Profile();
      case 2:
        return _buildStep2Behavior();
      case 3:
        return _buildStep3ModelChannel();
      case 4:
        return _buildStep4Preview();
      default:
        return const SizedBox.shrink();
    }
  }

  // ── Step 0: Welcome and Name ─────────────────────────────────────────────
  Widget _buildStep0Welcome() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SizedBox(height: 12),
        Container(
          padding: const EdgeInsets.all(20),
          decoration: BoxDecoration(
            color: ShadcnColors.card,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: ShadcnColors.border),
          ),
          child: Row(
            children: [
              Container(
                width: 48,
                height: 48,
                decoration: BoxDecoration(
                  color: ShadcnColors.primary.withOpacity(0.12),
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(color: ShadcnColors.primary.withOpacity(0.3)),
                ),
                child: const Icon(FontAwesomeIcons.handshake, color: ShadcnColors.primary, size: 22),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      DesktopI18n.tr('Здравствуйте! Я ваш агент Omnes.', 'Hello! I am your Omnes agent.'),
                      style: const TextStyle(
                        fontSize: 18,
                        fontWeight: FontWeight.bold,
                        color: ShadcnColors.foreground,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      DesktopI18n.tr(
                        'Давайте познакомимся — как к вам обращаться? Я настрою свою личность и буду ориентироваться на ваши инженерные цели.',
                        "Let's get acquainted — how should I address you? I will configure my personality to align with your engineering workflow.",
                      ),
                      style: const TextStyle(fontSize: 13, color: ShadcnColors.foregroundMuted),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 28),
        _buildFieldLabel(DesktopI18n.tr('Имя *', 'First Name *')),
        _buildTextField(
          controller: firstNameController,
          hint: DesktopI18n.tr('Например: Илья или Alice', 'e.g. Alex or Alice'),
          errorText: nameError,
        ),
        const SizedBox(height: 16),
        _buildFieldLabel(DesktopI18n.tr('Фамилия (опционально)', 'Last Name (optional)')),
        _buildTextField(
          controller: lastNameController,
          hint: DesktopI18n.tr('Например: Смирнов или Smith', 'e.g. Smith'),
        ),
        const SizedBox(height: 10),
        Row(
          children: [
            const Icon(Icons.info_outline, size: 14, color: ShadcnColors.foregroundSubtle),
            const SizedBox(width: 6),
            Text(
              DesktopI18n.tr(
                'Так вас будет называть агент в диалогах, отчётах и системных файлах личности.',
                'This is how the agent will address you in dialogues, reports, and personality files.',
              ),
              style: const TextStyle(fontSize: 11, color: ShadcnColors.foregroundSubtle),
            ),
          ],
        ),
      ],
    );
  }

  // ── Step 1: Role, Stack, Language ────────────────────────────────────────
  Widget _buildStep1Profile() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          DesktopI18n.tr('Кто вы и с чем работаете?', 'Who are you and what do you build?'),
          style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: ShadcnColors.foreground),
        ),
        const SizedBox(height: 6),
        Text(
          DesktopI18n.tr(
            'Эти сведения помогут агенту подбирать правильные паттерны кода, зависимости и уровень абстракции.',
            'This context helps the agent recommend relevant architectural patterns and idioms.',
          ),
          style: const TextStyle(fontSize: 13, color: ShadcnColors.foregroundMuted),
        ),
        const SizedBox(height: 24),
        _buildFieldLabel(DesktopI18n.tr('Ваша инженерная роль', 'Your Engineering Role')),
        _buildDropdown(
          value: selectedRole,
          items: standardRoles,
          onChanged: (val) => setState(() => selectedRole = val!),
        ),
        if (selectedRole == 'Другое…') ...[
          const SizedBox(height: 8),
          _buildTextField(
            controller: customRoleController,
            hint: DesktopI18n.tr('Введите вашу роль', 'Enter your role'),
          ),
        ],
        const SizedBox(height: 20),
        _buildFieldLabel(DesktopI18n.tr('Основной стек технологий', 'Primary Technology Stack')),
        _buildDropdown(
          value: selectedStack,
          items: standardStacks,
          onChanged: (val) => setState(() => selectedStack = val!),
        ),
        if (selectedStack == 'Другое…') ...[
          const SizedBox(height: 8),
          _buildTextField(
            controller: customStackController,
            hint: DesktopI18n.tr('Например: Elixir / Phoenix / LiveView', 'e.g. Elixir / Phoenix'),
          ),
        ],
        const SizedBox(height: 20),
        _buildFieldLabel(DesktopI18n.tr('Язык общения с агентом', 'Language of Interaction')),
        _buildDropdown(
          value: selectedLanguage,
          items: languages,
          onChanged: (val) => setState(() => selectedLanguage = val!),
        ),
        const SizedBox(height: 24),
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: ShadcnColors.card,
            borderRadius: BorderRadius.circular(10),
            border: Border.all(color: ShadcnColors.border),
          ),
          child: Row(
            children: [
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: ShadcnColors.primary.withOpacity(0.12),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: const Icon(FontAwesomeIcons.diagramProject, size: 16, color: ShadcnColors.primary),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      DesktopI18n.tr('Детерминированный AST-граф памяти (OB2H)', 'Deterministic AST Memory Graph (OB2H)'),
                      style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 13, color: ShadcnColors.foreground),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      DesktopI18n.tr(
                        'Автоматически анализировать структуру репозиториев и сеять факты в память агента.',
                        'Automatically analyze repository structures and populate agent knowledge graph.',
                      ),
                      style: const TextStyle(fontSize: 11, color: ShadcnColors.foregroundMuted),
                    ),
                  ],
                ),
              ),
              Switch(
                value: enableAstMemory,
                activeColor: ShadcnColors.primary,
                onChanged: (val) => setState(() => enableAstMemory = val),
              ),
            ],
          ),
        ),
      ],
    );
  }

  // ── Step 2: Behavior and Tone ────────────────────────────────────────────
  Widget _buildStep2Behavior() {
    final autonomyOptions = [
      {
        'title': 'Ask before changes (спрашивать перед правками)',
        'desc': DesktopI18n.tr(
          'Агент запрашивает подтверждение перед любой записью файлов или выполнением опасных команд. Чтение и анализ — без ограничений.',
          'Agent asks for explicit approval before modifying files or executing risky commands. Read & inspect are autonomous.',
        ),
        'icon': Icons.security,
      },
      {
        'title': 'Plan mode (планирование перед действиями)',
        'desc': DesktopI18n.tr(
          'Агент сначала готовит подробный план задачи, согласует его с вами и только затем переходит к выполнению.',
          'Agent produces a detailed implementation plan first, waits for your review, and then proceeds step by step.',
        ),
        'icon': Icons.checklist,
      },
      {
        'title': 'Full access (максимальная автономность)',
        'desc': DesktopI18n.tr(
          'Агент выполняет задачи под ключ от начала до конца, сам меняет файлы и запускает тесты, отчитываясь о результате.',
          'Agent solves problems end-to-end autonomously, modifying workspace files and reporting status.',
        ),
        'icon': Icons.bolt,
      },
    ];

    final toneOptions = [
      {
        'title': 'Дружелюбный',
        'desc': DesktopI18n.tr(
          'Тёплый, прямой, на «ты», без канцелярита. Задаёт уточняющие вопросы при неясности.',
          'Warm, friendly, direct. Asks clarifying questions rather than guessing.',
        ),
        'icon': Icons.sentiment_satisfied_alt,
      },
      {
        'title': 'Деловой',
        'desc': DesktopI18n.tr(
          'Профессиональный, на «вы», структурированный. Чёткий фокус на инженерном результате и метриках.',
          'Professional, structured, engineering-focused. Clear deliverables and rationale.',
        ),
        'icon': Icons.business_center,
      },
      {
        'title': 'Лаконичный',
        'desc': DesktopI18n.tr(
          'Предельно краткий. Без вежливых шаблонов — только код, списки, технические факты и выводы.',
          'Extremely concise. No conversational filler — direct code snippets, diffs, and conclusions.',
        ),
        'icon': Icons.short_text,
      },
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          DesktopI18n.tr('Как агенту себя вести?', 'How should the agent behave?'),
          style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: ShadcnColors.foreground),
        ),
        const SizedBox(height: 6),
        Text(
          DesktopI18n.tr(
            'Выберите контракт автономности и желаемый тон общения. Это сразу пропишется в SOUL.md.',
            'Choose the autonomy contract and communication tone. These become the core tenets of SOUL.md.',
          ),
          style: const TextStyle(fontSize: 13, color: ShadcnColors.foregroundMuted),
        ),
        const SizedBox(height: 20),
        _buildFieldLabel(DesktopI18n.tr('Стиль автономности', 'Autonomy Contract')),
        ...autonomyOptions.map((opt) {
          final isSelected = selectedAutonomy == opt['title'];
          return _buildSelectableCard(
            title: opt['title'] as String,
            desc: opt['desc'] as String,
            icon: opt['icon'] as IconData,
            isSelected: isSelected,
            onTap: () => setState(() => selectedAutonomy = opt['title'] as String),
          );
        }),
        const SizedBox(height: 20),
        _buildFieldLabel(DesktopI18n.tr('Тон общения', 'Communication Tone')),
        ...toneOptions.map((opt) {
          final isSelected = selectedTone == opt['title'];
          return _buildSelectableCard(
            title: opt['title'] as String,
            desc: opt['desc'] as String,
            icon: opt['icon'] as IconData,
            isSelected: isSelected,
            onTap: () => setState(() => selectedTone = opt['title'] as String),
          );
        }),
      ],
    );
  }

  // ── Step 3: Model and Channel (Optional) ─────────────────────────────────
  Widget _buildStep3ModelChannel() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(
              DesktopI18n.tr('Подключение модели и каналов', 'Model & Channels Setup'),
              style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: ShadcnColors.foreground),
            ),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
              decoration: BoxDecoration(
                color: ShadcnColors.cardElevated,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: ShadcnColors.border),
              ),
              child: Text(
                DesktopI18n.tr('Опционально', 'Optional'),
                style: const TextStyle(fontSize: 11, color: ShadcnColors.primary),
              ),
            ),
          ],
        ),
        const SizedBox(height: 6),
        Text(
          DesktopI18n.tr(
            'Вы можете ввести API-ключ прямо сейчас или нажать «Пропустить» — агент будет создан с дефолтной заглушкой, а ключ можно добавить в Настройках позже.',
            'You can provide your API credentials now or click "Skip" — the agent will initialize with placeholder credentials.',
          ),
          style: const TextStyle(fontSize: 13, color: ShadcnColors.foregroundMuted),
        ),
        const SizedBox(height: 24),
        _buildFieldLabel(DesktopI18n.tr('Провайдер LLM', 'LLM Provider')),
        _buildDropdown(
          value: selectedProviderType,
          items: const ['anthropic', 'openai', 'openrouter', 'ollama'],
          onChanged: (val) {
            setState(() {
              selectedProviderType = val!;
              if (selectedProviderType == 'anthropic') {
                modelNameController.text = 'claude-sonnet-4-5';
              } else if (selectedProviderType == 'openai') {
                modelNameController.text = 'gpt-4o';
              } else if (selectedProviderType == 'openrouter') {
                modelNameController.text = 'anthropic/claude-3.5-sonnet';
              } else if (selectedProviderType == 'ollama') {
                modelNameController.text = 'llama3:latest';
              }
            });
          },
        ),
        const SizedBox(height: 16),
        _buildFieldLabel(DesktopI18n.tr('Модель', 'Model Identifier')),
        _buildTextField(
          controller: modelNameController,
          hint: 'e.g. claude-sonnet-4-5',
        ),
        const SizedBox(height: 16),
        _buildFieldLabel(DesktopI18n.tr('API Ключ', 'API Key')),
        _buildTextField(
          controller: apiKeyController,
          hint: 'sk-...',
          obscureText: true,
        ),
        const SizedBox(height: 24),
        _buildFieldLabel(DesktopI18n.tr('Telegram Bot Token (опционально)', 'Telegram Bot Token (optional)')),
        _buildTextField(
          controller: telegramTokenController,
          hint: '123456789:ABCDefGhIJKlmNoPQRsTUVwxyZ',
          obscureText: true,
        ),
        const SizedBox(height: 16),
        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: ShadcnColors.card,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: ShadcnColors.borderSubtle),
          ),
          child: Row(
            children: [
              const Icon(Icons.shield_outlined, size: 16, color: ShadcnColors.success),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  DesktopI18n.tr(
                    'Ключи шифруются и хранятся исключительно в SecretStore локального шлюза.',
                    'API keys are stored securely inside the local gateway SecretStore.',
                  ),
                  style: const TextStyle(fontSize: 11, color: ShadcnColors.foregroundMuted),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  // ── Step 4: Personality Preview and Apply ─────────────────────────────────
  Widget _buildStep4Preview() {
    final files = ['SOUL.md', 'IDENTITY.md', 'USER.md', 'MEMORY.md'];
    final activeController = personalityControllers[activePreviewFile];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          DesktopI18n.tr('Сформированная личность агента', 'Generated Agent Personality'),
          style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: ShadcnColors.foreground),
        ),
        const SizedBox(height: 6),
        Text(
          DesktopI18n.tr(
            'Эти 4 файла будут записаны в рабочее пространство агента. Вы можете отредактировать их прямо сейчас — ваши правки сохранятся.',
            'These 4 files will be materialized in the workspace and loaded into system prompts. You can edit them freely.',
          ),
          style: const TextStyle(fontSize: 13, color: ShadcnColors.foregroundMuted),
        ),
        const SizedBox(height: 20),
        // File tabs
        Row(
          children: files.map((fileName) {
            final isActive = activePreviewFile == fileName;
            return Padding(
              padding: const EdgeInsets.only(right: 8),
              child: InkWell(
                onTap: () => setState(() => activePreviewFile = fileName),
                borderRadius: BorderRadius.circular(8),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                  decoration: BoxDecoration(
                    color: isActive ? ShadcnColors.primary.withOpacity(0.15) : ShadcnColors.card,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(
                      color: isActive ? ShadcnColors.primary : ShadcnColors.border,
                      width: isActive ? 1.5 : 1,
                    ),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        FontAwesomeIcons.fileLines,
                        size: 12,
                        color: isActive ? ShadcnColors.primary : ShadcnColors.foregroundMuted,
                      ),
                      const SizedBox(width: 8),
                      Text(
                        fileName,
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
                          color: isActive ? ShadcnColors.primary : ShadcnColors.foreground,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            );
          }).toList(),
        ),
        const SizedBox(height: 12),
        // Editor area
        if (activeController != null) ...[
          Container(
            height: 260,
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: ShadcnColors.background,
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: ShadcnColors.border),
            ),
            child: TextField(
              controller: activeController,
              maxLines: null,
              expands: true,
              style: const TextStyle(
                fontSize: 12,
                fontFamily: 'Consolas',
                color: ShadcnColors.foreground,
                height: 1.4,
              ),
              decoration: const InputDecoration(
                border: InputBorder.none,
                isDense: true,
                contentPadding: EdgeInsets.zero,
              ),
              onChanged: (_) => setState(() {}),
            ),
          ),
          const SizedBox(height: 6),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                '${DesktopI18n.tr("Символов:", "Characters:")} ${activeController.text.length} / ${PersonalityComposer.maxFileChars}',
                style: TextStyle(
                  fontSize: 11,
                  color: activeController.text.length > PersonalityComposer.maxFileChars
                      ? ShadcnColors.destructive
                      : ShadcnColors.foregroundSubtle,
                ),
              ),
              Text(
                DesktopI18n.tr('Редактируемо перед записью', 'Editable before write'),
                style: const TextStyle(fontSize: 11, color: ShadcnColors.foregroundSubtle),
              ),
            ],
          ),
        ],
      ],
    );
  }

  // ── Error Banner ─────────────────────────────────────────────────────────
  Widget _buildErrorBanner() {
    return Container(
      width: double.infinity,
      margin: const EdgeInsets.symmetric(horizontal: 24, vertical: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: ShadcnColors.destructiveMuted,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: ShadcnColors.destructive),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.error_outline, size: 16, color: ShadcnColors.destructive),
              const SizedBox(width: 8),
              Text(
                backendErrorSummary ?? '',
                style: const TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  color: ShadcnColors.destructive,
                ),
              ),
            ],
          ),
          if (inlineErrors.isNotEmpty) ...[
            const SizedBox(height: 6),
            ...inlineErrors.entries.map((e) => Text(
                  '• ${e.key}: ${e.value}',
                  style: const TextStyle(fontSize: 11, color: ShadcnColors.foreground),
                )),
          ],
        ],
      ),
    );
  }

  // ── Bottom Actions ───────────────────────────────────────────────────────
  Widget _buildBottomActions() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      decoration: const BoxDecoration(
        color: ShadcnColors.card,
        border: Border(top: BorderSide(color: ShadcnColors.border)),
      ),
      child: Row(
        children: [
          if (currentStep > 0)
            OutlinedButton(
              onPressed: isSubmitting ? null : _onBack,
              style: OutlinedButton.styleFrom(
                foregroundColor: ShadcnColors.foreground,
                side: const BorderSide(color: ShadcnColors.border),
                padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: Text(DesktopI18n.backBtn),
            ),
          const Spacer(),
          if (currentStep == 3) ...[
            TextButton(
              onPressed: isSubmitting ? null : () => _submitProfile(skipModel: true),
              style: TextButton.styleFrom(
                foregroundColor: ShadcnColors.foregroundMuted,
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
              ),
              child: Text(DesktopI18n.tr('Пропустить шаг', 'Skip this step')),
            ),
            const SizedBox(width: 12),
          ],
          if (currentStep < totalSteps - 1)
            ElevatedButton(
              onPressed: _onNext,
              style: ElevatedButton.styleFrom(
                backgroundColor: ShadcnColors.primary,
                foregroundColor: Colors.black,
                padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(DesktopI18n.nextBtn, style: const TextStyle(fontWeight: FontWeight.bold)),
                  const SizedBox(width: 8),
                  const Icon(Icons.arrow_forward, size: 16),
                ],
              ),
            )
          else
            ElevatedButton(
              onPressed: isSubmitting ? null : () => _submitProfile(skipModel: false),
              style: ElevatedButton.styleFrom(
                backgroundColor: ShadcnColors.primary,
                foregroundColor: Colors.black,
                padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 14),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: isSubmitting
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2, color: Colors.black),
                    )
                  : Text(
                      DesktopI18n.tr('Применить и запустить агента', 'Apply & Start Agent'),
                      style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
            ),
        ],
      ),
    );
  }

  // ── Helper Widgets ───────────────────────────────────────────────────────
  Widget _buildFieldLabel(String label) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Text(
        label,
        style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: ShadcnColors.foreground),
      ),
    );
  }

  Widget _buildTextField({
    required TextEditingController controller,
    required String hint,
    String? errorText,
    bool obscureText = false,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          height: 44,
          padding: const EdgeInsets.symmetric(horizontal: 14),
          decoration: BoxDecoration(
            color: ShadcnColors.cardElevated,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(
              color: errorText != null ? ShadcnColors.destructive : ShadcnColors.border,
            ),
          ),
          child: TextField(
            controller: controller,
            obscureText: obscureText,
            style: const TextStyle(fontSize: 13, color: ShadcnColors.foreground),
            decoration: InputDecoration(
              hintText: hint,
              hintStyle: const TextStyle(fontSize: 13, color: ShadcnColors.foregroundSubtle),
              border: InputBorder.none,
              isDense: true,
              contentPadding: const EdgeInsets.symmetric(vertical: 12),
            ),
            onChanged: (_) {
              if (errorText != null) {
                setState(() => nameError = null);
              }
            },
          ),
        ),
        if (errorText != null)
          Padding(
            padding: const EdgeInsets.only(top: 6, left: 2),
            child: Text(
              errorText,
              style: const TextStyle(fontSize: 11, color: ShadcnColors.destructive),
            ),
          ),
      ],
    );
  }

  Widget _buildDropdown({
    required String value,
    required List<String> items,
    required ValueChanged<String?> onChanged,
  }) {
    return Container(
      height: 44,
      padding: const EdgeInsets.symmetric(horizontal: 14),
      decoration: BoxDecoration(
        color: ShadcnColors.cardElevated,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: ShadcnColors.border),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<String>(
          value: items.contains(value) ? value : items.first,
          isExpanded: true,
          dropdownColor: ShadcnColors.cardElevated,
          icon: const Icon(Icons.keyboard_arrow_down, size: 18, color: ShadcnColors.foregroundMuted),
          style: const TextStyle(fontSize: 13, color: ShadcnColors.foreground),
          items: items.map((item) {
            return DropdownMenuItem<String>(
              value: item,
              child: Text(item),
            );
          }).toList(),
          onChanged: onChanged,
        ),
      ),
    );
  }

  Widget _buildSelectableCard({
    required String title,
    required String desc,
    required IconData icon,
    required bool isSelected,
    required VoidCallback onTap,
  }) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(10),
        child: Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: isSelected ? ShadcnColors.primary.withOpacity(0.08) : ShadcnColors.card,
            borderRadius: BorderRadius.circular(10),
            border: Border.all(
              color: isSelected ? ShadcnColors.primary : ShadcnColors.border,
              width: isSelected ? 1.5 : 1,
            ),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Container(
                width: 32,
                height: 32,
                decoration: BoxDecoration(
                  color: isSelected ? ShadcnColors.primary.withOpacity(0.18) : ShadcnColors.cardElevated,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Icon(
                  icon,
                  size: 16,
                  color: isSelected ? ShadcnColors.primary : ShadcnColors.foregroundMuted,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      title,
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.bold,
                        color: isSelected ? ShadcnColors.primary : ShadcnColors.foreground,
                      ),
                    ),
                    const SizedBox(height: 3),
                    Text(
                      desc,
                      style: const TextStyle(fontSize: 11, color: ShadcnColors.foregroundMuted),
                    ),
                  ],
                ),
              ),
              Radio<bool>(
                value: true,
                groupValue: isSelected,
                activeColor: ShadcnColors.primary,
                onChanged: (_) => onTap(),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
