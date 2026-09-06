// Repository for browsing and modifying files in an agent's workspace.

import '../../core/gateway/gateway_config.dart';
import '../../core/gateway/gateway_http.dart';
import 'models/workspace_entry.dart';

class WorkspaceRepository {
  final GatewayHttpClient _http;

  WorkspaceRepository({GatewayHttpClient? http}) : _http = http ?? GatewayHttpClient();

  Future<String> _resolveAgent(String? agentAlias) async {
    if (agentAlias != null && agentAlias.isNotEmpty) return agentAlias;
    return GatewayConfig.getActiveAgent();
  }

  /// Lists entries in a given relative path within the agent workspace.
  Future<List<WorkspaceEntry>> listDirectory(String path, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.workspaceList(alias, path: path);
  }

  /// Reads a file from the workspace.
  Future<WorkspaceFileContent?> readFile(String path, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.workspaceReadFile(alias, path);
  }

  /// Downloads raw binary or text bytes from the workspace.
  Future<List<int>?> downloadRawFile(String path, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.workspaceRawFileBytes(alias, path);
  }

  /// Returns the full raw URL for direct network images or browser downloads.
  Future<String> getRawFileUrl(String path, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    final baseUrl = GatewayConfig.getBaseUrl();
    final encodedPath = Uri.encodeComponent(path.trim().replaceAll(RegExp(r'^/+|/+$'), ''));
    return '$baseUrl/api/agents/${Uri.encodeComponent(alias)}/workspace/raw?path=$encodedPath';
  }

  /// Writes text content to a file.
  Future<bool> writeFile(String path, String content, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.workspaceWriteFile(alias, path, content);
  }

  /// Creates a directory.
  Future<bool> createDirectory(String path, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.workspaceMkdir(alias, path);
  }

  /// Moves or renames a path.
  Future<bool> movePath(String from, String to, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.workspaceMove(alias, from, to);
  }

  /// Deletes a path (file or directory).
  Future<bool> deletePath(String path, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.workspaceDelete(alias, path);
  }

  /// Uploads binary file bytes (image or document).
  Future<String?> uploadFile(List<int> bytes, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.uploadFile(alias, bytes);
  }

  void dispose() {
    _http.dispose();
  }
}
