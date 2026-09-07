// ZCode Settings View matching official ZCode "memory-and-network-settings-en.webp".

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import '../../theme/desktop_theme.dart';

class ZCodeSettingsView extends StatefulWidget {
  final VoidCallback onBackToWorkspace;
  final String initialSection;

  const ZCodeSettingsView({
    super.key,
    required this.onBackToWorkspace,
    this.initialSection = 'General',
  });

  @override
  State<ZCodeSettingsView> createState() => _ZCodeSettingsViewState();
}

class _ZCodeSettingsViewState extends State<ZCodeSettingsView> {
  late String selectedSection;
  bool memoryEnabled = true;
  bool inheritTerminal = true;
  bool enhancedGrep = true;
  final terminalFontController = TextEditingController(text: 'JetBrains Mono, SFMono-Regular, monospace');
  final gatewayUrlController = TextEditingController(text: 'http://127.0.0.1:42617');

  @override
  void initState() {
    super.initState();
    selectedSection = widget.initialSection;
  }

  @override
  void dispose() {
    terminalFontController.dispose();
    gatewayUrlController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: const Color(0xFF131518),
      child: Row(
        children: [
          // Left Settings Navigation (240px)
          Container(
            width: 240,
            decoration: const BoxDecoration(
              color: Color(0xFF16181D),
              border: Border(right: BorderSide(color: Color(0xFF23272F))),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Window top padding / Back button
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
                            'Back to Workspace',
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
                      _buildSectionHeader('Basic Settings'),
                      _buildNavItem('General', Icons.tune),
                      _buildNavItem('Appearance', Icons.brightness_6_outlined),
                      _buildNavItem('Model Settings', FontAwesomeIcons.brain),
                      _buildNavItem('Browser', FontAwesomeIcons.globe),

                      const SizedBox(height: 16),
                      _buildSectionHeader('Agent Capabilities'),
                      _buildNavItem('Plugins', FontAwesomeIcons.puzzlePiece),
                      _buildNavItem('Skills', Icons.auto_awesome),
                      _buildNavItem('Subagents', Icons.groups_outlined),
                      _buildNavItem('MCP Servers', Icons.extension_outlined),
                      _buildNavItem('Commands', FontAwesomeIcons.terminal),
                      _buildNavItem('Hooks', Icons.webhook),

                      const SizedBox(height: 16),
                      _buildSectionHeader('Data & Analytics'),
                      _buildNavItem('Index', Icons.shield_outlined),
                      _buildNavItem('Usage Statistics', Icons.bar_chart),
                    ],
                  ),
                ),

                // Bottom Connected User Info
                Container(
                  padding: const EdgeInsets.all(16),
                  decoration: const BoxDecoration(
                    border: Border(top: BorderSide(color: Color(0xFF23272F))),
                  ),
                  child: Row(
                    children: [
                      Container(
                        width: 28,
                        height: 28,
                        decoration: BoxDecoration(
                          color: const Color(0xFF23272F),
                          borderRadius: BorderRadius.circular(14),
                        ),
                        child: const Center(
                          child: Text(
                            'O',
                            style: TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 13),
                          ),
                        ),
                      ),
                      const SizedBox(width: 10),
                      const Text(
                        'OmnesAgent',
                        style: TextStyle(fontSize: 13, color: Colors.white, fontWeight: FontWeight.w500),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),

          // Main Settings Content
          Expanded(
            child: Container(
              color: const Color(0xFF131518),
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
                            fontSize: 22,
                            fontWeight: FontWeight.bold,
                            color: Colors.white,
                          ),
                        ),
                        const Spacer(),
                        IconButton(
                          icon: const Icon(Icons.close, size: 20, color: Color(0xFF94A3B8)),
                          tooltip: 'Close settings',
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
              color: isSelected ? Colors.white : const Color(0xFF94A3B8),
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
      case 'General':
        return _buildGeneralSection();
      case 'Appearance':
        return _buildAppearanceSection();
      case 'Model Settings':
        return _buildModelSettingsSection();
      case 'Browser':
        return _buildBrowserSettingsSection();
      default:
        return _buildGeneralSection();
    }
  }

  Widget _buildGeneralSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Interface Language Card
        _buildSettingCard(
          title: 'Interface Language',
          subtitle: 'Choose the display language for the app UI.',
          control: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            decoration: BoxDecoration(
              color: const Color(0xFF1E2229),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: const Color(0xFF2D333F)),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: const [
                Text('Russian (Русский)', style: TextStyle(fontSize: 12, color: Colors.white)),
                SizedBox(width: 8),
                Icon(Icons.keyboard_arrow_down, size: 16, color: Color(0xFF94A3B8)),
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),

        // Memory Card
        _buildSettingCard(
          title: 'Memory (ob2h AST)',
          subtitle: 'Controls whether new tasks and tasks restored after an app restart use persistent AST Memory. When enabled, the agent references code graphs without consuming token context.',
          control: Switch(
            value: memoryEnabled,
            activeColor: const Color(0xFF00D2FF),
            onChanged: (val) => setState(() => memoryEnabled = val),
          ),
        ),
        const SizedBox(height: 16),

        // Gateway Daemon Status & URL Card
        _buildSettingCard(
          title: 'Gateway Runtime Daemon',
          subtitle: 'Connects the ADE desktop client to the Rust gateway backend and tools (127.0.0.1:42617).',
          control: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 220,
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
                child: const Text('Connected (3ms)', style: TextStyle(fontSize: 11, color: Color(0xFF10B981), fontWeight: FontWeight.bold)),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),

        // Inherit Terminal Profile Card
        _buildSettingCard(
          title: 'Inherit System Terminal Profile',
          subtitle: 'When launching the built-in terminal, inherit the login shell environment, proxy, variables, and local terminal font whenever possible.',
          control: Switch(
            value: inheritTerminal,
            activeColor: const Color(0xFF00D2FF),
            onChanged: (val) => setState(() => inheritTerminal = val),
          ),
        ),
        const SizedBox(height: 16),

        // Terminal Font Card
        _buildSettingCard(
          title: 'Terminal Font',
          subtitle: 'Leave blank to detect the system terminal settings automatically; enter a value to override the ZCode terminal font.',
          control: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 280,
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
                child: const Text('Save', style: TextStyle(fontSize: 12)),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),

        // Enhanced Find and Grep Card
        _buildSettingCard(
          title: 'Enhanced Find and Grep (CodeGraph AST)',
          subtitle: 'Use enhanced AST code search in new chats and tasks. Indexes symbols and cross-file references.',
          control: Switch(
            value: enhancedGrep,
            activeColor: const Color(0xFF00D2FF),
            onChanged: (val) => setState(() => enhancedGrep = val),
          ),
        ),
      ],
    );
  }

  Widget _buildAppearanceSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSettingCard(
          title: 'Color Theme',
          subtitle: 'Switch between sleek ZCode Dark slate and crisp Scandinavian Light theme.',
          control: Obx(() {
            final isDark = DesktopThemeController.to.isDarkMode.value;
            return Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                InkWell(
                  onTap: () {
                    if (!isDark) DesktopThemeController.to.toggleTheme();
                  },
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    decoration: BoxDecoration(
                      color: isDark ? const Color(0xFF00D2FF).withOpacity(0.15) : const Color(0xFF1E2229),
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: isDark ? const Color(0xFF00D2FF) : const Color(0xFF2D333F)),
                    ),
                    child: Row(
                      children: const [
                        Icon(Icons.dark_mode_outlined, size: 14, color: Color(0xFF00D2FF)),
                        SizedBox(width: 6),
                        Text('Dark (ZCode)', style: TextStyle(fontSize: 12, color: Colors.white, fontWeight: FontWeight.bold)),
                      ],
                    ),
                  ),
                ),
                const SizedBox(width: 10),
                InkWell(
                  onTap: () {
                    if (isDark) DesktopThemeController.to.toggleTheme();
                  },
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    decoration: BoxDecoration(
                      color: !isDark ? const Color(0xFF0284C7).withOpacity(0.15) : const Color(0xFF1E2229),
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: !isDark ? const Color(0xFF0284C7) : const Color(0xFF2D333F)),
                    ),
                    child: Row(
                      children: const [
                        Icon(Icons.light_mode_outlined, size: 14, color: Color(0xFF0284C7)),
                        SizedBox(width: 6),
                        Text('Light', style: TextStyle(fontSize: 12, color: Colors.white)),
                      ],
                    ),
                  ),
                ),
              ],
            );
          }),
        ),
      ],
    );
  }

  Widget _buildModelSettingsSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSettingCard(
          title: 'Active Coding Model',
          subtitle: 'Primary foundation model for autonomous coding and goal execution.',
          control: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            decoration: BoxDecoration(
              color: const Color(0xFF1E2229),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: const Color(0xFF2D333F)),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: const [
                Text('GLM-5.3 (1M Long-Context)', style: TextStyle(fontSize: 12, color: Colors.white)),
                SizedBox(width: 8),
                Icon(Icons.keyboard_arrow_down, size: 16, color: Color(0xFF94A3B8)),
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        _buildSettingCard(
          title: 'Bring Your Own Key (BYOK)',
          subtitle: 'Configure Anthropic Claude 3.5, DeepSeek V3, OpenAI, or local Ollama endpoints.',
          control: ElevatedButton(
            onPressed: () {},
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF2A303C),
              foregroundColor: Colors.white,
            ),
            child: const Text('Manage API Keys', style: TextStyle(fontSize: 12)),
          ),
        ),
      ],
    );
  }

  Widget _buildBrowserSettingsSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildSettingCard(
          title: 'Browser Automation & Element Picker',
          subtitle: 'Allows the agent to open web applications locally (http://localhost:3000) and pick elements.',
          control: Switch(
            value: true,
            activeColor: const Color(0xFF00D2FF),
            onChanged: (val) {},
          ),
        ),
      ],
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
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
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
        ],
      ),
    );
  }
}
