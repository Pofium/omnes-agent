import 'package:flutter/services.dart';
import 'dart:async';
import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';
import 'package:omnes_shared/omnes_shared.dart';
import '../../theme/desktop_theme.dart';
import '../terminal/web_terminal_service.dart';
import '../workspace/task_workspace_controller.dart';
import 'browser/web_browser_controller.dart';
import 'browser/web_iframe_view.dart';
import 'artifacts/artifacts_viewer_controller.dart';
import 'artifacts/diff_viewer_widget.dart';
import 'side_chat/side_chat_controller.dart';

class DesktopInspectorPanel extends StatefulWidget {
  final double width;
  final DesktopTaskWorkspaceController controller;
  final int initialTabIndex;
  final String? initialSideChatText;
  final VoidCallback? onClose;

  const DesktopInspectorPanel({
    super.key,
    this.width = 420.0,
    required this.controller,
    this.initialTabIndex = 0,
    this.initialSideChatText,
    this.onClose,
  });

  @override
  State<DesktopInspectorPanel> createState() => _DesktopInspectorPanelState();
}

class _DesktopInspectorPanelState extends State<DesktopInspectorPanel>
    with SingleTickerProviderStateMixin {
  late TabController tabController;

  // Real Terminal Service & Live Canvas State
  late final WebTerminalService terminalService;
  final GatewayHttpClient httpClient = GatewayHttpClient();
  CanvasWsClient? canvasWsClient;
  CanvasFrame? currentCanvasFrame;
  String activeCanvasId = 'main';
  final List<String> availableCanvases = ['main'];
  StreamSubscription? _canvasSub;
  bool isCanvasLoading = false;

  // Live Browser State & WebBrowserController
  final browserUrlController = TextEditingController(text: 'http://localhost:3000');
  late final WebBrowserController webviewController;
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

  bool showTabChooser = true;
  final List<String> openTabKeys = [];
  String activeTabKey = '';
  bool isCanvasRawMode = false;

  @override
  void initState() {
    super.initState();
    terminalService = WebTerminalService.to;
    _initCanvas();

    // Initialize Web Browser Controller
    webviewController = WebBrowserController();
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
    if (widget.initialSideChatText != null && widget.initialSideChatText!.trim().isNotEmpty) {
      sideChatLogicController.addBranchContext(widget.initialSideChatText!);
    }

    if (widget.initialTabIndex == 5) {
      if (!openTabKeys.contains('file')) openTabKeys.add('file');
      activeTabKey = 'file';
      showTabChooser = false;
    } else if (widget.initialTabIndex >= 0 && widget.initialTabIndex < 5) {
      final defaultKeys = ['browser', 'terminal', 'canvas', 'preview', 'side_chat'];
      final targetKey = defaultKeys[widget.initialTabIndex];
      if (!openTabKeys.contains(targetKey)) openTabKeys.add(targetKey);
      activeTabKey = targetKey;
      showTabChooser = false;
    } else {
      showTabChooser = true;
    }
    tabController = TabController(
      length: 5,
      vsync: this,
      initialIndex: 0,
    );
  }

  @override
  void didUpdateWidget(DesktopInspectorPanel oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.initialTabIndex != oldWidget.initialTabIndex) {
      if (widget.initialTabIndex == 5) {
        if (!openTabKeys.contains('file')) openTabKeys.add('file');
        activeTabKey = 'file';
        showTabChooser = false;
      } else if (widget.initialTabIndex >= 0 && widget.initialTabIndex < 5) {
        final defaultKeys = ['browser', 'terminal', 'canvas', 'preview', 'side_chat'];
        final targetKey = defaultKeys[widget.initialTabIndex];
        if (!openTabKeys.contains(targetKey)) openTabKeys.add(targetKey);
        activeTabKey = targetKey;
        showTabChooser = false;
      }
      setState(() {});
    }
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
      width: widget.width,
      decoration: BoxDecoration(
        color: DesktopTheme.bgSidebar,
        border: Border(
          left: BorderSide(color: DesktopTheme.borderSubtle, width: 1),
        ),
      ),
      child: Column(
        children: [
          // Inspector Header with dynamic lazy tabs and close [x] buttons
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
                  child: SingleChildScrollView(
                    scrollDirection: Axis.horizontal,
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const SizedBox(width: 8),
                        for (final tabKey in openTabKeys)
                          _buildInspectorTabHeader(tabKey),
                      ],
                    ),
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
            child: (showTabChooser || openTabKeys.isEmpty)
                ? _buildOpenTabChooser()
                : _buildActiveTabContent(),
          ),
        ],
      ),
    );
  }

  Widget _buildActiveTabContent() {
    switch (activeTabKey) {
      case 'browser':
        return _buildLiveBrowserTab();
      case 'terminal':
        return _buildTerminalTab();
      case 'canvas':
        return _buildCanvasTab();
      case 'preview':
        return _buildPreviewTab();
      case 'side_chat':
        return _buildSideChatTab();
      case 'file':
        return _buildFileViewerTab();
      default:
        return _buildOpenTabChooser();
    }
  }

  Widget _buildInspectorTabHeader(String tabKey) {
    final isSelected = activeTabKey == tabKey && !showTabChooser;
    final (label, icon) = _getTabMeta(tabKey);

    return InkWell(
      onTap: () {
        setState(() {
          activeTabKey = tabKey;
          showTabChooser = false;
        });
      },
      borderRadius: BorderRadius.circular(6),
      child: Container(
        height: 32,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        margin: const EdgeInsets.only(right: 4),
        decoration: BoxDecoration(
          color: isSelected ? DesktopTheme.bgSurfaceElevated : Colors.transparent,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            color: isSelected ? DesktopTheme.borderSubtle : Colors.transparent,
            width: 0.8,
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              icon,
              size: 12,
              color: isSelected ? DesktopTheme.accentSky : DesktopTheme.textMuted,
            ),
            const SizedBox(width: 6),
            Text(
              label,
              style: TextStyle(
                fontSize: 11,
                fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                color: isSelected ? DesktopTheme.textPrimary : DesktopTheme.textMuted,
              ),
            ),
            const SizedBox(width: 6),
            InkWell(
              onTap: () {
                setState(() {
                  openTabKeys.remove(tabKey);
                  if (activeTabKey == tabKey) {
                    if (openTabKeys.isNotEmpty) {
                      activeTabKey = openTabKeys.last;
                    } else {
                      showTabChooser = true;
                    }
                  }
                });
              },
              borderRadius: BorderRadius.circular(3),
              child: Padding(
                padding: const EdgeInsets.all(2),
                child: Icon(
                  Icons.close,
                  size: 11,
                  color: isSelected ? DesktopTheme.textSecondary : DesktopTheme.textMuted,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  (String, IconData) _getTabMeta(String key) {
    switch (key) {
      case 'browser':
        return ('Браузер', FontAwesomeIcons.globe);
      case 'terminal':
        return ('Терминал', FontAwesomeIcons.terminal);
      case 'canvas':
        return ('Холст', FontAwesomeIcons.wandMagicSparkles);
      case 'preview':
        return ('Превью', FontAwesomeIcons.eye);
      case 'side_chat':
        return ('Боковой чат', FontAwesomeIcons.comments);
      case 'file':
        final path = widget.controller.selectedFilePath.value;
        final name = path != null ? path.split(RegExp(r'[\\/]')).last : 'Файл';
        return (name, Icons.code);
      default:
        return (key, Icons.tab);
    }
  }

  Widget _buildFileViewerTab() {
    return Obx(() {
      final path = widget.controller.selectedFilePath.value;
      final content = widget.controller.selectedFileContent.value;
      if (path == null || content == null) {
        return Center(
          child: Text(
            'Файл не выбран\nВыберите файл в дереве проекта',
            textAlign: TextAlign.center,
            style: TextStyle(color: DesktopTheme.textMuted, fontSize: 13),
          ),
        );
      }

      final lines = content.split('\n');

      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // File Breadcrumb Bar
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurface,
              border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 0.8)),
            ),
            child: Row(
              children: [
                Icon(Icons.insert_drive_file_outlined, size: 14, color: DesktopTheme.accentSky),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    path,
                    style: TextStyle(fontSize: 11, fontFamily: 'Consolas', color: DesktopTheme.textSecondary),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                Text(
                  '${lines.length} строк',
                  style: TextStyle(fontSize: 10, color: DesktopTheme.textMuted),
                ),
              ],
            ),
          ),

          // Code Viewer with line numbers
          Expanded(
            child: Container(
              color: DesktopTheme.bgCanvas,
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(12),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Line numbers
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.end,
                      children: [
                        for (int i = 1; i <= lines.length; i++)
                          Text(
                            '$i ',
                            style: const TextStyle(
                              fontFamily: 'Consolas',
                              fontSize: 11,
                              color: Color(0xFF64748B),
                              height: 1.4,
                            ),
                          ),
                      ],
                    ),
                    const SizedBox(width: 12),
                    Container(width: 1, height: lines.length * 15.4, color: DesktopTheme.borderSubtle),
                    const SizedBox(width: 12),
                    // Code content
                    Expanded(
                      child: SelectableText(
                        content,
                        style: TextStyle(
                          fontFamily: 'Consolas',
                          fontSize: 11,
                          color: DesktopTheme.textPrimary,
                          height: 1.4,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      );
    });
  }

  // ==========================================
  // OPEN TAB CHOOSER SCREEN (Screenshot 2 Match)
  // ==========================================
  Widget _buildOpenTabChooser() {
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 24),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Text(
              'Выбор вкладки',
              style: TextStyle(
                fontSize: 20,
                fontWeight: FontWeight.bold,
                color: Colors.white,
              ),
            ),
            const SizedBox(height: 6),
            const Text(
              'Выберите инструмент для открытия в правой панели:',
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 12,
                color: Color(0xFF94A3B8),
              ),
            ),
            const SizedBox(height: 24),
            _buildChooserCard(
              icon: Icons.chat_bubble_outline,
              label: 'Боковой чат',
              subtitle: 'Ветки диалога и контекстные уточнения',
              onTap: () {
                setState(() {
                  if (!openTabKeys.contains('side_chat')) openTabKeys.add('side_chat');
                  activeTabKey = 'side_chat';
                  showTabChooser = false;
                });
              },
            ),
            const SizedBox(height: 10),
            _buildChooserCard(
              icon: Icons.assignment_outlined,
              label: 'Превью и Диффы',
              subtitle: 'Просмотр артефактов и изменений кода',
              onTap: () {
                setState(() {
                  if (!openTabKeys.contains('preview')) openTabKeys.add('preview');
                  activeTabKey = 'preview';
                  showTabChooser = false;
                });
              },
            ),
            const SizedBox(height: 10),
            _buildChooserCard(
              icon: FontAwesomeIcons.wandMagicSparkles,
              label: 'Интерактивный Холст',
              subtitle: 'Рендеринг Markdown, HTML и интерфейсов',
              onTap: () {
                setState(() {
                  if (!openTabKeys.contains('canvas')) openTabKeys.add('canvas');
                  activeTabKey = 'canvas';
                  showTabChooser = false;
                });
              },
            ),
            const SizedBox(height: 10),
            _buildChooserCard(
              icon: FontAwesomeIcons.terminal,
              label: 'Терминал',
              subtitle: 'Встроенная системная консоль PowerShell',
              onTap: () {
                setState(() {
                  if (!openTabKeys.contains('terminal')) openTabKeys.add('terminal');
                  activeTabKey = 'terminal';
                  showTabChooser = false;
                });
              },
            ),
            const SizedBox(height: 10),
            _buildChooserCard(
              icon: FontAwesomeIcons.globe,
              label: 'Браузер',
              subtitle: 'Живой предпросмотр веб-страниц и инспекция DOM',
              onTap: () {
                setState(() {
                  if (!openTabKeys.contains('browser')) openTabKeys.add('browser');
                  activeTabKey = 'browser';
                  showTabChooser = false;
                });
              },
            ),
            if (widget.controller.selectedFilePath.value != null) ...[
              const SizedBox(height: 10),
              _buildChooserCard(
                icon: Icons.code,
                label: 'Файл: ${widget.controller.selectedFilePath.value!.split(RegExp(r"[\\/]")).last}',
                subtitle: widget.controller.selectedFilePath.value!,
                onTap: () {
                  setState(() {
                    if (!openTabKeys.contains('file')) openTabKeys.add('file');
                    activeTabKey = 'file';
                    showTabChooser = false;
                  });
                },
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildChooserCard({
    required IconData icon,
    required String label,
    String? subtitle,
    required VoidCallback onTap,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(8),
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 11),
        decoration: BoxDecoration(
          color: const Color(0xFF161A22),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: const Color(0xFF2B3240), width: 0.8),
        ),
        child: Row(
          children: [
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: const Color(0xFF0F131A),
                borderRadius: BorderRadius.circular(6),
              ),
              child: Icon(icon, size: 15, color: const Color(0xFF00D2FF)),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    label,
                    style: const TextStyle(
                      fontSize: 13,
                      fontWeight: FontWeight.w600,
                      color: Colors.white,
                    ),
                  ),
                  if (subtitle != null) ...[
                    const SizedBox(height: 2),
                    Text(
                      subtitle,
                      style: const TextStyle(
                        fontSize: 10.5,
                        color: Color(0xFF94A3B8),
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ],
              ),
            ),
            const Icon(Icons.chevron_right, size: 16, color: Color(0xFF64748B)),
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
                                    ? 'Выбор элемента активен'
                                    : 'Выбрать элемент',
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
                          isInitialized ? 'WEB BROWSER · IFRAME' : 'STANDBY MODE',
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
                          ? WebIframeView(url: webviewController.currentUrl)
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
                                'Выбранный элемент:',
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
                      label: const Text('Добавить в чат', style: TextStyle(fontSize: 11)),
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
  // TAB 3: LIVE CANVAS (Markdown & Artifacts)
  // ==========================================
  Widget _buildCanvasTab() {
    final frame = currentCanvasFrame;
    final isWsConnected = canvasWsClient?.isConnected == true;
    final selectedFile = widget.controller.selectedFilePath.value;
    final isMarkdownFile = selectedFile != null && selectedFile.toLowerCase().endsWith('.md');
    final selectedContent = widget.controller.selectedFileContent.value;

    final String? markdownToRender = isMarkdownFile && selectedContent != null && selectedContent.isNotEmpty
        ? selectedContent
        : (frame?.contentType == 'markdown' ? frame?.content : null);

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
                    color: isWsConnected ? const Color(0xFF10B981) : const Color(0xFF00D2FF),
                    shape: BoxShape.circle,
                  ),
                ),
                const SizedBox(width: 8),
                Text(
                  selectedFile != null
                      ? 'Холст: ${selectedFile.replaceAll(r'\', '/').split('/').last}'
                      : 'Холст: $activeCanvasId',
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'Consolas',
                    color: DesktopTheme.textPrimary,
                  ),
                ),
                const SizedBox(width: 8),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: DesktopTheme.accentCyan.withOpacity(0.15),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    isMarkdownFile ? 'Markdown документ' : '${frame?.contentType ?? "interactive"} v${frame?.version ?? 1}',
                    style: const TextStyle(fontSize: 10, fontFamily: 'Consolas', color: DesktopTheme.accentCyan),
                  ),
                ),
                const Spacer(),
                if (markdownToRender != null)
                  InkWell(
                    onTap: () => setState(() => isCanvasRawMode = !isCanvasRawMode),
                    borderRadius: BorderRadius.circular(4),
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                      margin: const EdgeInsets.only(right: 6),
                      decoration: BoxDecoration(
                        color: isCanvasRawMode ? DesktopTheme.accentCyan.withOpacity(0.2) : DesktopTheme.bgSurfaceElevated,
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(color: DesktopTheme.borderSubtle),
                      ),
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(isCanvasRawMode ? Icons.code : Icons.visibility, size: 11, color: DesktopTheme.accentCyan),
                          const SizedBox(width: 4),
                          Text(
                            isCanvasRawMode ? 'Исходник' : 'Предпросмотр',
                            style: TextStyle(fontSize: 10, fontFamily: 'Consolas', color: DesktopTheme.textPrimary),
                          ),
                        ],
                      ),
                    ),
                  ),
                IconButton(
                  icon: const Icon(Icons.refresh, size: 14),
                  color: DesktopTheme.textMuted,
                  tooltip: 'Обновить холст',
                  onPressed: () async {
                    if (isMarkdownFile) {
                      widget.controller.openProjectFile(selectedFile);
                    } else {
                      final res = await httpClient.getCanvas(activeCanvasId);
                      if (res != null && res['frame'] != null && mounted) {
                        setState(() {
                          currentCanvasFrame = CanvasFrame.fromJson(res['frame'] as Map<String, dynamic>);
                        });
                      }
                    }
                  },
                ),
                IconButton(
                  icon: const Icon(FontAwesomeIcons.trashCan, size: 12),
                  color: DesktopTheme.textMuted,
                  tooltip: 'Очистить холст',
                  onPressed: () async {
                    widget.controller.selectedFilePath.value = null;
                    widget.controller.selectedFileContent.value = null;
                    if (mounted) {
                      setState(() {
                        currentCanvasFrame = null;
                      });
                    }
                    try {
                      await httpClient.clearCanvas(activeCanvasId);
                    } catch (_) {}
                  },
                ),
              ],
            ),
          ),

          // Canvas Content View
          Expanded(
            child: _buildCanvasContent(markdownToRender, selectedFile, selectedContent, frame),
          ),
        ],
      ),
    );
  }

  Widget _buildCanvasContent(
    String? markdownToRender,
    String? selectedFile,
    String? selectedContent,
    CanvasFrame? frame,
  ) {
    if (markdownToRender != null) {
      if (isCanvasRawMode) {
        return SingleChildScrollView(
          padding: const EdgeInsets.all(14),
          child: SelectableText(
            markdownToRender,
            style: TextStyle(fontSize: 12, fontFamily: 'Consolas', color: DesktopTheme.textPrimary, height: 1.4),
          ),
        );
      }
      return _buildFormattedMarkdownView(markdownToRender);
    }

    if (selectedContent != null && selectedContent.isNotEmpty) {
      return _buildFileContentInCanvas(selectedFile ?? 'code', selectedContent);
    }

    if (frame == null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(FontAwesomeIcons.wandMagicSparkles, size: 36, color: DesktopTheme.accentCyan.withOpacity(0.6)),
              const SizedBox(height: 14),
              Text(
                'Холст пуст',
                style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: DesktopTheme.textPrimary),
              ),
              const SizedBox(height: 6),
              Text(
                'Агент может транслировать сюда Markdown-документы, интерактивные формы, графики и диаграммы через /ws/canvas.',
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
                        'content': '<form class="omnes-canvas"><label>Параметры деплоя:</label><input type="text" value="v1.0.0-rc2" /><button>Подтвердить</button></form>',
                      });
                    },
                  ),
                  OutlinedButton.icon(
                    icon: const Icon(FontAwesomeIcons.diagramProject, size: 11),
                    label: const Text('Демо Mermaid схемы', style: TextStyle(fontSize: 11)),
                    onPressed: () async {
                      await httpClient.postCanvas(activeCanvasId, {
                        'content_type': 'markdown',
                        '''content''': '''```mermaid\ngraph LR\nClient[Omnes Desktop ADE] -->|WS/chat| Gateway\nGateway --> Runtime\nGateway --> Canvas\n```''',
                      });
                    },
                  ),
                ],
              ),
            ],
          ),
        ),
      );
    }

    return SingleChildScrollView(
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
                  'Артефакт холста (${frame.contentType})',
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
                    Get.snackbar('Холст', 'Действие отправлено в шлюз через WS');
                  },
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildFileContentInCanvas(String path, String content) {
    final fileName = path.replaceAll(r'\', '/').split('/').last;
    final lines = content.split('\n');

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
          decoration: BoxDecoration(
            color: DesktopTheme.bgSurfaceElevated,
            border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle)),
          ),
          child: Row(
            children: [
              const Icon(Icons.code, size: 14, color: Color(0xFF00D2FF)),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  fileName,
                  style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold, fontFamily: 'Consolas', color: Colors.white),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              Text(
                '${lines.length} строк',
                style: TextStyle(fontSize: 11, color: DesktopTheme.textMuted),
              ),
              const SizedBox(width: 10),
              InkWell(
                onTap: () {
                  Clipboard.setData(ClipboardData(text: content));
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('Код скопирован в буфер'), duration: Duration(seconds: 1)),
                  );
                },
                borderRadius: BorderRadius.circular(4),
                child: Padding(
                  padding: const EdgeInsets.all(4),
                  child: Icon(Icons.copy, size: 13, color: DesktopTheme.textMuted),
                ),
              ),
            ],
          ),
        ),
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(12),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    for (int i = 1; i <= lines.length; i++)
                      Text(
                        '$i ',
                        style: const TextStyle(
                          fontFamily: 'Consolas',
                          fontSize: 11,
                          color: Color(0xFF64748B),
                          height: 1.4,
                        ),
                      ),
                  ],
                ),
                const SizedBox(width: 10),
                Container(width: 1, height: lines.length * 15.4, color: DesktopTheme.borderSubtle),
                const SizedBox(width: 10),
                Expanded(
                  child: SelectableText(
                    content,
                    style: TextStyle(
                      fontFamily: 'Consolas',
                      fontSize: 11,
                      color: DesktopTheme.textPrimary,
                      height: 1.4,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildFormattedMarkdownView(String text) {
    final lines = text.split('\n');
    final widgets = <Widget>[];

    for (int i = 0; i < lines.length; i++) {
      final line = lines[i];
      final trimmed = line.trim();

      // Markdown Table Detection in Inspector
      if ((trimmed.startsWith('|') || (trimmed.contains('|') && !trimmed.startsWith('#'))) && i + 1 < lines.length) {
        final nextTrimmed = lines[i + 1].trim();
        if (RegExp(r'^\s*\|?\s*:?-{2,}:?\s*(\|\s*:?-{2,}:?\s*)+\|?\s*$').hasMatch(nextTrimmed)) {
          final tableLines = <String>[line, lines[i + 1]];
          i += 2;
          while (i < lines.length) {
            final rowTrim = lines[i].trim();
            if (rowTrim.contains('|') && !rowTrim.startsWith('#') && rowTrim.isNotEmpty) {
              tableLines.add(lines[i]);
              i++;
            } else {
              break;
            }
          }
          i--;
          widgets.add(_buildInspectorMarkdownTable(tableLines));
          continue;
        }
      }

      widgets.add(_buildMarkdownLine(line));
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: widgets,
      ),
    );
  }

  /// Builds a cyber table for Inspector Canvas
  Widget _buildInspectorMarkdownTable(List<String> tableLines) {
    if (tableLines.length < 2) return const SizedBox.shrink();

    List<String> parseRow(String l) {
      String t = l.trim();
      if (t.startsWith('|')) t = t.substring(1);
      if (t.endsWith('|')) t = t.substring(0, t.length - 1);
      return t.split('|').map((c) => c.trim()).toList();
    }

    final headerCells = parseRow(tableLines[0]);
    if (headerCells.isEmpty) return const SizedBox.shrink();

    final dataRows = <List<String>>[];
    for (int r = 2; r < tableLines.length; r++) {
      final cells = parseRow(tableLines[r]);
      if (cells.isNotEmpty) dataRows.add(cells);
    }

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        color: DesktopTheme.bgSurface,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: DesktopTheme.borderSubtle, width: 1.0),
      ),
      clipBehavior: Clip.antiAlias,
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Table(
          defaultColumnWidth: const IntrinsicColumnWidth(),
          children: [
            TableRow(
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurfaceElevated,
                border: Border(bottom: BorderSide(color: DesktopTheme.borderSubtle, width: 1.0)),
              ),
              children: [
                for (final h in headerCells)
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                    child: Text(
                      h,
                      style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: DesktopTheme.textPrimary),
                    ),
                  ),
              ],
            ),
            for (int r = 0; r < dataRows.length; r++)
              TableRow(
                decoration: BoxDecoration(
                  color: r % 2 == 1 ? DesktopTheme.bgSurfaceElevated.withOpacity(0.35) : Colors.transparent,
                  border: Border(
                    bottom: r < dataRows.length - 1
                        ? BorderSide(color: DesktopTheme.borderSubtle.withOpacity(0.5), width: 0.8)
                        : BorderSide.none,
                  ),
                ),
                children: [
                  for (int c = 0; c < headerCells.length; c++)
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 7),
                      child: Text(
                        c < dataRows[r].length ? dataRows[r][c] : '',
                        style: TextStyle(fontSize: 12, color: DesktopTheme.textPrimary),
                      ),
                    ),
                ],
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildMarkdownLine(String rawLine) {
    final line = rawLine.trimRight();
    if (line.startsWith('# ')) {
      return Padding(
        padding: const EdgeInsets.only(top: 14, bottom: 6),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SelectableText(
              line.substring(2),
              style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: Color(0xFF00D2FF), letterSpacing: 0.3),
            ),
            const SizedBox(height: 4),
            const Divider(color: Color(0xFF334155), height: 1),
          ],
        ),
      );
    } else if (line.startsWith('## ')) {
      return Padding(
        padding: const EdgeInsets.only(top: 12, bottom: 4),
        child: SelectableText(
          line.substring(3),
          style: const TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: Colors.white),
        ),
      );
    } else if (line.startsWith('### ')) {
      return Padding(
        padding: const EdgeInsets.only(top: 10, bottom: 3),
        child: SelectableText(
          line.substring(4),
          style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600, color: Color(0xFF38BDF8)),
        ),
      );
    } else if (line.startsWith('- [ ] ') || line.startsWith('* [ ] ')) {
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 2),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.check_box_outline_blank, size: 14, color: Color(0xFF94A3B8)),
            const SizedBox(width: 8),
            Expanded(
              child: SelectableText(
                line.substring(6),
                style: const TextStyle(fontSize: 12, color: Color(0xFFE2E8F0)),
              ),
            ),
          ],
        ),
      );
    } else if (line.startsWith('- [x] ') || line.startsWith('* [x] ')) {
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 2),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.check_box, size: 14, color: Color(0xFF10B981)),
            const SizedBox(width: 8),
            Expanded(
              child: SelectableText(
                line.substring(6),
                style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8), decoration: TextDecoration.lineThrough),
              ),
            ),
          ],
        ),
      );
    } else if (line.startsWith('- ') || line.startsWith('* ')) {
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 2),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Padding(
              padding: EdgeInsets.only(top: 6, right: 8),
              child: Icon(Icons.circle, size: 5, color: Color(0xFF00D2FF)),
            ),
            Expanded(
              child: SelectableText(
                line.substring(2),
                style: const TextStyle(fontSize: 12, color: Color(0xFFE2E8F0), height: 1.4),
              ),
            ),
          ],
        ),
      );
    } else if (line.startsWith('> ')) {
      return Container(
        margin: const EdgeInsets.symmetric(vertical: 4),
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
        decoration: const BoxDecoration(
          border: Border(left: BorderSide(color: Color(0xFF00D2FF), width: 3)),
          color: Color(0xFF0F172A),
        ),
        child: SelectableText(
          line.substring(2),
          style: const TextStyle(fontSize: 12, fontStyle: FontStyle.italic, color: Color(0xFF94A3B8)),
        ),
      );
    } else if (line.startsWith('```')) {
      return Container(
        width: double.infinity,
        margin: const EdgeInsets.symmetric(vertical: 4),
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: const Color(0xFF050811),
          borderRadius: BorderRadius.circular(5),
          border: Border.all(color: const Color(0xFF1E293B)),
        ),
        child: SelectableText(
          line,
          style: const TextStyle(fontSize: 11, fontFamily: 'Consolas', color: Color(0xFF38BDF8)),
        ),
      );
    } else if (line == '---' || line == '***') {
      return const Padding(
        padding: EdgeInsets.symmetric(vertical: 8),
        child: Divider(color: Color(0xFF334155), height: 1),
      );
    } else if (line.isEmpty) {
      return const SizedBox(height: 6);
    } else {
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 1.5),
        child: SelectableText(
          line,
          style: const TextStyle(fontSize: 12, color: Color(0xFFE2E8F0), height: 1.45),
        ),
      );
    }
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
