// Desktop Settings Dialog for OmnesAgent ADE matching authentic desktop styling.
// Includes LLM Providers management with custom provider creation, MCP Servers management,
// Skills/Plugins, General configuration, Russian localization, and Profile setup.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import '../../theme/desktop_theme.dart';
import '../onboarding/user_onboarding_dialog.dart';

class DesktopSettingsDialog extends StatefulWidget {
  final VoidCallback onBackToWorkspace;
  final String initialSection;
  final UserProfileData userProfile;
  final Function(UserProfileData) onUpdateProfile;

  const DesktopSettingsDialog({
    super.key,
    required this.onBackToWorkspace,
    this.initialSection = 'Общие',
    required this.userProfile,
    required this.onUpdateProfile,
  });

  @override
  State<DesktopSettingsDialog> createState() => _DesktopSettingsDialogState();
}

class _DesktopSettingsDialogState extends State<DesktopSettingsDialog> {
  late String selectedSection;

  // General settings state
  String currentLanguage = 'Русский (Russian)';
  bool memoryEnabled = true;
  bool inheritTerminal = true;
  bool enhancedGrep = true;
  final terminalFontController = TextEditingController(text: 'JetBrains Mono, SFMono-Regular, monospace');
  final gatewayUrlController = TextEditingController(text: 'http://127.0.0.1:42617');

  // LLM Providers state
  String activeModel = 'GLM-5.3';
  final List<Map<String, dynamic>> providers = [
    {
      'name': 'GLM (Z.ai API)',
      'type': 'glm',
      'url': 'https://open.bigmodel.cn/api/paas/v4',
      'models': ['GLM-5.3', 'GLM-5.3-Flash', 'GLM-5-Turbo'],
      'isConfigured': true,
      'isCustom': false,
    },
    {
      'name': 'Anthropic',
      'type': 'anthropic',
      'url': 'https://api.anthropic.com/v1',
      'models': ['claude-3-5-sonnet', 'claude-3-haiku'],
      'isConfigured': true,
      'isCustom': false,
    },
    {
      'name': 'DeepSeek',
      'type': 'deepseek',
      'url': 'https://api.deepseek.com/v1',
      'models': ['deepseek-chat', 'deepseek-reasoner'],
      'isConfigured': true,
      'isCustom': false,
    },
    {
      'name': 'OpenAI',
      'type': 'openai',
      'url': 'https://api.openai.com/v1',
      'models': ['gpt-4o', 'gpt-4o-mini', 'o1-preview'],
      'isConfigured': false,
      'isCustom': false,
    },
    {
      'name': 'Ollama (Локальный)',
      'type': 'ollama',
      'url': 'http://localhost:11434',
      'models': ['qwen2.5-coder:32b', 'llama3.2', 'deepseek-r1:14b'],
      'isConfigured': true,
      'isCustom': false,
    },
  ];

  // Custom Provider Form
  bool isAddingProvider = false;
  final customNameController = TextEditingController();
  final customUrlController = TextEditingController();
  final customKeyController = TextEditingController();
  final customModelController = TextEditingController();
  String customFormat = 'OpenAI-compatible';

  // MCP Servers State
  final List<Map<String, dynamic>> mcpServers = [
    {
      'name': 'filesystem',
      'description': 'Прямой безопасный доступ к файловой системе рабочей директории',
      'command': 'builtin:node_filesystem',
      'enabled': true,
    },
    {
      'name': 'git',
      'description': 'Git операции: статус, diff, коммиты, переключение веток',
      'command': 'builtin:git_tools',
      'enabled': true,
    },
    {
      'name': 'fetch',
      'description': 'HTTP REST клиент и веб-запросы к внешним API',
      'command': 'builtin:fetch',
      'enabled': true,
    },
    {
      'name': 'chrome-devtools',
      'description': 'Инспектор браузера, захват скриншотов и клики по DOM-элементам',
      'command': 'npx -y @modelcontextprotocol/server-puppeteer',
      'enabled': true,
    },
    {
      'name': 'codegraph',
      'description': 'Детерминированный AST-анализ графа символов и вызовов без траты токенов',
      'command': 'codegraph explore',
      'enabled': true,
    },
    {
      'name': 'ob2h',
      'description': 'Долговременная память, факты о проекте и оценка Blast Radius',
      'command': 'builtin:ob2h_runtime',
      'enabled': true,
    },
  ];

  bool isAddingMcp = false;
  final newMcpNameController = TextEditingController();
  final newMcpCommandController = TextEditingController();

  @override
  void initState() {
    super.initState();
    selectedSection = widget.initialSection;
    if (selectedSection == 'Model Settings') selectedSection = 'Провайдеры';
    if (selectedSection == 'General') selectedSection = 'Общие';
    if (selectedSection == 'Skills') selectedSection = 'Навыки';
  }

  @override
  void dispose() {
    terminalFontController.dispose();
    gatewayUrlController.dispose();
    customNameController.dispose();
    customUrlController.dispose();
    customKeyController.dispose();
    customModelController.dispose();
    newMcpNameController.dispose();
    newMcpCommandController.dispose();
    super.dispose();
  }

  void _saveCustomProvider() {
    if (customNameController.text.trim().isEmpty || customUrlController.text.trim().isEmpty) return;

    setState(() {
      providers.add({
        'name': customNameController.text.trim(),
        'type': 'custom',
        'url': customUrlController.text.trim(),
        'models': [customModelController.text.trim().isEmpty ? 'default-model' : customModelController.text.trim()],
        'isConfigured': true,
        'isCustom': true,
      });
      isAddingProvider = false;
      customNameController.clear();
      customUrlController.clear();
      customKeyController.clear();
      customModelController.clear();
    });
  }

  void _saveNewMcpServer() {
    if (newMcpNameController.text.trim().isEmpty || newMcpCommandController.text.trim().isEmpty) return;

    setState(() {
      mcpServers.add({
        'name': newMcpNameController.text.trim(),
        'description': 'Кастомный MCP сервер',
        'command': newMcpCommandController.text.trim(),
        'enabled': true,
      });
      isAddingMcp = false;
      newMcpNameController.clear();
      newMcpCommandController.clear();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: const Color(0xFF131518),
      child: Row(
        children: [
          // 1. Left Navigation Menu (250px)
          Container(
            width: 250,
            decoration: const BoxDecoration(
              color: Color(0xFF16181D),
              border: Border(right: BorderSide(color: Color(0xFF23272F))),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Back to Workspace
                Padding(
                  padding: const EdgeInsets.only(left: 16, top: 16, bottom: 16, right: 16),
                  child: InkWell(
                    onTap: widget.onBackToWorkspace,
                    borderRadius: BorderRadius.circular(6),
                    child: Padding(
                      padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 8),
                      child: Row(
                        children: const [
                          Icon(Icons.arrow_back, size: 16, color: Color(0xFF94A3B8)),
                          SizedBox(width: 8),
                          Text(
                            'Назад в рабочую область',
                            style: TextStyle(
                              fontSize: 13,
                              fontWeight: FontWeight.w500,
                              color: Color(0xFFCBD5E1),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),

                const Divider(height: 1, color: Color(0xFF23272F)),

                Expanded(
                  child: ListView(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
                    children: [
                      _buildSectionHeader('Основные настройки'),
                      _buildNavItem('Общие', Icons.tune),
                      _buildNavItem('Провайдеры', FontAwesomeIcons.brain),
                      _buildNavItem('Оформление', Icons.brightness_6_outlined),
                      _buildNavItem('Профиль', FontAwesomeIcons.userAstronaut),

                      const SizedBox(height: 16),
                      _buildSectionHeader('Возможности агента'),
                      _buildNavItem('MCP Серверы', Icons.extension_outlined),
                      _buildNavItem('Навыки', Icons.auto_awesome),
                      _buildNavItem('Команды', FontAwesomeIcons.terminal),

                      const SizedBox(height: 16),
                      _buildSectionHeader('Данные и анализ'),
                      _buildNavItem('ob2h AST Память', Icons.shield_outlined),
                      _buildNavItem('Статистика использования', Icons.bar_chart),
                    ],
                  ),
                ),

                // Bottom Profile Card
                Container(
                  padding: const EdgeInsets.all(14),
                  decoration: const BoxDecoration(
                    border: Border(top: BorderSide(color: Color(0xFF23272F))),
                  ),
                  child: Row(
                    children: [
                      Container(
                        width: 30,
                        height: 30,
                        decoration: BoxDecoration(
                          gradient: const LinearGradient(
                            colors: [Color(0xFF00D2FF), Color(0xFF0072FF)],
                          ),
                          borderRadius: BorderRadius.circular(15),
                        ),
                        child: Center(
                          child: Text(
                            widget.userProfile.initials,
                            style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 12),
                          ),
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              widget.userProfile.fullName,
                              style: const TextStyle(fontSize: 13, color: Colors.white, fontWeight: FontWeight.w600),
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                            ),
                            Text(
                              widget.userProfile.tier,
                              style: const TextStyle(fontSize: 10, color: Color(0xFF00D2FF), fontWeight: FontWeight.bold),
                            ),
                          ],
                        ),
                      ),
                      IconButton(
                        tooltip: 'Изменить профиль',
                        icon: const Icon(Icons.edit_outlined, size: 14, color: Color(0xFF94A3B8)),
                        onPressed: () {
                          UserOnboardingDialog.show(
                            context,
                            initialProfile: widget.userProfile,
                            onSave: widget.onUpdateProfile,
                          );
                        },
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),

          // 2. Right Content Area
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Top bar with Section Title & Close button
                Container(
                  height: 56,
                  padding: const EdgeInsets.symmetric(horizontal: 32),
                  alignment: Alignment.centerLeft,
                  decoration: const BoxDecoration(
                    border: Border(bottom: BorderSide(color: Color(0xFF23272F))),
                  ),
                  child: Row(
                    children: [
                      Text(
                        selectedSection,
                        style: const TextStyle(
                          fontSize: 20,
                          fontWeight: FontWeight.bold,
                          color: Colors.white,
                        ),
                      ),
                      const Spacer(),
                      IconButton(
                        icon: const Icon(Icons.close, size: 20, color: Color(0xFF94A3B8)),
                        tooltip: 'Закрыть настройки',
                        onPressed: widget.onBackToWorkspace,
                      ),
                    ],
                  ),
                ),

                // Section Content
                Expanded(
                  child: SingleChildScrollView(
                    padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 24),
                    child: _buildCurrentSectionContent(),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Padding(
      padding: const EdgeInsets.only(left: 8, bottom: 8, top: 4),
      child: Text(
        title,
        style: const TextStyle(
          fontSize: 11,
          fontWeight: FontWeight.w600,
          color: Color(0xFF64748B),
          letterSpacing: 0.5,
        ),
      ),
    );
  }

  Widget _buildNavItem(String title, IconData icon) {
    final isSelected = selectedSection == title;
    return InkWell(
      onTap: () => setState(() => selectedSection = title),
      borderRadius: BorderRadius.circular(6),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
        margin: const EdgeInsets.only(bottom: 2),
        decoration: BoxDecoration(
          color: isSelected ? const Color(0xFF262B33) : Colors.transparent,
          borderRadius: BorderRadius.circular(6),
        ),
        child: Row(
          children: [
            Icon(
              icon,
              size: 14,
              color: isSelected ? const Color(0xFF00D2FF) : const Color(0xFF94A3B8),
            ),
            const SizedBox(width: 10),
            Text(
              title,
              style: TextStyle(
                fontSize: 13,
                fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
                color: isSelected ? Colors.white : const Color(0xFF94A3B8),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCurrentSectionContent() {
    switch (selectedSection) {
      case 'Общие':
        return _buildGeneralSection();
      case 'Провайдеры':
        return _buildProvidersSection();
      case 'MCP Серверы':
        return _buildMcpServersSection();
      case 'Навыки':
        return _buildSkillsSection();
      case 'Оформление':
        return _buildAppearanceSection();
      case 'Профиль':
        return _buildProfileSection();
      case 'ob2h AST Память':
        return _buildMemoryAstSection();
      case 'Статистика использования':
        return _buildUsageStatsSection();
      default:
        return _buildGeneralSection();
    }
  }

  // ========================================================
  // SECTION: ОБЩИЕ (General)
  // ========================================================
  Widget _buildGeneralSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Interface Language
        _buildSettingCard(
          title: 'Язык интерфейса (Interface Language)',
          subtitle: 'Выберите язык интерфейса рабочего окружения OmnesAgent.',
          control: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            decoration: BoxDecoration(
              color: const Color(0xFF1E2229),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: const Color(0xFF2D333F)),
            ),
            child: DropdownButton<String>(
              value: currentLanguage,
              underline: const SizedBox.shrink(),
              dropdownColor: const Color(0xFF1E2229),
              style: const TextStyle(fontSize: 12, color: Colors.white),
              items: const [
                DropdownMenuItem(value: 'Русский (Russian)', child: Text('Русский (Russian)')),
                DropdownMenuItem(value: 'English', child: Text('English')),
              ],
              onChanged: (val) {
                if (val != null) setState(() => currentLanguage = val);
              },
            ),
          ),
        ),
        const SizedBox(height: 16),

        // Memory ob2h AST
        _buildSettingCard(
          title: 'Долговременная память ob2h AST',
          subtitle: 'Управляет использованием графа фактов и символов кода в задачах без лишней траты токенов контекста.',
          control: Switch(
            value: memoryEnabled,
            activeColor: const Color(0xFF00D2FF),
            onChanged: (val) => setState(() => memoryEnabled = val),
          ),
        ),
        const SizedBox(height: 16),

        // Gateway Daemon
        _buildSettingCard(
          title: 'Gateway Runtime Daemon',
          subtitle: 'Подключение клиента к локальному Rust gateway демону и исполнителям инструментов.',
          control: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 200,
                height: 34,
                padding: const EdgeInsets.symmetric(horizontal: 10),
                decoration: BoxDecoration(
                  color: const Color(0xFF1E2229),
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(color: const Color(0xFF2D333F)),
                ),
                child: TextField(
                  controller: gatewayUrlController,
                  style: const TextStyle(fontSize: 12, fontFamily: 'Consolas', color: Colors.white),
                  decoration: const InputDecoration(border: InputBorder.none, isDense: true, contentPadding: EdgeInsets.only(top: 8)),
                ),
              ),
              const SizedBox(width: 8),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                decoration: BoxDecoration(
                  color: const Color(0xFF059669).withOpacity(0.15),
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(color: const Color(0xFF059669).withOpacity(0.4)),
                ),
                child: const Text('Подключено (3ms)', style: TextStyle(fontSize: 11, color: Color(0xFF10B981), fontWeight: FontWeight.bold)),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),

        // Terminal Font
        _buildSettingCard(
          title: 'Шрифт встроенного терминала',
          subtitle: 'Семейство моноширинных шрифтов для встроенной консоли и вывода bash/powershell.',
          control: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 260,
                height: 34,
                padding: const EdgeInsets.symmetric(horizontal: 10),
                decoration: BoxDecoration(
                  color: const Color(0xFF1E2229),
                  borderRadius: BorderRadius.circular(6),
                  border: Border.all(color: const Color(0xFF2D333F)),
                ),
                child: TextField(
                  controller: terminalFontController,
                  style: const TextStyle(fontSize: 12, fontFamily: 'Consolas', color: Colors.white),
                  decoration: const InputDecoration(border: InputBorder.none, isDense: true, contentPadding: EdgeInsets.only(top: 8)),
                ),
              ),
              const SizedBox(width: 8),
              ElevatedButton(
                onPressed: () {},
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF2A303C),
                  foregroundColor: Colors.white,
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                ),
                child: const Text('Сохранить', style: TextStyle(fontSize: 12)),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),

        // Enhanced Grep
        _buildSettingCard(
          title: 'Enhanced AST Grep (CodeGraph)',
          subtitle: 'Использовать поиск по AST графу вместо обычного ripgrep для быстрого поиска символов и определений.',
          control: Switch(
            value: enhancedGrep,
            activeColor: const Color(0xFF00D2FF),
            onChanged: (val) => setState(() => enhancedGrep = val),
          ),
        ),
      ],
    );
  }

  // ========================================================
  // SECTION: ПРОВАЙДЕРЫ И МОДЕЛИ (LLM Providers)
  // ========================================================
  Widget _buildProvidersSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: const [
                  Text(
                    'Конфигурация провайдеров моделей',
                    style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: Colors.white),
                  ),
                  SizedBox(height: 4),
                  Text(
                    'Настройка стандартных провайдеров из бэкенда и добавление собственных OpenAI-совместимых эндпоинтов.',
                    style: TextStyle(fontSize: 12, color: Color(0xFF94A3B8)),
                  ),
                ],
              ),
            ),
            const SizedBox(width: 16),
            ElevatedButton.icon(
              icon: const Icon(Icons.add, size: 14),
              label: const Text('Добавить провайдера'),
              onPressed: () => setState(() => isAddingProvider = !isAddingProvider),
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF00D2FF),
                foregroundColor: const Color(0xFF0F172A),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
              ),
            ),
          ],
        ),
        const SizedBox(height: 20),

        // Custom Provider Form if open
        if (isAddingProvider) ...[
          Container(
            padding: const EdgeInsets.all(18),
            decoration: BoxDecoration(
              color: const Color(0xFF1E222A),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.5)),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('Новый кастомный LLM провайдер', style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: Colors.white)),
                const SizedBox(height: 14),
                Row(
                  children: [
                    Expanded(
                      child: _buildFormInput('Название (например: OpenRouter, vLLM)', customNameController),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: _buildFormInput('Base URL (например: http://localhost:8000/v1)', customUrlController),
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                Row(
                  children: [
                    Expanded(
                      child: _buildFormInput('API Key (или оставьте пустым для локальных)', customKeyController, isPassword: true),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: _buildFormInput('Имя модели (например: Qwen/Qwen2.5-72B)', customModelController),
                    ),
                  ],
                ),
                const SizedBox(height: 14),
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    TextButton(
                      onPressed: () => setState(() => isAddingProvider = false),
                      child: const Text('Отмена', style: TextStyle(color: Color(0xFF94A3B8))),
                    ),
                    const SizedBox(width: 8),
                    ElevatedButton(
                      onPressed: _saveCustomProvider,
                      style: ElevatedButton.styleFrom(
                        backgroundColor: const Color(0xFF00D2FF),
                        foregroundColor: const Color(0xFF0F172A),
                      ),
                      child: const Text('Сохранить провайдера', style: TextStyle(fontWeight: FontWeight.bold)),
                    ),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),
        ],

        // List of Providers
        ...providers.map((p) {
          final isCustom = p['isCustom'] == true;
          final isConfigured = p['isConfigured'] == true;

          return Container(
            margin: const EdgeInsets.only(bottom: 12),
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: const Color(0xFF1B1D22),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: const Color(0xFF262A33)),
            ),
            child: Row(
              children: [
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: const Color(0xFF222834),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: const Color(0xFF2E3646)),
                  ),
                  child: const Center(
                    child: Icon(FontAwesomeIcons.robot, size: 16, color: Color(0xFF00D2FF)),
                  ),
                ),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Text(
                            p['name'],
                            style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: Colors.white),
                          ),
                          const SizedBox(width: 8),
                          if (isCustom)
                            Container(
                              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
                              decoration: BoxDecoration(
                                color: const Color(0xFF00D2FF).withOpacity(0.15),
                                borderRadius: BorderRadius.circular(4),
                              ),
                              child: const Text('CUSTOM', style: TextStyle(fontSize: 9, color: Color(0xFF00D2FF), fontWeight: FontWeight.bold)),
                            ),
                        ],
                      ),
                      const SizedBox(height: 4),
                      Text(
                        'URL: ${p['url']} • Модели: ${(p['models'] as List).join(', ')}',
                        style: const TextStyle(fontSize: 11, color: Color(0xFF94A3B8)),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ),
                ),
                Switch(
                  value: isConfigured,
                  activeColor: const Color(0xFF00D2FF),
                  onChanged: (val) {
                    setState(() => p['isConfigured'] = val);
                  },
                ),
                if (isCustom) ...[
                  const SizedBox(width: 8),
                  IconButton(
                    icon: const Icon(Icons.delete_outline, size: 18, color: Color(0xFFEF4444)),
                    onPressed: () {
                      setState(() => providers.remove(p));
                    },
                  ),
                ],
              ],
            ),
          );
        }),
      ],
    );
  }

  // ========================================================
  // SECTION: MCP СЕРВЕРЫ (Model Context Protocol)
  // ========================================================
  Widget _buildMcpServersSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: const [
                  Text(
                    'MCP Серверы и Инструменты',
                    style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: Colors.white),
                  ),
                  SizedBox(height: 4),
                  Text(
                    'Управление внешними серверами протокола MCP, расширяющими возможности агента инструментами.',
                    style: TextStyle(fontSize: 12, color: Color(0xFF94A3B8)),
                  ),
                ],
              ),
            ),
            const SizedBox(width: 16),
            ElevatedButton.icon(
              icon: const Icon(Icons.add, size: 14),
              label: const Text('Добавить MCP сервер'),
              onPressed: () => setState(() => isAddingMcp = !isAddingMcp),
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF00D2FF),
                foregroundColor: const Color(0xFF0F172A),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
              ),
            ),
          ],
        ),
        const SizedBox(height: 20),

        if (isAddingMcp) ...[
          Container(
            padding: const EdgeInsets.all(18),
            decoration: BoxDecoration(
              color: const Color(0xFF1E222A),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: const Color(0xFF00D2FF).withOpacity(0.5)),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text('Регистрация нового MCP сервера', style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: Colors.white)),
                const SizedBox(height: 12),
                _buildFormInput('Идентификатор сервера (например: postgres-mcp, docker-tools)', newMcpNameController),
                const SizedBox(height: 10),
                _buildFormInput('Команда запуска / бинарник (например: npx -y @modelcontextprotocol/server-postgres)', newMcpCommandController),
                const SizedBox(height: 14),
                Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    TextButton(
                      onPressed: () => setState(() => isAddingMcp = false),
                      child: const Text('Отмена', style: TextStyle(color: Color(0xFF94A3B8))),
                    ),
                    const SizedBox(width: 8),
                    ElevatedButton(
                      onPressed: _saveNewMcpServer,
                      style: ElevatedButton.styleFrom(
                        backgroundColor: const Color(0xFF00D2FF),
                        foregroundColor: const Color(0xFF0F172A),
                      ),
                      child: const Text('Зарегистрировать', style: TextStyle(fontWeight: FontWeight.bold)),
                    ),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),
        ],

        ...mcpServers.map((server) {
          final isEnabled = server['enabled'] == true;
          return Container(
            margin: const EdgeInsets.only(bottom: 12),
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: const Color(0xFF1B1D22),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: const Color(0xFF262A33)),
            ),
            child: Row(
              children: [
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: const Color(0xFF222834),
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: const Color(0xFF2E3646)),
                  ),
                  child: const Center(
                    child: Icon(Icons.extension, size: 18, color: Color(0xFF00D2FF)),
                  ),
                ),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Text(
                            server['name'],
                            style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: Colors.white, fontFamily: 'Consolas'),
                          ),
                          const SizedBox(width: 8),
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
                            decoration: BoxDecoration(
                              color: isEnabled ? const Color(0xFF10B981).withOpacity(0.15) : const Color(0xFF64748B).withOpacity(0.15),
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              isEnabled ? 'ACTIVE' : 'DISABLED',
                              style: TextStyle(fontSize: 9, color: isEnabled ? const Color(0xFF10B981) : const Color(0xFF94A3B8), fontWeight: FontWeight.bold),
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 4),
                      Text(
                        server['description'],
                        style: const TextStyle(fontSize: 12, color: Color(0xFFCBD5E1)),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        'Cmd: ${server['command']}',
                        style: const TextStyle(fontSize: 10, fontFamily: 'Consolas', color: Color(0xFF64748B)),
                      ),
                    ],
                  ),
                ),
                Switch(
                  value: isEnabled,
                  activeColor: const Color(0xFF00D2FF),
                  onChanged: (val) {
                    setState(() => server['enabled'] = val);
                  },
                ),
              ],
            ),
          );
        }),
      ],
    );
  }

  // ========================================================
  // SECTION: НАВЫКИ И ПЛАГИНЫ
  // ========================================================
  Widget _buildSkillsSection() {
    final List<Map<String, String>> skills = [
      {'name': 'ob2h', 'desc': 'Интеграция с долговременной памятью, фактами и AST анализом'},
      {'name': 'android-cli', 'desc': 'Сборка, эмуляторы и инспекция Android приложений'},
      {'name': 'chrome-devtools', 'desc': 'Автоматизация веб-браузера и интерактивный Element Picker'},
      {'name': 'science', 'desc': 'Научные базы данных и обработка биологических последовательностей'},
      {'name': 'workflow-skill-creator', 'desc': 'Автоматическое сохранение сессии в многоразовый навык'},
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: skills.map((s) {
        return Container(
          margin: const EdgeInsets.only(bottom: 12),
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: const Color(0xFF1B1D22),
            borderRadius: BorderRadius.circular(10),
            border: Border.all(color: const Color(0xFF262A33)),
          ),
          child: Row(
            children: [
              const Icon(Icons.auto_awesome, size: 16, color: Color(0xFF00D2FF)),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(s['name']!, style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: Colors.white)),
                    const SizedBox(height: 4),
                    Text(s['desc']!, style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8))),
                  ],
                ),
              ),
              const Text('Установлен', style: TextStyle(fontSize: 11, color: Color(0xFF10B981), fontWeight: FontWeight.bold)),
            ],
          ),
        );
      }).toList(),
    );
  }

  // ========================================================
  // SECTION: ОФОРМЛЕНИЕ (Appearance)
  // ========================================================
  Widget _buildAppearanceSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSettingCard(
          title: 'Цветовая схема',
          subtitle: 'Переключение между тёмной темой Slate ADE и светлой скандинавской темой.',
          control: Obx(() {
            final isDark = DesktopThemeController.to.isDarkMode.value;
            return Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                ElevatedButton.icon(
                  icon: const Icon(Icons.dark_mode_outlined, size: 14),
                  label: const Text('Тёмная (Dark Slate)'),
                  onPressed: () {
                    if (!isDark) DesktopThemeController.to.toggleTheme();
                  },
                  style: ElevatedButton.styleFrom(
                    backgroundColor: isDark ? const Color(0xFF00D2FF).withOpacity(0.2) : const Color(0xFF1E2229),
                    foregroundColor: isDark ? const Color(0xFF00D2FF) : Colors.white,
                    side: BorderSide(color: isDark ? const Color(0xFF00D2FF) : const Color(0xFF2D333F)),
                  ),
                ),
                const SizedBox(width: 10),
                ElevatedButton.icon(
                  icon: const Icon(Icons.light_mode_outlined, size: 14),
                  label: const Text('Светлая (Light)'),
                  onPressed: () {
                    if (isDark) DesktopThemeController.to.toggleTheme();
                  },
                  style: ElevatedButton.styleFrom(
                    backgroundColor: !isDark ? const Color(0xFF0284C7).withOpacity(0.2) : const Color(0xFF1E2229),
                    foregroundColor: !isDark ? const Color(0xFF0284C7) : Colors.white,
                    side: BorderSide(color: !isDark ? const Color(0xFF0284C7) : const Color(0xFF2D333F)),
                  ),
                ),
              ],
            );
          }),
        ),
      ],
    );
  }

  // ========================================================
  // SECTION: ПРОФИЛЬ ПОЛЬЗОВАТЕЛЯ (User Profile)
  // ========================================================
  Widget _buildProfileSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          padding: const EdgeInsets.all(20),
          decoration: BoxDecoration(
            color: const Color(0xFF1B1D22),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: const Color(0xFF262A33)),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Container(
                    width: 56,
                    height: 56,
                    decoration: BoxDecoration(
                      gradient: const LinearGradient(colors: [Color(0xFF00D2FF), Color(0xFF0072FF)]),
                      borderRadius: BorderRadius.circular(28),
                    ),
                    child: Center(
                      child: Text(
                        widget.userProfile.initials,
                        style: const TextStyle(fontSize: 20, fontWeight: FontWeight.bold, color: Colors.white),
                      ),
                    ),
                  ),
                  const SizedBox(width: 16),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(widget.userProfile.fullName, style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: Colors.white)),
                      const SizedBox(height: 4),
                      Text('${widget.userProfile.role} • Тариф ${widget.userProfile.tier}', style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8))),
                    ],
                  ),
                  const Spacer(),
                  ElevatedButton(
                    onPressed: () {
                      UserOnboardingDialog.show(
                        context,
                        initialProfile: widget.userProfile,
                        onSave: widget.onUpdateProfile,
                      );
                    },
                    style: ElevatedButton.styleFrom(
                      backgroundColor: const Color(0xFF00D2FF),
                      foregroundColor: const Color(0xFF0F172A),
                    ),
                    child: const Text('Пройти анкету заново'),
                  ),
                ],
              ),
              const Divider(height: 28, color: Color(0xFF282D38)),
              _buildProfileParam('Основной стек', widget.userProfile.primaryStack),
              _buildProfileParam('Стиль автономности', widget.userProfile.autonomyStyle),
              _buildProfileParam('Язык общения', widget.userProfile.language),
              _buildProfileParam('ob2h AST Память', widget.userProfile.enableAstMemory ? 'Включена (активна)' : 'Отключена'),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildProfileParam(String title, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        children: [
          SizedBox(width: 180, child: Text(title, style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8)))),
          Expanded(child: Text(value, style: const TextStyle(fontSize: 13, color: Colors.white, fontWeight: FontWeight.w500))),
        ],
      ),
    );
  }

  // ========================================================
  // SECTION: ob2h AST Память & Статистика
  // ========================================================
  Widget _buildMemoryAstSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSettingCard(
          title: 'Индексация проекта в ob2h',
          subtitle: 'Хранилище фактов о репозитории, зафиксированных решениях и ключевых зависимостях.',
          control: ElevatedButton(
            onPressed: () {},
            style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFF2A303C)),
            child: const Text('Переиндексировать AST'),
          ),
        ),
      ],
    );
  }

  Widget _buildUsageStatsSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSettingCard(
          title: 'Статистика токенов за текущую сессию',
          subtitle: 'Prompt: 14,250 tokens • Completion: 3,120 tokens • Сэкономлено через ob2h: ~42,000 tokens',
          control: const Text('Активно', style: TextStyle(color: Color(0xFF10B981), fontWeight: FontWeight.bold)),
        ),
      ],
    );
  }

  // ========================================================
  // HELPERS
  // ========================================================
  Widget _buildFormInput(String hint, TextEditingController controller, {bool isPassword = false}) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: const Color(0xFF14171E),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: const Color(0xFF2B3240)),
      ),
      child: TextField(
        controller: controller,
        obscureText: isPassword,
        style: const TextStyle(fontSize: 12, color: Colors.white),
        decoration: InputDecoration(
          hintText: hint,
          hintStyle: const TextStyle(fontSize: 12, color: Color(0xFF64748B)),
          border: InputBorder.none,
          isDense: true,
          contentPadding: const EdgeInsets.symmetric(vertical: 10),
        ),
      ),
    );
  }

  Widget _buildSettingCard({
    required String title,
    required String subtitle,
    required Widget control,
  }) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(18),
      decoration: BoxDecoration(
        color: const Color(0xFF1B1D22),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: const Color(0xFF262A33)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.bold,
                    color: Colors.white,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  subtitle,
                  style: const TextStyle(
                    fontSize: 12,
                    color: Color(0xFF94A3B8),
                    height: 1.4,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 24),
          control,
        ],
      ),
    );
  }
}
