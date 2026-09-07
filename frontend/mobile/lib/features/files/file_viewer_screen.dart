// Screen for viewing and editing workspace text and markdown files.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../widgets/markdown_preview_widget.dart';
import 'workspace_repository.dart';

class FileViewerScreen extends StatefulWidget {
  final String filePath;
  final String fileName;
  final String? agentAlias;
  final bool isReadOnly;

  const FileViewerScreen({
    super.key,
    required this.filePath,
    required this.fileName,
    this.agentAlias,
    this.isReadOnly = false,
  });

  @override
  State<FileViewerScreen> createState() => _FileViewerScreenState();
}

class _FileViewerScreenState extends State<FileViewerScreen> with SingleTickerProviderStateMixin {
  final WorkspaceRepository _repo = WorkspaceRepository();
  late TextEditingController _textController;
  late TabController _tabController;

  bool _isLoading = true;
  bool _isSaving = false;
  bool _hasChanges = false;
  String _initialText = '';

  bool get _isMarkdown => widget.fileName.toLowerCase().endsWith('.md');

  @override
  void initState() {
    super.initState();
    _textController = TextEditingController();
    _tabController = TabController(length: _isMarkdown ? 2 : 1, vsync: this);
    _loadFile();
  }

  Future<void> _loadFile() async {
    setState(() => _isLoading = true);
    final res = await _repo.readFile(widget.filePath, agentAlias: widget.agentAlias);
    if (res != null) {
      _initialText = res.content;
      _textController.text = res.content;
    } else {
      _textController.text = '';
    }
    setState(() => _isLoading = false);
  }

  Future<void> _saveFile() async {
    if (widget.isReadOnly) return;
    setState(() => _isSaving = true);
    final success = await _repo.writeFile(
      widget.filePath,
      _textController.text,
      agentAlias: widget.agentAlias,
    );
    setState(() {
      _isSaving = false;
      if (success) {
        _hasChanges = false;
        _initialText = _textController.text;
      }
    });

    Get.snackbar(
      success ? 'Сохранено' : 'Ошибка',
      success ? 'Файл ${widget.fileName} сохранён' : 'Не удалось сохранить файл',
      snackPosition: SnackPosition.BOTTOM,
      backgroundColor: success ? Colors.green.shade800 : Colors.red.shade800,
      colorText: Colors.white,
      duration: const Duration(seconds: 2),
    );
  }

  @override
  void dispose() {
    _textController.dispose();
    _tabController.dispose();
    _repo.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(widget.fileName, style: const TextStyle(fontSize: 16)),
            Text(
              widget.filePath,
              style: TextStyle(fontSize: 11, color: Colors.grey.shade400),
              overflow: TextOverflow.ellipsis,
            ),
          ],
        ),
        actions: [
          if (!widget.isReadOnly)
            IconButton(
              tooltip: 'Сохранить',
              icon: _isSaving
                  ? const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                    )
                  : Icon(Icons.save, color: _hasChanges ? Colors.amber : Colors.white),
              onPressed: _isSaving ? null : _saveFile,
            ),
        ],
        bottom: _isMarkdown
            ? TabBar(
                controller: _tabController,
                tabs: const [
                  Tab(text: 'Редактор', icon: Icon(Icons.edit, size: 16)),
                  Tab(text: 'Просмотр', icon: Icon(Icons.visibility, size: 16)),
                ],
              )
            : null,
      ),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _isMarkdown
              ? TabBarView(
                  controller: _tabController,
                  children: [
                    _buildEditor(),
                    _buildPreview(),
                  ],
                )
              : _buildEditor(),
    );
  }

  Widget _buildEditor() {
    return Padding(
      padding: const EdgeInsets.all(12),
      child: TextField(
        controller: _textController,
        readOnly: widget.isReadOnly,
        maxLines: null,
        expands: true,
        keyboardType: TextInputType.multiline,
        style: const TextStyle(fontFamily: 'monospace', fontSize: 13, height: 1.4),
        decoration: InputDecoration(
          border: InputBorder.none,
          hintText: widget.isReadOnly ? 'Файл пуст' : 'Введите текст...',
        ),
        onChanged: (val) {
          setState(() {
            _hasChanges = val != _initialText;
          });
        },
      ),
    );
  }

  Widget _buildPreview() {
    return MarkdownPreviewWidget(text: _textController.text);
  }
}
