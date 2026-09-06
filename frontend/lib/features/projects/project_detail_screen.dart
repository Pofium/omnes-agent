// Screen displaying details for a project: Chats, Files, and Notes tabs.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../core/gateway/gateway_http.dart';
import '../../routes/routes.dart';
import '../../widgets/markdown_preview_widget.dart';
import '../files/workspace_browser_widget.dart';
import '../sessions/models/session_info.dart';
import 'models/project_model.dart';
import 'project_repository.dart';

class ProjectDetailScreen extends StatefulWidget {
  final Project project;

  const ProjectDetailScreen({super.key, required this.project});

  @override
  State<ProjectDetailScreen> createState() => _ProjectDetailScreenState();
}

class _ProjectDetailScreenState extends State<ProjectDetailScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final ProjectRepository _projectRepo = ProjectRepository();
  final GatewayHttpClient _http = GatewayHttpClient();

  // Chats tab state
  bool _isLoadingChats = true;
  List<SessionInfo> _projectSessions = [];

  // Notes tab state
  bool _isLoadingNotes = true;
  bool _isSavingNotes = false;
  late TextEditingController _notesController;
  String _savedNotes = '';
  bool _hasNotesChanges = false;
  bool _previewNotes = false;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
    _notesController = TextEditingController();
    _loadProjectChats();
    _loadProjectNotes();
  }

  Future<void> _loadProjectChats() async {
    setState(() => _isLoadingChats = true);
    try {
      final allSessions = await _http.getSessionsList();
      // Filter sessions by workspaceDir or by session id prefix if matching
      final filtered = allSessions.where((s) {
        if (s.workspaceDir == widget.project.workspaceDir) return true;
        // Also match if sessionId contains project id
        if (s.sessionId.contains(widget.project.id)) return true;
        return false;
      }).toList();
      setState(() {
        _projectSessions = filtered;
        _isLoadingChats = false;
      });
    } catch (_) {
      setState(() => _isLoadingChats = false);
    }
  }

  Future<void> _loadProjectNotes() async {
    setState(() => _isLoadingNotes = true);
    try {
      final notes = await _projectRepo.getProjectNotes(
        widget.project.id,
        agentAlias: widget.project.agentAlias,
      );
      _savedNotes = notes;
      _notesController.text = notes;
    } catch (_) {} finally {
      setState(() => _isLoadingNotes = false);
    }
  }

  Future<void> _saveProjectNotes() async {
    setState(() => _isSavingNotes = true);
    final success = await _projectRepo.saveProjectNotes(
      widget.project.id,
      _notesController.text,
      agentAlias: widget.project.agentAlias,
    );
    setState(() {
      _isSavingNotes = false;
      if (success) {
        _savedNotes = _notesController.text;
        _hasNotesChanges = false;
      }
    });

    Get.snackbar(
      success ? 'Сохранено' : 'Ошибка',
      success ? 'Заметки сохранены' : 'Не удалось сохранить заметки',
      snackPosition: SnackPosition.BOTTOM,
      backgroundColor: success ? Colors.green.shade800 : Colors.red.shade800,
      colorText: Colors.white,
      duration: const Duration(seconds: 2),
    );
  }

  void _startNewChat() {
    Get.toNamed(Routes.chatScreen, arguments: {
      'isNew': true,
      'agentAlias': widget.project.agentAlias,
      'workspaceDir': widget.project.workspaceDir,
      'sessionId': 'proj_${widget.project.id}_${DateTime.now().millisecondsSinceEpoch}',
    })?.then((_) => _loadProjectChats());
  }

  void _openChat(SessionInfo session) {
    Get.toNamed(Routes.chatScreen, arguments: {
      'sessionId': session.sessionId,
      'agentAlias': session.agentAlias.isNotEmpty ? session.agentAlias : widget.project.agentAlias,
      'workspaceDir': widget.project.workspaceDir,
      'isNew': false,
    })?.then((_) => _loadProjectChats());
  }

  @override
  void dispose() {
    _tabController.dispose();
    _notesController.dispose();
    _projectRepo.dispose();
    _http.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(widget.project.name),
            Text(
              widget.project.workspaceDir,
              style: TextStyle(fontSize: 11, color: Colors.grey.shade400, fontFamily: 'monospace'),
            ),
          ],
        ),
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(icon: Icon(Icons.chat_bubble_outline), text: 'Чаты'),
            Tab(icon: Icon(Icons.folder_outlined), text: 'Файлы'),
            Tab(icon: Icon(Icons.edit_note), text: 'Заметки'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          _buildChatsTab(),
          _buildFilesTab(),
          _buildNotesTab(),
        ],
      ),
    );
  }

  Widget _buildChatsTab() {
    if (_isLoadingChats) {
      return const Center(child: CircularProgressIndicator());
    }

    return Scaffold(
      body: _projectSessions.isEmpty
          ? Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(Icons.chat_bubble_outline, size: 56, color: Colors.grey.shade400),
                  const SizedBox(height: 12),
                  Text(
                    'Нет чатов в этом проекте',
                    style: TextStyle(color: Colors.grey.shade600, fontSize: 16),
                  ),
                  const SizedBox(height: 8),
                  const Padding(
                    padding: EdgeInsets.symmetric(horizontal: 32),
                    child: Text(
                      'Все сообщения в проекте изолированы в его рабочей папке.',
                      textAlign: TextAlign.center,
                      style: TextStyle(fontSize: 12, color: Colors.grey),
                    ),
                  ),
                  const SizedBox(height: 16),
                  ElevatedButton.icon(
                    onPressed: _startNewChat,
                    icon: const Icon(Icons.add),
                    label: const Text('Начать диалог в проекте'),
                  ),
                ],
              ),
            )
          : RefreshIndicator(
              onRefresh: _loadProjectChats,
              child: ListView.separated(
                padding: const EdgeInsets.symmetric(vertical: 8),
                itemCount: _projectSessions.length,
                separatorBuilder: (_, __) => const Divider(height: 1),
                itemBuilder: (context, index) {
                  final s = _projectSessions[index];
                  return ListTile(
                    leading: CircleAvatar(
                      backgroundColor: Colors.blue.shade100,
                      child: const Icon(Icons.chat, color: Colors.blue),
                    ),
                    title: Text(
                      s.previewText.isNotEmpty ? s.previewText : s.sessionId,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
                    subtitle: Text(
                      '${s.messageCount} сообщ. • ${s.formattedLastActivity}',
                      style: TextStyle(fontSize: 12, color: Colors.grey.shade600),
                    ),
                    trailing: const Icon(Icons.arrow_forward_ios, size: 14),
                    onTap: () => _openChat(s),
                  );
                },
              ),
            ),
      floatingActionButton: _projectSessions.isNotEmpty
          ? FloatingActionButton(
              onPressed: _startNewChat,
              tooltip: 'Новый чат в проекте',
              child: const Icon(Icons.chat),
            )
          : null,
    );
  }

  Widget _buildFilesTab() {
    return WorkspaceBrowserWidget(
      rootPath: widget.project.workspaceDir,
      agentAlias: widget.project.agentAlias,
      showHeader: true,
    );
  }

  Widget _buildNotesTab() {
    if (_isLoadingNotes) {
      return const Center(child: CircularProgressIndicator());
    }

    return Column(
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
          color: Theme.of(context).cardColor,
          child: Row(
            children: [
              Text(
                'NOTES.md',
                style: const TextStyle(fontWeight: FontWeight.bold, fontFamily: 'monospace'),
              ),
              const Spacer(),
              IconButton(
                tooltip: _previewNotes ? 'Редактировать' : 'Предпросмотр',
                icon: Icon(_previewNotes ? Icons.edit : Icons.visibility, size: 20),
                onPressed: () => setState(() => _previewNotes = !_previewNotes),
              ),
              IconButton(
                tooltip: 'Сохранить заметки',
                icon: _isSavingNotes
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : Icon(Icons.save, color: _hasNotesChanges ? Colors.amber : null),
                onPressed: _isSavingNotes ? null : _saveProjectNotes,
              ),
            ],
          ),
        ),
        const Divider(height: 1),
        Expanded(
          child: _previewNotes
              ? MarkdownPreviewWidget(
                  text: _notesController.text.isNotEmpty
                      ? _notesController.text
                      : '*Заметки пусты*',
                )
              : Padding(
                  padding: const EdgeInsets.all(12),
                  child: TextField(
                    controller: _notesController,
                    maxLines: null,
                    expands: true,
                    keyboardType: TextInputType.multiline,
                    style: const TextStyle(fontFamily: 'monospace', fontSize: 13, height: 1.4),
                    decoration: const InputDecoration(
                      border: InputBorder.none,
                      hintText: 'Заметки по проекту в формате Markdown...',
                    ),
                    onChanged: (val) {
                      setState(() {
                        _hasNotesChanges = val != _savedNotes;
                      });
                    },
                  ),
                ),
        ),
      ],
    );
  }
}
