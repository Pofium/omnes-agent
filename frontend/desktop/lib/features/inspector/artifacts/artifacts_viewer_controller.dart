import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:omnes_shared/omnes_shared.dart';

/// Representation of an artifact generated or modified by the agent.
class AgentArtifact {
  final String id;
  final String name;
  final String path;
  final String type; // 'markdown', 'code', 'svg', 'html', 'diff'
  final String content;
  final String? diff;
  final DateTime updatedAt;

  AgentArtifact({
    required this.id,
    required this.name,
    required this.path,
    required this.type,
    required this.content,
    this.diff,
    DateTime? updatedAt,
  }) : updatedAt = updatedAt ?? DateTime.now();
}

/// Controller managing artifacts and file diffs in the ADE Inspector.
class ArtifactsViewerController extends ChangeNotifier {
  final GatewayHttpClient httpClient;
  final List<AgentArtifact> _artifacts = [];
  int _selectedIndex = 0;
  bool _isLoading = false;
  StreamSubscription? _sseSub;

  ArtifactsViewerController({required this.httpClient}) {
    _initSampleArtifacts();
    _subscribeSse();
  }

  List<AgentArtifact> get artifacts => List.unmodifiable(_artifacts);
  int get selectedIndex => _selectedIndex;
  AgentArtifact? get activeArtifact =>
      _artifacts.isNotEmpty && _selectedIndex < _artifacts.length ? _artifacts[_selectedIndex] : null;
  bool get isLoading => _isLoading;

  void selectArtifact(int index) {
    if (index >= 0 && index < _artifacts.length) {
      _selectedIndex = index;
      notifyListeners();
    }
  }

  void _initSampleArtifacts() {
    _artifacts.addAll([
      AgentArtifact(
        id: '1',
        name: 'task_workspace_controller.dart',
        path: 'frontend/desktop/lib/features/workspace/task_workspace_controller.dart',
        type: 'diff',
        content: '''// Controller managing multi-session workspaces with real-time SSE & WebSocket.''',
        diff: '''--- a/task_workspace_controller.dart
+++ b/task_workspace_controller.dart
@@ -105,6 +105,12 @@
       // Start global SSE stream
       GatewaySseClient.instance.connect();
+      // Connect real-time WebSocket for active session
+      _connectWebSocket(activeSessionId.value);
+      // Add DOM selector chip to context
+      addDomSelectorChip(selector);
''',
      ),
      AgentArtifact(
        id: '2',
        name: 'architecture_diagram.svg',
        path: 'docs/assets/architecture.svg',
        type: 'svg',
        content: '''<svg width="400" height="160" xmlns="http://www.w3.org/2000/svg">
  <rect width="400" height="160" rx="10" fill="#0F172A" stroke="#00D2FF" stroke-width="2"/>
  <text x="20" y="40" fill="#00D2FF" font-size="16" font-family="Consolas" font-weight="bold">OmnesAgent ADE Architecture</text>
  <rect x="20" y="60" width="160" height="70" rx="6" fill="#1E293B" stroke="#334155"/>
  <text x="35" y="95" fill="#F8FAFC" font-size="12" font-family="Consolas">Gateway :42617</text>
  <text x="35" y="115" fill="#94A3B8" font-size="10" font-family="Consolas">REST &amp; WS Engine</text>
  <path d="M 180 95 L 230 95" stroke="#00D2FF" stroke-width="2" marker-end="url(#arrow)"/>
  <rect x="230" y="60" width="150" height="70" rx="6" fill="#1E293B" stroke="#00D2FF"/>
  <text x="245" y="95" fill="#00D2FF" font-size="12" font-family="Consolas">Desktop ADE</text>
  <text x="245" y="115" fill="#94A3B8" font-size="10" font-family="Consolas">Flutter + Edge WV2</text>
</svg>''',
      ),
      AgentArtifact(
        id: '3',
        name: 'release_report.md',
        path: 'reports/release_report.md',
        type: 'markdown',
        content: '''# OmnesAgent Release Verification Report

## Status Summary
- **Gateway Health**: 100% Operational (Port 42617)
- **Active Protocols**: WebSocket Chat, A2UI Canvas, SSE Stream
- **UI Architecture**: Cyber-ADE Theme, Adaptive Light Mode
''',
      ),
    ]);
  }

  void _subscribeSse() {
    _sseSub = GatewaySseClient.instance.events.listen((event) {
      if (event.type == 'file_change' || event.type == 'artifact_created') {
        final path = event.data['path']?.toString() ?? 'artifact.txt';
        final name = path.split('/').last;
        final content = event.data['content']?.toString() ?? '';
        final diff = event.data['diff']?.toString();

        _artifacts.insert(
          0,
          AgentArtifact(
            id: DateTime.now().millisecondsSinceEpoch.toString(),
            name: name,
            path: path,
            type: path.endsWith('.svg')
                ? 'svg'
                : (diff != null ? 'diff' : (path.endsWith('.md') ? 'markdown' : 'code')),
            content: content,
            diff: diff,
          ),
        );
        notifyListeners();
      }
    });
  }

  /// Applies change from diff back to filesystem.
  Future<bool> applyDiff(AgentArtifact artifact) async {
    _isLoading = true;
    notifyListeners();
    try {
      if (artifact.content.isNotEmpty) {
        await httpClient.workspaceWriteFile('main', artifact.path, artifact.content);
      }
      _isLoading = false;
      notifyListeners();
      return true;
    } catch (_) {
      _isLoading = false;
      notifyListeners();
      return false;
    }
  }

  /// Discards / rolls back the proposed change.
  void rollbackArtifact(AgentArtifact artifact) {
    _artifacts.remove(artifact);
    if (_selectedIndex >= _artifacts.length) {
      _selectedIndex = _artifacts.isEmpty ? 0 : _artifacts.length - 1;
    }
    notifyListeners();
  }

  @override
  void dispose() {
    _sseSub?.cancel();
    super.dispose();
  }
}
