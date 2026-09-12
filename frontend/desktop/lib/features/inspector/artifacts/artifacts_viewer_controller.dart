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

  /// Removes a single artifact by index.
  void removeArtifact(int index) {
    if (index >= 0 && index < _artifacts.length) {
      _artifacts.removeAt(index);
      if (_selectedIndex >= _artifacts.length) {
        _selectedIndex = (_artifacts.length - 1).clamp(0, 9999);
      }
      notifyListeners();
    }
  }

  /// Clears all artifacts.
  void clearArtifacts() {
    _artifacts.clear();
    _selectedIndex = 0;
    notifyListeners();
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
