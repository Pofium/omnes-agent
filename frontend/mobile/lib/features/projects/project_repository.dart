// Repository for managing Omnes projects via workspace API convention:
// projects/<id>/project.json and projects/<id>/NOTES.md

import 'dart:convert';
import '../../core/gateway/gateway_config.dart';
import '../../core/gateway/gateway_http.dart';
import 'models/project_model.dart';

class ProjectRepository {
  final GatewayHttpClient _http;

  ProjectRepository({GatewayHttpClient? http}) : _http = http ?? GatewayHttpClient();

  Future<String> _resolveAgent(String? agentAlias) async {
    if (agentAlias != null && agentAlias.isNotEmpty) return agentAlias;
    return GatewayConfig.getActiveAgent();
  }

  /// Lists all projects in projects/ folder for the agent workspace.
  Future<List<Project>> listProjects({String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    final entries = await _http.workspaceList(alias, path: 'projects');
    final dirEntries = entries.where((e) => e.isDir).toList();

    final projects = <Project>[];
    for (final dir in dirEntries) {
      final jsonPath = 'projects/${dir.name}/project.json';
      final file = await _http.workspaceReadFile(alias, jsonPath);
      if (file != null && file.content.isNotEmpty) {
        try {
          final decoded = jsonDecode(file.content) as Map<String, dynamic>;
          projects.add(Project.fromJson(decoded, dir.name));
          continue;
        } catch (_) {}
      }
      // Fallback if project.json does not exist yet in that directory
      projects.add(Project(
        id: dir.name,
        name: dir.name,
        createdAt: DateTime.now(),
        updatedAt: DateTime.now(),
        agentAlias: alias,
      ));
    }

    projects.sort((a, b) => b.updatedAt.compareTo(a.updatedAt));
    return projects;
  }

  /// Retrieves a specific project by id.
  Future<Project?> getProject(String projectId, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    final jsonPath = 'projects/$projectId/project.json';
    final file = await _http.workspaceReadFile(alias, jsonPath);
    if (file != null && file.content.isNotEmpty) {
      try {
        final decoded = jsonDecode(file.content) as Map<String, dynamic>;
        return Project.fromJson(decoded, projectId);
      } catch (_) {}
    }
    return Project(
      id: projectId,
      name: projectId,
      createdAt: DateTime.now(),
      updatedAt: DateTime.now(),
      agentAlias: alias,
    );
  }

  /// Creates a new project: creates directory projects/<id>/, project.json and NOTES.md.
  Future<Project?> createProject({
    required String name,
    String description = '',
    String? agentAlias,
  }) async {
    final alias = await _resolveAgent(agentAlias);
    final safeId = _slugify(name);
    final id = '$safeId-${DateTime.now().millisecondsSinceEpoch % 100000}';

    // 1. Create directory
    final dirCreated = await _http.workspaceMkdir(alias, 'projects/$id');
    if (!dirCreated) {
      // Continue anyway, write will create directory if supported
    }

    // 2. Write project.json
    final now = DateTime.now();
    final project = Project(
      id: id,
      name: name.trim(),
      description: description.trim(),
      createdAt: now,
      updatedAt: now,
      agentAlias: alias,
    );
    final jsonStr = const JsonEncoder.withIndent('  ').convert(project.toJson());
    await _http.workspaceWriteFile(alias, 'projects/$id/project.json', jsonStr);

    // 3. Write initial NOTES.md
    final initialNotes = '# $name\n\n${description.isNotEmpty ? '$description\n\n' : ''}## Заметки\n';
    await _http.workspaceWriteFile(alias, 'projects/$id/NOTES.md', initialNotes);

    return project;
  }

  /// Updates project metadata in project.json.
  Future<bool> updateProject(Project project, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias ?? project.agentAlias);
    final updated = project.copyWith(updatedAt: DateTime.now());
    final jsonStr = const JsonEncoder.withIndent('  ').convert(updated.toJson());
    return _http.workspaceWriteFile(alias, 'projects/${project.id}/project.json', jsonStr);
  }

  /// Deletes a project folder from agent workspace.
  Future<bool> deleteProject(String projectId, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.workspaceDelete(alias, 'projects/$projectId');
  }

  /// Reads NOTES.md content for a project.
  Future<String> getProjectNotes(String projectId, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    final file = await _http.workspaceReadFile(alias, 'projects/$projectId/NOTES.md');
    return file?.content ?? '';
  }

  /// Saves NOTES.md content for a project.
  Future<bool> saveProjectNotes(String projectId, String notes, {String? agentAlias}) async {
    final alias = await _resolveAgent(agentAlias);
    return _http.workspaceWriteFile(alias, 'projects/$projectId/NOTES.md', notes);
  }

  String _slugify(String text) {
    final clean = text
        .toLowerCase()
        .trim()
        .replaceAll(RegExp(r'[^a-zA-Z0-9а-яА-ЯёЁ]+'), '-')
        .replaceAll(RegExp(r'^-+|-+$'), '');
    return clean.isEmpty ? 'proj' : clean;
  }

  void dispose() {
    _http.dispose();
  }
}
