import 'dart:async';
import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';
import 'package:webview_windows/webview_windows.dart';
import '../../theme/desktop_theme.dart';
import '../terminal/desktop_terminal_service.dart';
import '../workspace/task_workspace_controller.dart';
import 'browser/desktop_webview_controller.dart';
import 'artifacts/artifacts_viewer_controller.dart';
import 'artifacts/diff_viewer_widget.dart';
import 'side_chat/side_chat_controller.dart';

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

  // Real Terminal Service & Live Canvas State
  late final DesktopTerminalService terminalService;
  final GatewayHttpClient httpClient = GatewayHttpClient();
  CanvasWsClient? canvasWsClient;
  CanvasFrame? currentCanvasFrame;
  String activeCanvasId = 'main';
  final List<String> availableCanvases = ['main'];
  StreamSubscription? _canvasSub;
  bool isCanvasLoading = false;

  // Live Browser State & WebView2 Controller
  final browserUrlController = TextEditingController(text: 'http://localhost:3000');
  late final DesktopWebviewController webviewController;
  int browserViewportMode = 0; // 0: Desktop, 1: Tablet, 2: Mobile
  bool isElementPickerActive = false;
  String? hoveredElementSelector;
  String? selectedElementSelector;
  String? selectedElementTag;
  String? selectedElementText;

  // Artifacts & Diff Controller
  late final ArtifactsViewerController artifactsController;

  // Terminal State
  final terminalInputController = TextEditingController();

  // Preview State
  int previewModeIndex = 0; // 0: Markdown Report, 1: Architecture Mermaid, 2: Git Diff

  // Side Chat State & Logic Controller
  final sideChatController = TextEditingController();
  late final SideChatController sideChatLogicController;

  bool showTabChooser = false;

  @override
  void initState() {
    super.initState();
    terminalService = DesktopTerminalService.to;
    _initCanvas();

    // Initialize WebView2 Controller
    webviewController = DesktopWebviewController();
    webviewController.onElementPicked = (selector, tag, text) {
      widget.controller.addDomSelectorChip(selector);
      setState(() {
        selectedElementSelector = selector;
        selectedElementTag = tag;
        selectedElementText = text;
      });
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Селектор "$selector" добавлен в контекст Composer'),
            duration: const Duration(seconds: 2),
          ),
        );
      }
    };
    webviewController.initialize(initialUrl: browserUrlController.text);

    // Initialize Artifacts & Side Chat
    artifactsController = ArtifactsViewerController(httpClient: httpClient);
    sideChatLogicController = SideChatController(httpClient: httpClient);

    final effectiveIndex = widget.initialTabIndex < 0 ? 0 : widget.initialTabIndex.clamp(0, 4);
    showTabChooser = widget.initialTabIndex < 0;
    tabController = TabController(
      length: 5,
      vsync: this,
      initialIndex: effectiveIndex,
    );
  }

  Future<void> _initCanvas() async {
    try {
      final list = await httpClient.getCanvasList();
      if (list.isNotEmpty && mounted) {
        setState(() {
          availableCanvases.addAll(
            list.map((e) => e['canvas_id']?.toString() ?? e['id']?.toString() ?? e['name']?.toString() ?? 'main'),
          );
          if (!availableCanvases.contains(activeCanvasId)) {
            activeCanvasId = availableCanvases.first;
          }
        });
      }
    } catch (_) {}
    _connectCanvasWs();
  }

  void _connectCanvasWs() {
    _canvasSub?.cancel();
    canvasWsClient?.disconnect();
    canvasWsClient = CanvasWsClient(canvasId: activeCanvasId);
    _canvasSub = canvasWsClient!.stream.listen((frame) {
      if (mounted) {
        setState(() {
          currentCanvasFrame = frame;
        });
      }
    });
    canvasWsClient!.connect();
  }

  @override
  void dispose() {
    tabController.dispose();
    browserUrlController.dispose();
    terminalInputController.dispose();
    sideChatController.dispose();
    webviewController.dispose();
    artifactsController.dispose();
    sideChatLogicController.dispose();
    _canvasSub?.cancel();
    canvasWsClient?.dispose();
    httpClient.dispose();
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
          // Inspector Header with 4 ADE Tabs and Close button
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
                        icon: Icon(FontAwesomeIcons.wandMagicSparkles, size: 13),
                        text: 'Canvas',
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
                IconButton(
                  icon: const Icon(Icons.dashboard_customize_outlined, size: 15, color: Color(0xFF94A3B8)),
                  tooltip: 'Выбор вкладки (Open tab)',
                  onPressed: () => setState(() => showTabChooser = true),
                ),
                if (widget.onClose != null)
                  IconButton(
                    icon: const Icon(Icons.close, size: 16, color: Color(0xFF94A3B8)),
                    tooltip: 'Закрыть панель',
                    onPressed: widget.onClose,
                  ),
              ],
            ),
          ),

          // Tab Views or Open Tab Chooser
          Expanded(
            child: showTabChooser
                ? _buildOpenTabChooser()
                : TabBarView(
                    controller: tabController,
                    children: [
                      _buildLiveBrowserTab(),
                      _buildTerminalTab(),
                      _buildCanvasTab(),
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
  // OPEN TAB CHOOSER SCREEN (Screenshot 2 Match)
  // ==========================================
  Widget _buildOpenTabChooser() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 36),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Text(
              'Open tab',
              style: TextStyle(
                fontSize: 22,
                fontWeight: FontWeight.bold,
                color: Colors.white,
              ),
            ),
            const SizedBox(height: 8),
            const Text(
              'Choose a tab to open in the side pane.',
              style: TextStyle(
                fontSize: 13,
                color: Color(0xFF94A3B8),
              ),
            ),
            const SizedBox(height: 32),
            _buildChooserCard(
              icon: Icons.chat_bubble_outline,
              label: 'Side conversation',
              onTap: () {
                setState(() {
                  showTabChooser = false;
                  tabController.animateTo(4); // Side Chat
                });
              },
            ),
            const SizedBox(height: 12),
            _buildChooserCard(
              icon: Icons.assignment_outlined,
              label: 'Review & Diff',
              onTap: () {
                setState(() {
                  showTabChooser = false;
                  tabController.animateTo(3); // Preview / Review
                });
              },
            ),
            const SizedBox(height: 12),
            _buildChooserCard(
              icon: FontAwesomeIcons.wandMagicSparkles,
              label: 'Live Canvas (A2UI)',
              onTap: () {
                setState(() {
                  showTabChooser = false;
                  tabController.animateTo(2); // Canvas
                });
              },
            ),
            const SizedBox(height: 12),
            _buildChooserCard(
              icon: FontAwesomeIcons.terminal,
              label: 'Terminal',
              onTap: () {
                setState(() {
                  showTabChooser = false;
                  tabController.animateTo(1); // Terminal
                });
              },
            ),
            const SizedBox(height: 12),
            _buildChooserCard(
              icon: FontAwesomeIcons.globe,
              label: 'Browser',
              onTap: () {
                setState(() {
                  showTabChooser = false;
                  tabController.animateTo(0); // Browser
                });
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildChooserCard({
    required IconData icon,
    required String label,
    required VoidCallback onTap,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(8),
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
        decoration: BoxDecoration(
          color: const Color(0xFF1E222A),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: const Color(0xFF2B3240)),
        ),
        child: Row(
          children: [
            Icon(icon, size: 16, color: const Color(0xFF94A3B8)),
            const SizedBox(width: 14),
            Text(
              label,
              style: const TextStyle(
                fontSize: 14,
                fontWeight: FontWeight.w600,
                color: Colors.white,
              ),
            ),
          ],
        ),
      ),
    );
  }

  // ==========================================
  // TAB 1: LIVE BROWSER & ELEMENT PICKER
  // ==========================================
  Widget _buildLiveBrowserTab() {
    return AnimatedBuilder(
      animation: webviewController,
      builder: (context, _) {
        final isInitialized = webviewController.isInitialized;
        final isPickerActive = webviewController.isPickerActive;
        final selected = selectedElementSelector ?? webviewController.selectedSelector;

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
                      InkWell(
                        onTap: isInitialized ? () => webviewController.reload() : null,
                        child: Icon(Icons.arrow_back, size: 15, color: DesktopTheme.textMuted),
                      ),
                      const SizedBox(width: 6),
                      Icon(Icons.arrow_forward, size: 15, color: DesktopTheme.textMuted.withOpacity(0.4)),
                      const SizedBox(width: 8),
                      InkWell(
                        onTap: () {
                          if (isInitialized) {
                            webviewController.reload();
                          } else {
                            webviewController.initialize(initialUrl: browserUrlController.text);
                          }
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
                              Icon(
                                isInitialized ? Icons.lock_outline : Icons.cloud_queue,
                                size: 12,
                                color: isInitialized ? DesktopTheme.statusSuccess : DesktopTheme.textMuted,
                              ),
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
                                  onSubmitted: (url) {
                                    if (isInitialized) {
                                      webviewController.loadUrl(url);
                                    } else {
                                      webviewController.initialize(initialUrl: url);
                                    }
                                  },
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
                          if (isInitialized) {
                            webviewController.toggleElementPicker();
                          } else {
                            setState(() {
                              isElementPickerActive = !isElementPickerActive;
                              if (!isElementPickerActive) {
                                hoveredElementSelector = null;
                              }
                            });
                          }
                        },
                        borderRadius: BorderRadius.circular(5),
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                          decoration: BoxDecoration(
                            color: (isPickerActive || isElementPickerActive)
                                ? DesktopTheme.accentSky.withOpacity(0.18)
                                : DesktopTheme.bgCanvas,
                            borderRadius: BorderRadius.circular(5),
                            border: Border.all(
                              color: (isPickerActive || isElementPickerActive)
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
                                color: (isPickerActive || isElementPickerActive)
                                    ? DesktopTheme.accentSky
                                    : DesktopTheme.textMuted,
                              ),
                              const SizedBox(width: 6),
                              Text(
                                (isPickerActive || isElementPickerActive)
                                    ? 'Element Picker Active'
                                    : 'Pick Element',
                                style: TextStyle(
                                  fontSize: 11,
                                  fontWeight: FontWeight.bold,
                                  color: (isPickerActive || isElementPickerActive)
                                      ? DesktopTheme.accentSky
                                      : DesktopTheme.textSecondary,
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      if (webviewController.isLoading)
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: DesktopTheme.accentSky.withOpacity(0.1),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: const Text(
                            'LOADING...',
                            style: TextStyle(
                              fontSize: 9,
                              fontFamily: 'Consolas',
                              fontWeight: FontWeight.bold,
                              color: DesktopTheme.accentSky,
                            ),
                          ),
                        ),
                      const Spacer(),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: isInitialized
                              ? DesktopTheme.statusSuccess.withOpacity(0.1)
                              : DesktopTheme.statusWarning.withOpacity(0.1),
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Text(
                          isInitialized ? 'EDGE WEBVIEW2 · 60 FPS' : 'STANDBY MODE',
                          style: TextStyle(
                            fontSize: 9,
                            fontFamily: 'Consolas',
                            fontWeight: FontWeight.bold,
                            color: isInitialized
                                ? DesktopTheme.statusSuccess
                                : DesktopTheme.statusWarning,
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
                      child: isInitialized
                          ? Webview(webviewController.rawController)
                          : _buildBrowserStandbyView(),
                    ),
                  ),
                ),
              ),
            ),

            // Picked Element Details & Inject Button
            if (selected != null)
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
                                '<${selectedElementTag ?? 'el'}>',
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
                            selected,
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
                          selector: selected,
                          text: selectedElementText ?? '',
                          tag: selectedElementTag ?? 'div',
                        );
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(
                            content: Text('Селектор "$selected" добавлен в контекст задачи'),
                            duration: const Duration(seconds: 2),
                          ),
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
      },
    );
  }

  Widget _buildBrowserStandbyView() {
    return Container(
      color: const Color(0xFF0F172A),
      padding: const EdgeInsets.all(24),
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(FontAwesomeIcons.globe, size: 36, color: Color(0xFF00D2FF)),
            const SizedBox(height: 16),
            const Text(
              'Microsoft Edge WebView2',
              style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: Colors.white),
            ),
            const SizedBox(height: 8),
            Text(
              'Встроенный браузер для живого предпросмотра веб-приложений и захвата DOM-элементов.',
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: 12, color: DesktopTheme.textMuted),
            ),
            const SizedBox(height: 20),
            ElevatedButton.icon(
              onPressed: () {
                webviewController.initialize(initialUrl: browserUrlController.text);
              },
              icon: const Icon(Icons.play_arrow, size: 16),
              label: const Text('Запустить WebView2'),
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF00D2FF),
                foregroundColor: const Color(0xFF090D12),
              ),
            ),
          ],
        ),
      ),
    );
  }



  // ==========================================
  // TAB 2: TERMINAL & GIT (Real Process Terminal)
  // ==========================================
  Widget _buildTerminalTab() {
    return Container(
      color: const Color(0xFF0F172A),
      child: Obx(() {
        final sessions = terminalService.sessions;
        final activeIdx = terminalService.activeSessionIndex.value;
        final activeSession = terminalService.activeSession;

        return Column(
          children: [
            // Terminal Session Tabs Strip
            Container(
              height: 36,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: const BoxDecoration(
                color: Color(0xFF0B101E),
                border: Border(bottom: BorderSide(color: Color(0xFF1E293B))),
              ),
              child: Row(
                children: [
                  for (int i = 0; i < sessions.length; i++)
                    InkWell(
                      onTap: () => terminalService.activeSessionIndex.value = i,
                      borderRadius: BorderRadius.circular(4),
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                        margin: const EdgeInsets.only(right: 6),
                        decoration: BoxDecoration(
                          color: activeIdx == i ? const Color(0xFF1E293B) : Colors.transparent,
                          borderRadius: BorderRadius.circular(4),
                          border: Border.all(
                            color: activeIdx == i ? const Color(0xFF38BDF8) : Colors.transparent,
                            width: 0.8,
                          ),
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Text(
                              sessions[i].title,
                              style: TextStyle(
                                fontSize: 10,
                                fontFamily: 'Consolas',
                                fontWeight: activeIdx == i ? FontWeight.bold : FontWeight.normal,
                                color: activeIdx == i ? const Color(0xFF38BDF8) : const Color(0xFF94A3B8),
                              ),
                            ),
                            if (sessions.length > 1) ...[
                              const SizedBox(width: 4),
                              InkWell(
                                onTap: () => terminalService.closeSession(i),
                                child: const Icon(Icons.close, size: 10, color: Color(0xFF64748B)),
                              ),
                            ],
                          ],
                        ),
                      ),
                    ),
                  IconButton(
                    tooltip: 'Новая консоль (powershell)',
                    icon: const Icon(Icons.add, size: 14, color: Color(0xFF38BDF8)),
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
                    onPressed: () => terminalService.startNewSession(),
                  ),
                  const Spacer(),
                  IconButton(
                    tooltip: 'Прервать (Ctrl+C)',
                    icon: const Icon(Icons.stop_circle_outlined, size: 13, color: Color(0xFFF87171)),
                    onPressed: () => terminalService.interruptActiveSession(),
                  ),
                  IconButton(
                    tooltip: 'Очистить вывод',
                    icon: const Icon(FontAwesomeIcons.trashCan, size: 11, color: Color(0xFF64748B)),
                    onPressed: () => terminalService.clearActiveSession(),
                  ),
                ],
              ),
            ),

            // Terminal Output Log
            Expanded(
              child: activeSession == null
                  ? const Center(child: Text('Нет активных сессий консоли', style: TextStyle(color: Color(0xFF64748B))))
                  : ListView.builder(
                      padding: const EdgeInsets.all(12),
                      itemCount: activeSession.lines.length,
                      itemBuilder: (context, index) {
                        final line = activeSession.lines[index];
                        Color lineColor = const Color(0xFFE2E8F0);
                        FontWeight fontWeight = FontWeight.normal;

                        if (line.startsWith('>') || line.startsWith('\$')) {
                          lineColor = const Color(0xFF38BDF8);
                          fontWeight = FontWeight.bold;
                        } else if (line.contains('ok') || line.contains('No issues found') || line.contains('passed')) {
                          lineColor = const Color(0xFF34D399);
                        } else if (line.contains('error') || line.contains('failed') || line.contains('[STDERR]')) {
                          lineColor = const Color(0xFFF87171);
                        } else if (line.contains('Compiling') || line.contains('Analyzing')) {
                          lineColor = const Color(0xFFFBBF24);
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

            // Quick command chips
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
              decoration: const BoxDecoration(
                color: Color(0xFF0B101E),
                border: Border(top: BorderSide(color: Color(0xFF1E293B))),
              ),
              child: Row(
                children: [
                  _buildQuickCommandChip('cargo test', () => terminalService.sendCommand('cargo test')),
                  const SizedBox(width: 6),
                  _buildQuickCommandChip('flutter analyze', () => terminalService.sendCommand('flutter analyze')),
                  const SizedBox(width: 6),
                  _buildQuickCommandChip('git status', () => terminalService.sendCommand('git status')),
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
                    '> ',
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
                        hintText: 'Введите команду shell (powershell / cmd)...',
                        hintStyle: TextStyle(fontSize: 11, color: Color(0xFF64748B)),
                        isDense: true,
                        border: InputBorder.none,
                        contentPadding: EdgeInsets.zero,
                      ),
                      onSubmitted: (cmd) {
                        if (cmd.trim().isEmpty) return;
                        terminalService.sendCommand(cmd.trim());
                        terminalInputController.clear();
                      },
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.send, size: 14, color: Color(0xFF38BDF8)),
                    tooltip: 'Отправить',
                    onPressed: () {
                      final cmd = terminalInputController.text.trim();
                      if (cmd.isNotEmpty) {
                        terminalService.sendCommand(cmd);
                        terminalInputController.clear();
                      }
                    },
                  ),
                ],
              ),
            ),
          ],
        );
      }),
    );
  }

  // ==========================================
  // TAB 3: LIVE CANVAS (A2UI & Artifacts)
  // ==========================================
  Widget _buildCanvasTab() {
    final frame = currentCanvasFrame;
    final isWsConnected = canvasWsClient?.isConnected == true;

    return Container(
      color: DesktopTheme.bgCanvas,
      child: Column(
        children: [
          // Canvas Top Bar
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurface,
              border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
            ),
            child: Row(
              children: [
                Container(
                  width: 8,
                  height: 8,
                  decoration: BoxDecoration(
                    color: isWsConnected ? const Color(0xFF10B981) : const Color(0xFFF59E0B),
                    shape: BoxShape.circle,
                  ),
                ),
                const SizedBox(width: 8),
                Text(
                  'Canvas: $activeCanvasId',
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'Consolas',
                    color: DesktopTheme.textPrimary,
                  ),
                ),
                if (frame != null) ...[
                  const SizedBox(width: 8),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                    decoration: BoxDecoration(
                      color: DesktopTheme.accentCyan.withOpacity(0.15),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      '${frame.contentType} v${frame.version}',
                      style: const TextStyle(fontSize: 10, fontFamily: 'Consolas', color: DesktopTheme.accentCyan),
                    ),
                  ),
                ],
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.refresh, size: 14),
                  color: DesktopTheme.textMuted,
                  tooltip: 'Обновить холст',
                  onPressed: () async {
                    final res = await httpClient.getCanvas(activeCanvasId);
                    if (res != null && res['frame'] != null && mounted) {
                      setState(() {
                        currentCanvasFrame = CanvasFrame.fromJson(res['frame'] as Map<String, dynamic>);
                      });
                    }
                  },
                ),
                IconButton(
                  icon: const Icon(FontAwesomeIcons.trashCan, size: 12),
                  color: DesktopTheme.textMuted,
                  tooltip: 'Очистить холст',
                  onPressed: () async {
                    await httpClient.clearCanvas(activeCanvasId);
                    if (mounted) setState(() => currentCanvasFrame = null);
                  },
                ),
              ],
            ),
          ),

          // Canvas Content View
          Expanded(
            child: frame == null
                ? Center(
                    child: Padding(
                      padding: const EdgeInsets.all(24),
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(FontAwesomeIcons.wandMagicSparkles, size: 36, color: DesktopTheme.accentCyan.withOpacity(0.6)),
                          const SizedBox(height: 14),
                          Text(
                            'Live Canvas пуст',
                            style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
                          ),
                          const SizedBox(height: 6),
                          Text(
                            'Агент может транслировать сюда интерактивные A2UI формы, графики, HTML и диаграммы через /ws/canvas.',
                            textAlign: TextAlign.center,
                            style: TextStyle(fontSize: 12, color: DesktopTheme.textMuted),
                          ),
                          const SizedBox(height: 20),
                          Wrap(
                            spacing: 8,
                            runSpacing: 8,
                            children: [
                              OutlinedButton.icon(
                                icon: const Icon(FontAwesomeIcons.code, size: 11),
                                label: const Text('Демо HTML формы', style: TextStyle(fontSize: 11)),
                                onPressed: () async {
                                  await httpClient.postCanvas(activeCanvasId, {
                                    'content_type': 'html',
                                    'content': '<form class="omnes-a2ui"><label>Параметры деплоя:</label><input type="text" value="v1.0.0-rc2" /><button>Подтвердить</button></form>',
                                  });
                                },
                              ),
                              OutlinedButton.icon(
                                icon: const Icon(FontAwesomeIcons.diagramProject, size: 11),
                                label: const Text('Демо Mermaid схемы', style: TextStyle(fontSize: 11)),
                                onPressed: () async {
                                  await httpClient.postCanvas(activeCanvasId, {
                                    'content_type': 'markdown',
                                    'content': '```mermaid\ngraph LR\nClient[Omnes Desktop ADE] -->|WS/chat| Gateway\nGateway --> Runtime\nGateway --> Canvas\n```',
                                  });
                                },
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  )
                : SingleChildScrollView(
                    padding: const EdgeInsets.all(16),
                    child: Container(
                      width: double.infinity,
                      padding: const EdgeInsets.all(16),
                      decoration: BoxDecoration(
                        color: DesktopTheme.bgSurfaceElevated,
                        borderRadius: BorderRadius.circular(10),
                        border: Border.all(color: DesktopTheme.borderSubtle),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Icon(
                                frame.contentType == 'html' ? FontAwesomeIcons.code : FontAwesomeIcons.diagramProject,
                                size: 14,
                                color: DesktopTheme.accentCyan,
                              ),
                              const SizedBox(width: 8),
                              Text(
                                'A2UI Live Artifact (${frame.contentType})',
                                style: TextStyle(
                                  fontSize: 13,
                                  fontWeight: FontWeight.bold,
                                  color: DesktopTheme.textPrimary,
                                ),
                              ),
                            ],
                          ),
                          const Divider(height: 20),
                          SelectableText(
                            frame.content,
                            style: TextStyle(
                              fontSize: 12,
                              fontFamily: 'Consolas',
                              color: DesktopTheme.textPrimary,
                              height: 1.45,
                            ),
                          ),
                          const SizedBox(height: 16),
                          Row(
                            children: [
                              ElevatedButton.icon(
                                icon: const Icon(Icons.touch_app, size: 13),
                                label: const Text('Отправить действие (Action callback)', style: TextStyle(fontSize: 11)),
                                style: ElevatedButton.styleFrom(
                                  backgroundColor: DesktopTheme.accentCyan,
                                  foregroundColor: Colors.black,
                                ),
                                onPressed: () {
                                  canvasWsClient?.sendAction('submit', {'canvas_id': activeCanvasId});
                                  Get.snackbar('Canvas Action', 'Действие отправлено в шлюз через WS');
                                },
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
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
  // TAB 3: ARTIFACTS & GIT DIFF VIEWER
  // ==========================================
  Widget _buildPreviewTab() {
    return DiffViewerWidget(controller: artifactsController);
  }

  // ==========================================
  // TAB 4: SIDE CHAT (/side, /btw)
  // ==========================================
  Widget _buildSideChatTab() {
    return AnimatedBuilder(
      animation: sideChatLogicController,
      builder: (context, _) {
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
                      'Side Channel (/side, /btw) · Isolated thread',
                      style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  InkWell(
                    onTap: sideChatLogicController.clearChat,
                    borderRadius: BorderRadius.circular(4),
                    child: Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      child: Text(
                        'Очистить',
                        style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted),
                      ),
                    ),
                  ),
                ],
              ),
            ),

            // Messages list
            Expanded(
              child: ListView.builder(
                padding: const EdgeInsets.all(12),
                itemCount: sideChatLogicController.messages.length,
                itemBuilder: (context, index) {
                  final msg = sideChatLogicController.messages[index];
                  final isUser = msg.isUser;

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
                      child: SelectableText(
                        msg.text,
                        style: TextStyle(fontSize: 12, color: DesktopTheme.textPrimary, height: 1.35),
                      ),
                    ),
                  );
                },
              ),
            ),

            if (sideChatLogicController.isLoading)
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
                child: Row(
                  children: [
                    const SizedBox(
                      width: 12,
                      height: 12,
                      child: CircularProgressIndicator(strokeWidth: 2, color: DesktopTheme.accentSky),
                    ),
                    const SizedBox(width: 8),
                    Text(
                      'Шлюз формирует ответ...',
                      style: TextStyle(fontSize: 10, fontFamily: 'Consolas', color: DesktopTheme.textMuted),
                    ),
                  ],
                ),
              ),

            // Quick prompts strip
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              color: DesktopTheme.bgSurface,
              child: SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: Row(
                  children: [
                    _buildSidePromptChip('/btw '),
                    _buildSidePromptChip('/explain '),
                    _buildSidePromptChip('/test '),
                    _buildSidePromptChip('/refactor '),
                  ],
                ),
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
                          hintText: 'Задайте вопрос (/side, /btw)...',
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
      },
    );
  }

  Widget _buildSidePromptChip(String prompt) {
    return Padding(
      padding: const EdgeInsets.only(right: 6),
      child: InkWell(
        onTap: () {
          sideChatController.text = prompt;
          sideChatController.selection = TextSelection.fromPosition(TextPosition(offset: prompt.length));
        },
        borderRadius: BorderRadius.circular(4),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
          decoration: BoxDecoration(
            color: DesktopTheme.bgCanvas,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(color: DesktopTheme.borderSubtle),
          ),
          child: Text(
            prompt.trim(),
            style: const TextStyle(fontSize: 10, fontFamily: 'Consolas', color: DesktopTheme.accentSky),
          ),
        ),
      ),
    );
  }

  void _sendSideMessage() {
    final txt = sideChatController.text.trim();
    if (txt.isEmpty) return;
    sideChatLogicController.sendMessage(txt);
    sideChatController.clear();
  }
}
