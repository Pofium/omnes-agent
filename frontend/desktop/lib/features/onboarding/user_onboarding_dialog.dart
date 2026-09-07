import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import '../../utils/desktop_i18n.dart';

class UserProfileData {
  String firstName;
  String lastName;
  String tier;
  String role;
  String primaryStack;
  String autonomyStyle;
  String language;
  bool enableAstMemory;

  UserProfileData({
    this.firstName = 'Илья',
    this.lastName = 'Пресняков',
    this.tier = 'Lite',
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
            constraints: const BoxConstraints(maxWidth: 680, maxHeight: 720),
            decoration: BoxDecoration(
              color: const Color(0xFF14161B),
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: const Color(0xFF262B34), width: 1.2),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.7),
                  blurRadius: 40,
                  spreadRadius: 8,
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
  late String selectedTier;
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
    selectedTier = widget.initialProfile.tier;
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
      tier: selectedTier,
      role: selectedRole,
      primaryStack: selectedStack,
      autonomyStyle: selectedAutonomy,
      language: selectedLanguage,
      enableAstMemory: enableAstMemory,
    );
    widget.onSave(updated);
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
            decoration: const BoxDecoration(
              color: Color(0xFF181B22),
              border: Border(bottom: BorderSide(color: Color(0xFF242933))),
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
                  style: const TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                    color: Colors.white,
                  ),
                ),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.close, size: 18, color: Color(0xFF94A3B8)),
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
            decoration: const BoxDecoration(
              color: Color(0xFF181B22),
              border: Border(top: BorderSide(color: Color(0xFF242933))),
            ),
            child: Row(
              children: [
                if (currentStep > 0)
                  OutlinedButton(
                    onPressed: () => setState(() => currentStep = 0),
                    style: OutlinedButton.styleFrom(
                      foregroundColor: const Color(0xFF94A3B8),
                      side: const BorderSide(color: Color(0xFF333A47)),
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
              const SizedBox(height: 10),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(
                  color: const Color(0xFF222834),
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: const Color(0xFF3B4354)),
                ),
                child: Text(
                  selectedTier,
                  style: const TextStyle(fontSize: 11, color: Color(0xFF94A3B8), fontWeight: FontWeight.bold),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 24),

        // First Name
        Text(DesktopI18n.firstName, style: const TextStyle(fontSize: 13, color: Color(0xFF94A3B8), fontWeight: FontWeight.w600)),
        const SizedBox(height: 6),
        _buildTextField(firstNameController, DesktopI18n.tr('Введите ваше имя (например: Илья)', 'Enter your first name (e.g. Ilya)')),
        const SizedBox(height: 16),

        // Last Name
        Text(DesktopI18n.lastName, style: const TextStyle(fontSize: 13, color: Color(0xFF94A3B8), fontWeight: FontWeight.w600)),
        const SizedBox(height: 6),
        _buildTextField(lastNameController, DesktopI18n.tr('Введите вашу фамилию (например: Пресняков)', 'Enter your last name (e.g. Presnyakov)')),
        const SizedBox(height: 16),

        // Tier / Status
        Text(DesktopI18n.tr('Тарифный статус', 'Subscription Tier'), style: const TextStyle(fontSize: 13, color: Color(0xFF94A3B8), fontWeight: FontWeight.w600)),
        const SizedBox(height: 6),
        Row(
          children: ['Lite', 'Pro', 'Enterprise'].map((tier) {
            final isSel = selectedTier == tier;
            return Expanded(
              child: GestureDetector(
                onTap: () => setState(() => selectedTier = tier),
                child: Container(
                  margin: const EdgeInsets.only(right: 8),
                  padding: const EdgeInsets.symmetric(vertical: 10),
                  decoration: BoxDecoration(
                    color: isSel ? const Color(0xFF00D2FF).withOpacity(0.15) : const Color(0xFF1E222A),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: isSel ? const Color(0xFF00D2FF) : const Color(0xFF2A303D)),
                  ),
                  child: Center(
                    child: Text(
                      tier,
                      style: TextStyle(
                        fontSize: 13,
                        fontWeight: isSel ? FontWeight.bold : FontWeight.normal,
                        color: isSel ? Colors.white : const Color(0xFF94A3B8),
                      ),
                    ),
                  ),
                ),
              ),
            );
          }).toList(),
        ),
      ],
    );
  }

  Widget _buildStepQuestionnaire() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          DesktopI18n.tr('Анкета адаптации агента', 'Agent Adaptation Questionnaire'),
          style: const TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: Colors.white),
        ),
        const SizedBox(height: 6),
        Text(
          DesktopI18n.tr(
            'Агент сохранит ваши предпочтения в долговременную память ob2h и будет учитывать их при генерации решений и команд.',
            'The agent will save your preferences to ob2h AST memory and consider them when generating solutions.',
          ),
          style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8), height: 1.4),
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
            color: const Color(0xFF1E222A),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: const Color(0xFF2B313E)),
          ),
          child: Row(
            children: [
              const Icon(FontAwesomeIcons.brain, size: 16, color: Color(0xFF00D2FF)),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(DesktopI18n.memorySettingTitle, style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: Colors.white)),
                    const SizedBox(height: 4),
                    Text(
                      DesktopI18n.tr('Автоматически сохранять факты о структуре репозитория и окружении', 'Automatically persist repo structure and workspace facts'),
                      style: const TextStyle(fontSize: 11, color: Color(0xFF94A3B8)),
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
      padding: const EdgeInsets.symmetric(horizontal: 14),
      decoration: BoxDecoration(
        color: const Color(0xFF1E222A),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: const Color(0xFF2E3544)),
      ),
      child: TextField(
        controller: controller,
        style: const TextStyle(fontSize: 13, color: Colors.white),
        decoration: InputDecoration(
          hintText: hint,
          hintStyle: const TextStyle(fontSize: 13, color: Color(0xFF64748B)),
          border: InputBorder.none,
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
        Text(label, style: const TextStyle(fontSize: 13, color: Color(0xFFCBD5E1), fontWeight: FontWeight.w600)),
        const SizedBox(height: 6),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          decoration: BoxDecoration(
            color: const Color(0xFF1E222A),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: const Color(0xFF2E3544)),
          ),
          child: DropdownButton<String>(
            value: value,
            isExpanded: true,
            underline: const SizedBox.shrink(),
            dropdownColor: const Color(0xFF1E222A),
            icon: const Icon(Icons.arrow_drop_down, color: Color(0xFF94A3B8)),
            items: items.map((item) => DropdownMenuItem(
              value: item,
              child: Text(item, style: const TextStyle(fontSize: 13, color: Colors.white)),
            )).toList(),
            onChanged: onChanged,
          ),
        ),
      ],
    );
  }
}
