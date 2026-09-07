// Desktop Inspector Panel: ZCode ADE Right Tool Canvas with Live Browser & Element Picker,
// Integrated Terminal, Markdown/Mermaid Preview, and Side Chat (/side, /btw).

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import '../../theme/desktop_theme.dart';
import '../workspace/task_workspace_controller.dart';

class DesktopInspectorPanel extends StatefulWidget {
  final DesktopTaskWorkspaceController controller;
  final int initialTabIndex;
  final VoidCallback? onClose;

  const DesktopInspectorPanel({
    super.key,
    required this.controller,
    this.initialTabIndex = 0,
    this.onClose,
  });

  @override
  State<DesktopInspectorPanel> createState() => _DesktopInspectorPanelState();
}

class _DesktopInspectorPanelState extends State<DesktopInspectorPanel>
    with SingleTickerProviderStateMixin {
  late TabController tabController;

  // Live Browser State
  final browserUrlController = TextEditingController(text: 'http://localhost:3000/dashboard');
  bool isElementPickerActive = false;
  String? hoveredElementSelector;
  String? selectedElementSelector;
  String? selectedElementTag;
  String? selectedElementText;
  int browserViewportMode = 0; // 0: Desktop, 1: Tablet, 2: Mobile

  // Terminal State
  int selectedTerminalSession = 0;
  final terminalInputController = TextEditingController();
  final List<String> terminalSessions = ['1: cargo test', '2: flutter analyze', '3: bash'];
  final List<List<String>> terminalLogs = [
    [
      '\$ cargo test --package omnesagent-runtime',
      '   Compiling omnesagent-runtime v0.1.0 (C:\\Projects\\Omnes-agent\\backend\\crates\\omnesagent-runtime)',
      '   Finished test [unoptimized + debuginfo] target(s) in 1.42s',
      '     Running unittests src\\lib.rs (target\\debug\\deps\\omnesagent_runtime-84a1.exe)',
      '',
      'running 18 tests',
      'test agent::tests::test_memory_ast_provider ... ok',
      'test runtime::tests::test_gateway_ws_connection ... ok',
      'test security::tests::test_sop_permission_mode ... ok',
      'test tools::tests::test_browser_element_picker ... ok',
      'test stream::tests::test_goal_mode_iteration_divider ... ok',
      '',
      'test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out',
    ],
    [
      '\$ flutter analyze lib/features/workspace/task_workspace_view.dart',
      'Analyzing workspace/task_workspace_view.dart...',
      '• No issues found! (ran in 1.1s)',
      '',
      '\$ flutter analyze lib/features/inspector/inspector_panel.dart',
      'Analyzing inspector/inspector_panel.dart...',
      '• No issues found! (ran in 0.9s)',
    ],
    [
      '\$ git status --short',
      ' M frontend/desktop/lib/features/inspector/inspector_panel.dart',
      ' M frontend/desktop/lib/widgets/desktop_sidebar.dart',
      ' M frontend/desktop/lib/widgets/desktop_titlebar.dart',
      '?? frontend/desktop/assets/Logo/',
      '',
      '\$ echo "ZCode ADE background daemon ready on 127.0.0.1:42617"',
      'ZCode ADE background daemon ready on 127.0.0.1:42617',
    ],
  ];

  // Preview State
  int previewModeIndex = 0; // 0: Markdown Report, 1: Architecture Mermaid, 2: Git Diff

  // Side Chat State
  final sideChatController = TextEditingController();
  final List<Map<String, String>> sideMessages = [
    {
      'role': 'user',
      'text': 'Как Element Picker передает селектор в Composer?',
    },
    {
      'role': 'bot',
      'text': 'При клике на DOM-узел в Live Browser извлекаются тег, селектор и текст, после чего вызывается addElementContext(...) и селектор добавляется в Composer в виде чипа-вложения.',
    },
  ];

  @override
  void initState() {
    super.initState();
    tabController = TabController(
      length: 4,
      vsync: this,
      initialIndex: widget.initialTabIndex,
    );
  }

  @override
  void dispose() {
    tabController.dispose();
    browserUrlController.dispose();
    terminalInputController.dispose();
    sideChatController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 460,
      decoration: BoxDecoration(
        color: DesktopTheme.bgSidebar,
        border: Border(
          left: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
        ),
      ),
      child: Column(
        children: [
          // Inspector Header with 4 ZCode Tabs and Close button
          Container(
            height: 44,
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurface,
              border: Border(
                bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
              ),
            ),
            child: Row(
              children: [
                Expanded(
                  child: TabBar(
                    controller: tabController,
                    labelColor: DesktopTheme.accentSky,
                    unselectedLabelColor: DesktopTheme.textMuted,
                    indicatorColor: DesktopTheme.accentSky,
                    indicatorWeight: 2.5,
                    indicatorSize: TabBarIndicatorSize.tab,
                    labelStyle: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold),
                    tabs: const [
                      Tab(
                        iconMargin: EdgeInsets.only(bottom: 2),
                        icon: Icon(FontAwesomeIcons.globe, size: 13),
                        text: 'Live Browser',
                      ),
                      Tab(
                        iconMargin: EdgeInsets.only(bottom: 2),
                        icon: Icon(FontAwesomeIcons.terminal, size: 13),
                        text: 'Terminal',
                      ),
                      Tab(
                        iconMargin: EdgeInsets.only(bottom: 2),
                        icon: Icon(FontAwesomeIcons.eye, size: 13),
                        text: 'Preview',
                      ),
                      Tab(
                        iconMargin: EdgeInsets.only(bottom: 2),
                        icon: Icon(FontAwesomeIcons.comments, size: 13),
                        text: 'Side Chat',
                      ),
                    ],
                  ),
                ),
                if (widget.onClose != null)
                  IconButton(
                    icon: const Icon(Icons.close, size: 16, color: Color(0xFF94A3B8)),
                    tooltip: 'Close panel',
                    onPressed: widget.onClose,
                  ),
              ],
            ),
          ),

          // Tab Views
          Expanded(
            child: TabBarView(
              controller: tabController,
              children: [
                _buildLiveBrowserTab(),
                _buildTerminalTab(),
                _buildPreviewTab(),
                _buildSideChatTab(),
              ],
            ),
          ),
        ],
      ),
    );
  }

  // ==========================================
  // TAB 1: LIVE BROWSER & ELEMENT PICKER
  // ==========================================
  Widget _buildLiveBrowserTab() {
    return Column(
      children: [
        // Address bar and controls
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            border: Border(
              bottom: BorderSide(color: DesktopTheme.borderSubtle),
            ),
          ),
          child: Column(
            children: [
              Row(
                children: [
                  // Back / Forward / Refresh
                  Icon(Icons.arrow_back, size: 15, color: DesktopTheme.textMuted),
                  const SizedBox(width: 6),
                  Icon(Icons.arrow_forward, size: 15, color: DesktopTheme.textMuted.withOpacity(0.4)),
                  const SizedBox(width: 8),
                  InkWell(
                    onTap: () {
                      setState(() {
                        selectedElementSelector = null;
                      });
                    },
                    borderRadius: BorderRadius.circular(4),
                    child: Icon(Icons.refresh, size: 16, color: DesktopTheme.textSecondary),
                  ),
                  const SizedBox(width: 8),

                  // URL Input Bar
                  Expanded(
                    child: Container(
                      height: 28,
                      padding: const EdgeInsets.symmetric(horizontal: 8),
                      decoration: BoxDecoration(
                        color: DesktopTheme.bgCanvas,
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(color: DesktopTheme.borderSubtle),
                      ),
                      child: Row(
                        children: [
                          const Icon(Icons.lock_outline, size: 12, color: DesktopTheme.statusSuccess),
                          const SizedBox(width: 6),
                          Expanded(
                            child: TextField(
                              controller: browserUrlController,
                              style: TextStyle(
                                fontSize: 11,
                                fontFamily: 'Consolas',
                                color: DesktopTheme.textPrimary,
                              ),
                              decoration: const InputDecoration(
                                isDense: true,
                                border: InputBorder.none,
                                contentPadding: EdgeInsets.zero,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),

                  // Viewport toggles: Desktop / Mobile
                  InkWell(
                    onTap: () => setState(() => browserViewportMode = (browserViewportMode + 1) % 3),
                    borderRadius: BorderRadius.circular(4),
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
                      decoration: BoxDecoration(
                        color: DesktopTheme.bgSurfaceElevated,
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(color: DesktopTheme.borderSubtle),
                      ),
                      child: Icon(
                        browserViewportMode == 0
                            ? Icons.desktop_windows
                            : (browserViewportMode == 1 ? Icons.tablet : Icons.smartphone),
                        size: 14,
                        color: DesktopTheme.accentSky,
                      ),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 8),

              // Element Picker Toolbar Strip
              Row(
                children: [
                  // Crosshair Button
                  InkWell(
                    onTap: () {
                      setState(() {
                        isElementPickerActive = !isElementPickerActive;
                        if (!isElementPickerActive) {
                          hoveredElementSelector = null;
                        }
                      });
                    },
                    borderRadius: BorderRadius.circular(5),
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                      decoration: BoxDecoration(
                        color: isElementPickerActive
                            ? DesktopTheme.accentSky.withOpacity(0.18)
                            : DesktopTheme.bgCanvas,
                        borderRadius: BorderRadius.circular(5),
                        border: Border.all(
                          color: isElementPickerActive
                              ? DesktopTheme.accentSky
                              : DesktopTheme.borderSubtle,
                          width: 1.2,
                        ),
                      ),
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(
                            FontAwesomeIcons.crosshairs,
                            size: 12,
                            color: isElementPickerActive
                                ? DesktopTheme.accentSky
                                : DesktopTheme.textMuted,
                          ),
                          const SizedBox(width: 6),
                          Text(
                            isElementPickerActive ? 'Element Picker Active' : 'Pick Element',
                            style: TextStyle(
                              fontSize: 11,
                              fontWeight: FontWeight.bold,
                              color: isElementPickerActive
                                  ? DesktopTheme.accentSky
                                  : DesktopTheme.textSecondary,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                  const Spacer(),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                    decoration: BoxDecoration(
                      color: DesktopTheme.statusSuccess.withOpacity(0.1),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: const Text(
                      'DOM READY · 60 FPS',
                      style: TextStyle(
                        fontSize: 9,
                        fontFamily: 'Consolas',
                        fontWeight: FontWeight.bold,
                        color: DesktopTheme.statusSuccess,
                      ),
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),

        // Live Webview Viewport / Simulator
        Expanded(
          child: Container(
            color: DesktopTheme.bgCanvas,
            child: Center(
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 200),
                width: browserViewportMode == 0
                    ? double.infinity
                    : (browserViewportMode == 1 ? 380 : 310),
                margin: const EdgeInsets.all(10),
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: DesktopTheme.borderMedium, width: 1.5),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withOpacity(0.2),
                      blurRadius: 16,
                      offset: const Offset(0, 4),
                    ),
                  ],
                ),
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(7),
                  child: Column(
                    children: [
                      // Simulated Web Page Header
                      _buildSimulatedElement(
                        tag: 'header',
                        selector: 'header.navbar-main',
                        text: 'OmnesAgent Web Dashboard',
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                          color: const Color(0xFF0F172A),
                          child: Row(
                            children: [
                              const Icon(Icons.bolt, color: Color(0xFF00D2FF), size: 16),
                              const SizedBox(width: 8),
                              const Text(
                                'OmnesAgent Web',
                                style: TextStyle(
                                  color: Colors.white,
                                  fontSize: 12,
                                  fontWeight: FontWeight.bold,
                                ),
                              ),
                              const Spacer(),
                              _buildSimulatedElement(
                                tag: 'a',
                                selector: 'a.nav-link-docs',
                                text: 'Docs',
                                child: const Text(
                                  'Docs',
                                  style: TextStyle(color: Color(0xFF94A3B8), fontSize: 11),
                                ),
                              ),
                              const SizedBox(width: 12),
                              _buildSimulatedElement(
                                tag: 'button',
                                selector: 'button.btn-sign-in',
                                text: 'Sign In',
                                child: Container(
                                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                                  decoration: BoxDecoration(
                                    color: const Color(0xFF00D2FF),
                                    borderRadius: BorderRadius.circular(4),
                                  ),
                                  child: const Text(
                                    'Sign In',
                                    style: TextStyle(
                                      color: Color(0xFF090D12),
                                      fontSize: 10,
                                      fontWeight: FontWeight.bold,
                                    ),
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),

                      // Simulated Web Page Hero Body
                      Expanded(
                        child: Container(
                          color: const Color(0xFFF8FAFC),
                          padding: const EdgeInsets.all(16),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              _buildSimulatedElement(
                                tag: 'h1',
                                selector: 'h1.hero-title',
                                text: 'Agentic Development Environment',
                                child: const Text(
                                  'Agentic Development\nEnvironment',
                                  style: TextStyle(
                                    fontSize: 17,
                                    fontWeight: FontWeight.w800,
                                    color: Color(0xFF0F172A),
                                    height: 1.2,
                                  ),
                                ),
                              ),
                              const SizedBox(height: 8),
                              _buildSimulatedElement(
                                tag: 'p',
                                selector: 'p.hero-subtitle',
                                text: 'Autonomous coding agent powered by ZCode architecture.',
                                child: const Text(
                                  'Autonomous coding agent powered by ZCode architecture and multimodal visual inspection.',
                                  style: TextStyle(
                                    fontSize: 11,
                                    color: Color(0xFF64748B),
                                  ),
                                ),
                              ),
                              const SizedBox(height: 16),

                              // Simulated Action Button
                              _buildSimulatedElement(
                                tag: 'button',
                                selector: 'button.action-btn-primary',
                                text: 'Run Integration Tests',
                                child: Container(
                                  width: double.infinity,
                                  padding: const EdgeInsets.symmetric(vertical: 8),
                                  decoration: BoxDecoration(
                                    color: const Color(0xFF0284C7),
                                    borderRadius: BorderRadius.circular(6),
                                  ),
                                  child: const Center(
                                    child: Text(
                                      'Run Integration Tests',
                                      style: TextStyle(
                                        color: Colors.white,
                                        fontSize: 11,
                                        fontWeight: FontWeight.bold,
                                      ),
                                    ),
                                  ),
                                ),
                              ),
                              const SizedBox(height: 14),

                              // Simulated Code Card
                              Expanded(
                                child: _buildSimulatedElement(
                                  tag: 'div',
                                  selector: 'div.code-preview-card',
                                  text: 'Runtime Status: Healthy',
                                  child: Container(
                                    width: double.infinity,
                                    padding: const EdgeInsets.all(10),
                                    decoration: BoxDecoration(
                                      color: const Color(0xFF0F172A),
                                      borderRadius: BorderRadius.circular(6),
                                    ),
                                    child: Column(
                                      crossAxisAlignment: CrossAxisAlignment.start,
                                      children: const [
                                        Text(
                                          '// omnesagent-runtime: active',
                                          style: TextStyle(
                                            fontSize: 10,
                                            fontFamily: 'Consolas',
                                            color: Color(0xFF64748B),
                                          ),
                                        ),
                                        SizedBox(height: 4),
                                        Text(
                                          'const port = 42617;\nawait startDaemon({ mode: "ade" });',
                                          style: TextStyle(
                                            fontSize: 11,
                                            fontFamily: 'Consolas',
                                            color: Color(0xFF38BDF8),
                                          ),
                                        ),
                                      ],
                                    ),
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),

        // Picked Element Details & Inject Button
        if (selectedElementSelector != null)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurface,
              border: Border(top: BorderSide(color: DesktopTheme.borderSubtle)),
            ),
            child: Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Row(
                        children: [
                          const Icon(Icons.check_circle, size: 13, color: DesktopTheme.accentSky),
                          const SizedBox(width: 6),
                          Text(
                            'Element Selected:',
                            style: TextStyle(
                              fontSize: 11,
                              fontWeight: FontWeight.bold,
                              color: DesktopTheme.textPrimary,
                            ),
                          ),
                          const SizedBox(width: 6),
                          Text(
                            '<$selectedElementTag>',
                            style: const TextStyle(
                              fontSize: 11,
                              fontFamily: 'Consolas',
                              color: DesktopTheme.accentCyan,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 2),
                      Text(
                        selectedElementSelector!,
                        style: TextStyle(
                          fontSize: 11,
                          fontFamily: 'Consolas',
                          color: DesktopTheme.textSecondary,
                        ),
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ),
                ),
                ElevatedButton.icon(
                  onPressed: () {
                    widget.controller.addElementContext(
                      selector: selectedElementSelector!,
                      text: selectedElementText ?? '',
                      tag: selectedElementTag ?? 'div',
                    );
                    Get.snackbar(
                      'Element Injected',
                      'Селектор $selectedElementSelector добавлен в контекст Composer.',
                      snackPosition: SnackPosition.BOTTOM,
                      backgroundColor: DesktopTheme.bgSurfaceElevated,
                      colorText: DesktopTheme.accentSky,
                      duration: const Duration(seconds: 2),
                      margin: const EdgeInsets.all(12),
                    );
                  },
                  icon: const Icon(Icons.add, size: 14),
                  label: const Text('Add to Composer', style: TextStyle(fontSize: 11)),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: DesktopTheme.accentSky,
                    foregroundColor: const Color(0xFF090D12),
                    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                  ),
                ),
              ],
            ),
          ),
      ],
    );
  }

  Widget _buildSimulatedElement({
    required String tag,
    required String selector,
    required String text,
    required Widget child,
  }) {
    final isHovered = hoveredElementSelector == selector && isElementPickerActive;
    final isSelected = selectedElementSelector == selector;

    return MouseRegion(
      cursor: isElementPickerActive ? SystemMouseCursors.precise : SystemMouseCursors.basic,
      onEnter: (_) {
        if (isElementPickerActive) {
          setState(() => hoveredElementSelector = selector);
        }
      },
      onExit: (_) {
        if (isElementPickerActive && hoveredElementSelector == selector) {
          setState(() => hoveredElementSelector = null);
        }
      },
      child: GestureDetector(
        onTap: () {
          if (isElementPickerActive) {
            setState(() {
              selectedElementSelector = selector;
              selectedElementTag = tag;
              selectedElementText = text;
              isElementPickerActive = false;
            });
          }
        },
        child: Stack(
          clipBehavior: Clip.none,
          children: [
            Container(
              decoration: BoxDecoration(
                border: Border.all(
                  color: isSelected
                      ? const Color(0xFF0284C7)
                      : (isHovered ? const Color(0xFF00D2FF) : Colors.transparent),
                  width: (isSelected || isHovered) ? 2.0 : 0.0,
                ),
              ),
              child: child,
            ),
            if (isHovered || isSelected)
              Positioned(
                top: -16,
                left: 0,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                  decoration: BoxDecoration(
                    color: isSelected ? const Color(0xFF0284C7) : const Color(0xFF00D2FF),
                    borderRadius: BorderRadius.circular(2),
                  ),
                  child: Text(
                    selector,
                    style: const TextStyle(
                      fontSize: 9,
                      fontFamily: 'Consolas',
                      fontWeight: FontWeight.bold,
                      color: Colors.black,
                    ),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  // ==========================================
  // TAB 2: TERMINAL & GIT (Dark Slate Container)
  // ==========================================
  Widget _buildTerminalTab() {
    return Container(
      color: const Color(0xFF0F172A), // Always dark slate for code legibility
      child: Column(
        children: [
          // Terminal Session Tabs Strip
          Container(
            height: 34,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: const BoxDecoration(
              color: Color(0xFF0B101E),
              border: Border(
                bottom: BorderSide(color: Color(0xFF1E293B)),
              ),
            ),
            child: Row(
              children: [
                for (int i = 0; i < terminalSessions.length; i++)
                  InkWell(
                    onTap: () => setState(() => selectedTerminalSession = i),
                    borderRadius: BorderRadius.circular(4),
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                      margin: const EdgeInsets.only(right: 6),
                      decoration: BoxDecoration(
                        color: selectedTerminalSession == i
                            ? const Color(0xFF1E293B)
                            : Colors.transparent,
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(
                          color: selectedTerminalSession == i
                              ? const Color(0xFF38BDF8)
                              : Colors.transparent,
                          width: 0.8,
                        ),
                      ),
                      child: Text(
                        terminalSessions[i],
                        style: TextStyle(
                          fontSize: 10,
                          fontFamily: 'Consolas',
                          fontWeight: selectedTerminalSession == i
                              ? FontWeight.bold
                              : FontWeight.normal,
                          color: selectedTerminalSession == i
                              ? const Color(0xFF38BDF8)
                              : const Color(0xFF94A3B8),
                        ),
                      ),
                    ),
                  ),
                const Spacer(),
                IconButton(
                  tooltip: 'Clear terminal output',
                  icon: const Icon(FontAwesomeIcons.trashCan, size: 11, color: Color(0xFF64748B)),
                  onPressed: () {
                    setState(() {
                      terminalLogs[selectedTerminalSession].clear();
                    });
                  },
                ),
              ],
            ),
          ),

          // Terminal Output Log
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.all(12),
              itemCount: terminalLogs[selectedTerminalSession].length,
              itemBuilder: (context, index) {
                final line = terminalLogs[selectedTerminalSession][index];
                Color lineColor = const Color(0xFFE2E8F0);
                FontWeight fontWeight = FontWeight.normal;

                if (line.startsWith('\$')) {
                  lineColor = const Color(0xFF38BDF8); // Cyan prompt
                  fontWeight = FontWeight.bold;
                } else if (line.contains('ok') || line.contains('No issues found') || line.contains('passed')) {
                  lineColor = const Color(0xFF34D399); // Emerald success
                } else if (line.contains('error') || line.contains('failed')) {
                  lineColor = const Color(0xFFF87171); // Red error
                } else if (line.contains('Compiling') || line.contains('Analyzing')) {
                  lineColor = const Color(0xFFFBBF24); // Amber status
                }

                return Padding(
                  padding: const EdgeInsets.symmetric(vertical: 1.5),
                  child: SelectableText(
                    line,
                    style: TextStyle(
                      fontSize: 11,
                      fontFamily: 'Consolas',
                      color: lineColor,
                      fontWeight: fontWeight,
                      height: 1.35,
                    ),
                  ),
                );
              },
            ),
          ),

          // Terminal Quick Action Bar
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: const BoxDecoration(
              color: Color(0xFF0B101E),
              border: Border(top: BorderSide(color: Color(0xFF1E293B))),
            ),
            child: Row(
              children: [
                _buildQuickCommandChip('cargo test', () {
                  setState(() {
                    terminalLogs[0].add('\$ cargo test');
                    terminalLogs[0].add('test result: ok. All tests passing.');
                  });
                }),
                const SizedBox(width: 6),
                _buildQuickCommandChip('flutter analyze', () {
                  setState(() {
                    terminalLogs[1].add('\$ flutter analyze');
                    terminalLogs[1].add('• 0 issues found! (ran in 0.8s)');
                  });
                }),
                const SizedBox(width: 6),
                _buildQuickCommandChip('git diff', () {
                  setState(() {
                    terminalLogs[2].add('\$ git diff --stat');
                    terminalLogs[2].add(' 4 files changed, 142 insertions(+), 28 deletions(-)');
                  });
                }),
              ],
            ),
          ),

          // Terminal Input Prompt
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
            color: const Color(0xFF070A12),
            child: Row(
              children: [
                const Text(
                  '\$ ',
                  style: TextStyle(
                    fontSize: 13,
                    fontFamily: 'Consolas',
                    fontWeight: FontWeight.bold,
                    color: Color(0xFF38BDF8),
                  ),
                ),
                Expanded(
                  child: TextField(
                    controller: terminalInputController,
                    style: const TextStyle(
                      fontSize: 12,
                      fontFamily: 'Consolas',
                      color: Colors.white,
                    ),
                    decoration: const InputDecoration(
                      hintText: 'Enter shell command...',
                      hintStyle: TextStyle(fontSize: 11, color: Color(0xFF64748B)),
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                    ),
                    onSubmitted: (cmd) {
                      if (cmd.trim().isEmpty) return;
                      setState(() {
                        terminalLogs[selectedTerminalSession].add('\$ $cmd');
                        terminalLogs[selectedTerminalSession].add('Command executed successfully.');
                        terminalInputController.clear();
                      });
                    },
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildQuickCommandChip(String label, VoidCallback onTap) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(4),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 3),
        decoration: BoxDecoration(
          color: const Color(0xFF1E293B),
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: const Color(0xFF334155)),
        ),
        child: Text(
          label,
          style: const TextStyle(
            fontSize: 10,
            fontFamily: 'Consolas',
            color: Color(0xFF94A3B8),
          ),
        ),
      ),
    );
  }

  // ==========================================
  // TAB 3: PREVIEW (Markdown, Mermaid, Diff)
  // ==========================================
  Widget _buildPreviewTab() {
    return Column(
      children: [
        // Mode Selector (Markdown / Mermaid / Diff)
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
          ),
          child: Row(
            children: [
              _buildPreviewModeButton(0, 'Markdown Report', FontAwesomeIcons.fileLines),
              const SizedBox(width: 8),
              _buildPreviewModeButton(1, 'Architecture Graph', FontAwesomeIcons.sitemap),
              const SizedBox(width: 8),
              _buildPreviewModeButton(2, 'Git Diff', FontAwesomeIcons.codeBranch),
            ],
          ),
        ),

        // Preview Content
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: previewModeIndex == 0
                ? _buildMarkdownReportPreview()
                : (previewModeIndex == 1
                    ? _buildMermaidDiagramPreview()
                    : _buildGitDiffPreview()),
          ),
        ),
      ],
    );
  }

  Widget _buildPreviewModeButton(int index, String title, IconData icon) {
    final isSelected = previewModeIndex == index;
    return InkWell(
      onTap: () => setState(() => previewModeIndex = index),
      borderRadius: BorderRadius.circular(5),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: isSelected ? DesktopTheme.accentSky.withOpacity(0.14) : Colors.transparent,
          borderRadius: BorderRadius.circular(5),
          border: Border.all(
            color: isSelected ? DesktopTheme.accentSky : DesktopTheme.borderSubtle,
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 11, color: isSelected ? DesktopTheme.accentSky : DesktopTheme.textMuted),
            const SizedBox(width: 6),
            Text(
              title,
              style: TextStyle(
                fontSize: 11,
                fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                color: isSelected ? DesktopTheme.accentSky : DesktopTheme.textSecondary,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMarkdownReportPreview() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'OmnesAgent ADE Specification v1.0',
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.bold,
            color: DesktopTheme.textPrimary,
          ),
        ),
        const SizedBox(height: 6),
        Text(
          'Architectural validation report for desktop workstation client.',
          style: TextStyle(fontSize: 12, color: DesktopTheme.textSecondary),
        ),
        const SizedBox(height: 14),

        Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: DesktopTheme.borderSubtle),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                '✓ Verification Results',
                style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: DesktopTheme.statusSuccess),
              ),
              const SizedBox(height: 6),
              Text('• ZCode ADE 3-pane workstation layout mounted', style: TextStyle(fontSize: 11, color: DesktopTheme.textSecondary)),
              Text('• Permission Mode switcher (Shift+Tab) wired to controller', style: TextStyle(fontSize: 11, color: DesktopTheme.textSecondary)),
              Text('• Goal Mode (/goal) tracking with iteration dividers active', style: TextStyle(fontSize: 11, color: DesktopTheme.textSecondary)),
              Text('• Live Browser & Element Picker integration verified', style: TextStyle(fontSize: 11, color: DesktopTheme.textSecondary)),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildMermaidDiagramPreview() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            const Icon(FontAwesomeIcons.sitemap, size: 14, color: DesktopTheme.accentSky),
            const SizedBox(width: 8),
            Text(
              'ADE Architecture Flow (Mermaid)',
              style: TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
            ),
          ],
        ),
        const SizedBox(height: 14),
        Container(
          width: double.infinity,
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: DesktopTheme.borderSubtle),
          ),
          child: Column(
            children: [
              _buildDiagramNode('Agent Goal Mode (/goal)', DesktopTheme.accentSky),
              Icon(Icons.arrow_downward, size: 16, color: DesktopTheme.textMuted),
              _buildDiagramNode('AST CodeGraph & Tool Execution', DesktopTheme.statusWarning),
              Icon(Icons.arrow_downward, size: 16, color: DesktopTheme.textMuted),
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  _buildDiagramNode('Live Browser', DesktopTheme.accentCyan),
                  const SizedBox(width: 12),
                  _buildDiagramNode('PTY Terminal', DesktopTheme.statusSuccess),
                ],
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildDiagramNode(String title, Color color) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
      margin: const EdgeInsets.symmetric(vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.12),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: color.withOpacity(0.5), width: 1.2),
      ),
      child: Text(
        title,
        style: TextStyle(
          fontSize: 11,
          fontWeight: FontWeight.bold,
          color: color,
        ),
      ),
    );
  }

  Widget _buildGitDiffPreview() {
    final diffLines = [
      {'type': 'header', 'text': 'diff --git a/desktop_sidebar.dart b/desktop_sidebar.dart'},
      {'type': 'header', 'text': '@@ -12,6 +12,18 @@ class DesktopSidebar'},
      {'type': 'del', 'text': '- const oldTaskRow();'},
      {'type': 'add', 'text': '+ _buildDiffBadge(task.addedLines, task.deletedLines);'},
      {'type': 'add', 'text': '+ _buildGroupSegmentedSwitcher();'},
      {'type': 'normal', 'text': '  Widget build(BuildContext context) {'},
      {'type': 'normal', 'text': '    return Container('},
    ];

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: const Color(0xFF0F172A),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: diffLines.map((l) {
          Color bg = Colors.transparent;
          Color text = const Color(0xFFE2E8F0);
          if (l['type'] == 'add') {
            bg = const Color(0xFF059669).withOpacity(0.2);
            text = const Color(0xFF34D399);
          } else if (l['type'] == 'del') {
            bg = const Color(0xFFDC2626).withOpacity(0.2);
            text = const Color(0xFFF87171);
          } else if (l['type'] == 'header') {
            text = const Color(0xFF38BDF8);
          }

          return Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            color: bg,
            child: Text(
              l['text']!,
              style: TextStyle(
                fontSize: 11,
                fontFamily: 'Consolas',
                color: text,
              ),
            ),
          );
        }).toList(),
      ),
    );
  }

  // ==========================================
  // TAB 4: SIDE CHAT (/side, /btw)
  // ==========================================
  Widget _buildSideChatTab() {
    return Column(
      children: [
        // Side Chat Banner
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
          ),
          child: Row(
            children: [
              const Icon(FontAwesomeIcons.comments, size: 12, color: DesktopTheme.accentSky),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  'Side Channel (/side) · Isolated thread',
                  style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
        ),

        // Messages list
        Expanded(
          child: ListView.builder(
            padding: const EdgeInsets.all(12),
            itemCount: sideMessages.length,
            itemBuilder: (context, index) {
              final msg = sideMessages[index];
              final isUser = msg['role'] == 'user';

              return Align(
                alignment: isUser ? Alignment.centerRight : Alignment.centerLeft,
                child: Container(
                  margin: const EdgeInsets.only(bottom: 8),
                  padding: const EdgeInsets.all(10),
                  constraints: const BoxConstraints(maxWidth: 340),
                  decoration: BoxDecoration(
                    color: isUser ? DesktopTheme.accentSky.withOpacity(0.18) : DesktopTheme.bgSurface,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(
                      color: isUser ? DesktopTheme.accentSky.withOpacity(0.4) : DesktopTheme.borderSubtle,
                    ),
                  ),
                  child: Text(
                    msg['text']!,
                    style: TextStyle(fontSize: 12, color: DesktopTheme.textPrimary, height: 1.3),
                  ),
                ),
              );
            },
          ),
        ),

        // Input field
        Container(
          padding: const EdgeInsets.all(8),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurface,
            border: Border(top: BorderSide(color: DesktopTheme.borderSubtle)),
          ),
          child: Row(
            children: [
              Expanded(
                child: Container(
                  height: 32,
                  padding: const EdgeInsets.symmetric(horizontal: 10),
                  decoration: BoxDecoration(
                    color: DesktopTheme.bgCanvas,
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: DesktopTheme.borderSubtle),
                  ),
                  child: TextField(
                    controller: sideChatController,
                    style: TextStyle(fontSize: 12, color: DesktopTheme.textPrimary),
                    decoration: InputDecoration(
                      hintText: 'Ask side question (/side, /btw)...',
                      hintStyle: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                      border: InputBorder.none,
                      isDense: true,
                      contentPadding: const EdgeInsets.only(top: 8),
                    ),
                    onSubmitted: (txt) => _sendSideMessage(),
                  ),
                ),
              ),
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(FontAwesomeIcons.paperPlane, size: 13, color: DesktopTheme.accentSky),
                onPressed: _sendSideMessage,
              ),
            ],
          ),
        ),
      ],
    );
  }

  void _sendSideMessage() {
    final txt = sideChatController.text.trim();
    if (txt.isEmpty) return;

    setState(() {
      sideMessages.add({'role': 'user', 'text': txt});
      sideChatController.clear();
    });

    Future.delayed(const Duration(milliseconds: 600), () {
      if (!mounted) return;
      setState(() {
        sideMessages.add({
          'role': 'bot',
          'text': 'Понял вопрос по "$txt". Контекст сохранен в боковом канале без изменения основной цели.',
        });
      });
    });
  }
}
