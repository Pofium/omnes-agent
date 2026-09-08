import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';
import '../../theme/desktop_theme.dart';
import '../../utils/desktop_i18n.dart';

class UserProfileData {
  String firstName;
  String lastName;
  String role;
  String primaryStack;
  String autonomyStyle;
  String language;
  bool enableAstMemory;

  UserProfileData({
    this.firstName = 'Илья',
    this.lastName = 'Пресняков',
    this.role = 'Tech Lead / AI Engineer',
    this.primaryStack = 'Rust / Dart / Python',
    this.autonomyStyle = 'Full access (максимальная автономность)',
    this.language = 'Русский',
    this.enableAstMemory = true,
  });

  String get fullName => '$firstName $lastName'.trim();
  String get initials => '${firstName.isNotEmpty ? firstName[0] : ''}${lastName.isNotEmpty ? lastName[0] : ''}'.toUpperCase();
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
      barrierColor: Colors.black.withOpacity(0.75),
      barrierDismissible: true,
      builder: (ctx) => Dialog(
        backgroundColor: Colors.transparent,
        insetPadding: const EdgeInsets.symmetric(horizontal: 40, vertical: 30),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(16),
          child: Container(
            constraints: const BoxConstraints(maxWidth: 680, maxHeight: 660),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurface,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: DesktopTheme.borderSubtle, width: 1.2),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.4),
                  blurRadius: 28,
                  offset: const Offset(0, 10),
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
  int currentStep = 0; // 0: Personal Info, 1: AI Agent Questionnaire

  late TextEditingController firstNameController;
  late TextEditingController lastNameController;
  late String selectedRole;
  late String selectedStack;
  late String selectedAutonomy;
  late String selectedLanguage;
  late bool enableAstMemory;

  final List<String> availableRoles = [
    'Tech Lead / AI Engineer',
    'Backend Engineer (Rust / Go)',
    'Fullstack Developer (Flutter / TS)',
    'AI Researcher / ML Engineer',
    'DevOps / SRE Specialist',
  ];

  final List<String> availableStacks = [
    'Rust / Dart / Python',
    'Flutter / Dart / Node.js',
    'Rust / Axum / Actix',
    'Python / PyTorch / FastEmbed',
    'Go / Kubernetes / Docker',
  ];

  final List<String> autonomyStyles = [
    'Full access (максимальная автономность)',
    'Ask before changes (спрашивать перед правками)',
    'Plan mode (планирование перед действиями)',
  ];

  @override
  void initState() {
    super.initState();
    firstNameController = TextEditingController(text: widget.initialProfile.firstName);
    lastNameController = TextEditingController(text: widget.initialProfile.lastName);
    selectedRole = widget.initialProfile.role;
    selectedStack = widget.initialProfile.primaryStack;
    selectedAutonomy = widget.initialProfile.autonomyStyle;
    selectedLanguage = widget.initialProfile.language;
    enableAstMemory = widget.initialProfile.enableAstMemory;
  }

  @override
  void dispose() {
    firstNameController.dispose();
    lastNameController.dispose();
    super.dispose();
  }

  void _submitProfile() {
    final updated = UserProfileData(
      firstName: firstNameController.text.trim().isEmpty ? 'Илья' : firstNameController.text.trim(),
      lastName: lastNameController.text.trim().isEmpty ? 'Пресняков' : lastNameController.text.trim(),
      role: selectedRole,
      primaryStack: selectedStack,
      autonomyStyle: selectedAutonomy,
      language: selectedLanguage,
      enableAstMemory: enableAstMemory,
    );
    widget.onSave(updated);

    // Sync profile to gateway memory & quickstart in background
    try {
      final http = GatewayHttpClient();
      http.memoryStore(
        key: 'user_profile',
        content: jsonEncode({
          'name': updated.fullName,
          'role': updated.role,
          'stack': updated.primaryStack,
          'autonomy': updated.autonomyStyle,
          'language': updated.language,
          'ast_memory': updated.enableAstMemory,
        }),
      );
      http.applyQuickstart({
        'user_name': updated.fullName,
        'role': updated.role,
        'language': updated.language,
      });
    } catch (_) {}

    Navigator.of(context).pop(updated);
  }

  @override
  Widget build(BuildContext context) {
    return Obx(() {
      final _ = DesktopI18n.currentLanguage.value;

      return Column(
        children: [
          // Top Header
          Container(
            height: 60,
            padding: const EdgeInsets.symmetric(horizontal: 24),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurfaceElevated,
              border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
            ),
            child: Row(
              children: [
                Container(
                  width: 28,
                  height: 28,
                  decoration: BoxDecoration(
                    color: const Color(0xFF00D2FF).withOpacity(0.18),
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.35)),
                  ),
                  child: const Icon(FontAwesomeIcons.userAstronaut, size: 14, color: Color(0xFF00D2FF)),
                ),
                const SizedBox(width: 12),
                Text(
                  currentStep == 0
                      ? DesktopI18n.tr('Регистрация и профиль пользователя', 'User Registration & Profile')
                      : DesktopI18n.tr('Персонализация агента (Анкета)', 'Agent Personalization (Questionnaire)'),
                  style: TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                    color: DesktopTheme.textPrimary,
                  ),
                ),
                const Spacer(),
                IconButton(
                  icon: Icon(Icons.close, size: 18, color: DesktopTheme.textMuted),
                  onPressed: () => Navigator.of(context).pop(),
                ),
              ],
            ),
          ),

          // Body with Step Content
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(24),
              child: currentStep == 0 ? _buildStepPersonalInfo() : _buildStepQuestionnaire(),
            ),
          ),

          // Bottom Actions
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurfaceElevated,
              border: Border(top: BorderSide(color: DesktopTheme.borderSubtle)),
            ),
            child: Row(
              children: [
                if (currentStep > 0)
                  OutlinedButton(
                    onPressed: () => setState(() => currentStep = 0),
                    style: OutlinedButton.styleFrom(
                      foregroundColor: DesktopTheme.textSecondary,
                      side: BorderSide(color: DesktopTheme.borderSubtle),
                      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                    ),
                    child: Text(DesktopI18n.tr('Назад', 'Back')),
                  ),
                const Spacer(),
                if (currentStep == 0)
                  ElevatedButton(
                    onPressed: () => setState(() => currentStep = 1),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: const Color(0xFF00D2FF),
                      foregroundColor: const Color(0xFF0F172A),
                      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(DesktopI18n.tr('Далее к анкете агента', 'Next to Agent Questionnaire'), style: const TextStyle(fontWeight: FontWeight.bold)),
                        const SizedBox(width: 8),
                        const Icon(Icons.arrow_forward, size: 16),
                      ],
                    ),
                  )
                else
                  ElevatedButton(
                    onPressed: _submitProfile,
                    style: ElevatedButton.styleFrom(
                      backgroundColor: const Color(0xFF00D2FF),
                      foregroundColor: const Color(0xFF0F172A),
                      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                    ),
                    child: Text(DesktopI18n.finishBtn, style: const TextStyle(fontWeight: FontWeight.bold)),
                  ),
              ],
            ),
          ),
        ],
      );
    });
  }

  Widget _buildStepPersonalInfo() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Avatar preview
        Center(
          child: Column(
            children: [
              Container(
                width: 80,
                height: 80,
                decoration: BoxDecoration(
                  gradient: const LinearGradient(
                    colors: [Color(0xFF00D2FF), Color(0xFF0072FF)],
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                  ),
                  borderRadius: BorderRadius.circular(40),
                  border: Border.all(color: const Color(0xFF00D2FF), width: 2),
                ),
                child: Center(
                  child: Text(
                    '${firstNameController.text.isNotEmpty ? firstNameController.text[0] : 'И'}${lastNameController.text.isNotEmpty ? lastNameController.text[0] : 'П'}',
                    style: const TextStyle(fontSize: 28, fontWeight: FontWeight.bold, color: Colors.white),
                  ),
                ),
              ),
              const SizedBox(height: 12),
              Text(
                DesktopI18n.tr('Ваш профиль в рабочей среде', 'Your workspace profile'),
                style: TextStyle(fontSize: 12, color: DesktopTheme.textMuted),
              ),
            ],
          ),
        ),
        const SizedBox(height: 24),

        // First Name
        Text(DesktopI18n.firstName, style: TextStyle(fontSize: 13, color: DesktopTheme.textSecondary, fontWeight: FontWeight.w600)),
        const SizedBox(height: 6),
        _buildTextField(firstNameController, DesktopI18n.tr('Введите ваше имя (например: Илья)', 'Enter your first name (e.g. Ilya)')),
        const SizedBox(height: 16),

        // Last Name
        Text(DesktopI18n.lastName, style: TextStyle(fontSize: 13, color: DesktopTheme.textSecondary, fontWeight: FontWeight.w600)),
        const SizedBox(height: 6),
        _buildTextField(lastNameController, DesktopI18n.tr('Введите вашу фамилию (например: Пресняков)', 'Enter your last name (e.g. Presnyakov)')),
      ],
    );
  }

  Widget _buildStepQuestionnaire() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          DesktopI18n.tr('Анкета адаптации агента', 'Agent Adaptation Questionnaire'),
          style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
        ),
        const SizedBox(height: 6),
        Text(
          DesktopI18n.tr(
            'Агент сохранит ваши предпочтения в долговременную память ob2h и будет учитывать их при генерации решений и команд.',
            'The agent will save your preferences to ob2h AST memory and consider them when generating solutions.',
          ),
          style: TextStyle(fontSize: 12, color: DesktopTheme.textSecondary, height: 1.4),
        ),
        const SizedBox(height: 20),

        // 1. Role
        _buildDropdownSetting(
          label: DesktopI18n.tr('1. Ваша роль / специализация:', '1. Your Role / Specialization:'),
          value: selectedRole,
          items: availableRoles,
          onChanged: (val) => setState(() => selectedRole = val!),
        ),
        const SizedBox(height: 16),

        // 2. Primary Stack
        _buildDropdownSetting(
          label: DesktopI18n.tr('2. Основной стек технологий:', '2. Primary Tech Stack:'),
          value: selectedStack,
          items: availableStacks,
          onChanged: (val) => setState(() => selectedStack = val!),
        ),
        const SizedBox(height: 16),

        // 3. Autonomy Style
        _buildDropdownSetting(
          label: DesktopI18n.tr('3. Предпочитаемый режим автономности:', '3. Preferred Autonomy Mode:'),
          value: selectedAutonomy,
          items: autonomyStyles,
          onChanged: (val) => setState(() => selectedAutonomy = val!),
        ),
        const SizedBox(height: 16),

        // 4. Communication Language
        _buildDropdownSetting(
          label: DesktopI18n.tr('4. Язык общения агента:', '4. Agent Communication Language:'),
          value: selectedLanguage,
          items: const ['Русский', 'English'],
          onChanged: (val) => setState(() => selectedLanguage = val!),
        ),
        const SizedBox(height: 16),

        // 5. Memory AST Toggle
        Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurfaceElevated,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: DesktopTheme.borderSubtle),
          ),
          child: Row(
            children: [
              const Icon(FontAwesomeIcons.brain, size: 16, color: Color(0xFF00D2FF)),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(DesktopI18n.memorySettingTitle, style: TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary)),
                    const SizedBox(height: 4),
                    Text(
                      DesktopI18n.tr('Автоматически сохранять факты о структуре репозитория и окружении', 'Automatically persist repo structure and workspace facts'),
                      style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                    ),
                  ],
                ),
              ),
              Switch(
                value: enableAstMemory,
                activeColor: const Color(0xFF00D2FF),
                onChanged: (val) => setState(() => enableAstMemory = val),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildTextField(TextEditingController controller, String hint) {
    return Container(
      height: 40,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurfaceElevated,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: DesktopTheme.borderSubtle),
      ),
      child: TextField(
        controller: controller,
        style: TextStyle(fontSize: 13, color: DesktopTheme.textPrimary),
        decoration: InputDecoration(
          hintText: hint,
          hintStyle: TextStyle(fontSize: 13, color: DesktopTheme.textMuted),
          border: InputBorder.none,
          enabledBorder: InputBorder.none,
          focusedBorder: InputBorder.none,
          errorBorder: InputBorder.none,
          focusedErrorBorder: InputBorder.none,
          isDense: true,
          contentPadding: const EdgeInsets.symmetric(vertical: 10),
        ),
        onChanged: (_) => setState(() {}),
      ),
    );
  }

  Widget _buildDropdownSetting({
    required String label,
    required String value,
    required List<String> items,
    required ValueChanged<String?> onChanged,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: TextStyle(fontSize: 13, color: DesktopTheme.textSecondary, fontWeight: FontWeight.w600)),
        const SizedBox(height: 6),
        Container(
          height: 40,
          padding: const EdgeInsets.symmetric(horizontal: 12),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurfaceElevated,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: DesktopTheme.borderSubtle),
          ),
          child: DropdownButtonHideUnderline(
            child: DropdownButton<String>(
              value: value,
              isExpanded: true,
              dropdownColor: DesktopTheme.bgSurfaceElevated,
              icon: Icon(Icons.arrow_drop_down, color: DesktopTheme.textMuted),
              items: items.map((item) => DropdownMenuItem(
                value: item,
                child: Text(item, style: TextStyle(fontSize: 13, color: DesktopTheme.textPrimary)),
              )).toList(),
              onChanged: onChanged,
            ),
          ),
        ),
      ],
    );
  }
}
