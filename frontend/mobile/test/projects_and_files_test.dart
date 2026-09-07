import 'package:flutter_test/flutter_test.dart';
import 'package:omagent_front/features/files/models/workspace_entry.dart';
import 'package:omagent_front/features/projects/models/project_model.dart';

void main() {
  group('WorkspaceEntry model parsing', () {
    test('parses directory entry correctly', () {
      final json = {
        'name': 'notes',
        'kind': 'dir',
        'protected': false,
      };
      final entry = WorkspaceEntry.fromJson(json, parentPath: 'projects/alpha');
      expect(entry.name, equals('notes'));
      expect(entry.path, equals('projects/alpha/notes'));
      expect(entry.isDir, isTrue);
      expect(entry.isFile, isFalse);
      expect(entry.formattedSize, equals(''));
      expect(entry.protected, isFalse);
    });

    test('parses file entry correctly with formatting', () {
      final json = {
        'name': 'report.md',
        'kind': 'file',
        'size': 2048,
        'protected': false,
      };
      final entry = WorkspaceEntry.fromJson(json, parentPath: '');
      expect(entry.name, equals('report.md'));
      expect(entry.path, equals('report.md'));
      expect(entry.isFile, isTrue);
      expect(entry.extension, equals('md'));
      expect(entry.formattedSize, equals('2.0 KB'));
    });

    test('parses protected file entry', () {
      final json = {
        'name': 'IDENTITY.md',
        'kind': 'file',
        'size': 270,
        'protected': true,
      };
      final entry = WorkspaceEntry.fromJson(json);
      expect(entry.protected, isTrue);
      expect(entry.formattedSize, equals('270 B'));
    });

    test('parses WorkspaceFileContent', () {
      final json = {
        'path': 'projects/test/NOTES.md',
        'size': 42,
        'is_text': true,
        'content': '# My Project Notes',
        'encoding': 'utf8',
      };
      final content = WorkspaceFileContent.fromJson(json);
      expect(content.path, equals('projects/test/NOTES.md'));
      expect(content.size, equals(42));
      expect(content.isText, isTrue);
      expect(content.content, equals('# My Project Notes'));
      expect(content.encoding, equals('utf8'));
    });
  });

  group('Project model serialization', () {
    test('parses Project from json and computes workspaceDir', () {
      final now = DateTime.now();
      final json = {
        'id': 'diploma-2026',
        'name': 'Дипломная работа',
        'description': 'Исследование мультиагентных систем',
        'created_at': now.toIso8601String(),
        'updated_at': now.toIso8601String(),
        'agent_alias': 'chief',
      };

      final project = Project.fromJson(json, 'diploma-2026');
      expect(project.id, equals('diploma-2026'));
      expect(project.name, equals('Дипломная работа'));
      expect(project.description, equals('Исследование мультиагентных систем'));
      expect(project.workspaceDir, equals('projects/diploma-2026'));
      expect(project.agentAlias, equals('chief'));

      final encoded = project.toJson();
      expect(encoded['name'], equals('Дипломная работа'));
      expect(encoded['agent_alias'], equals('chief'));
    });

    test('copyWith updates fields correctly', () {
      final project = Project(
        id: 'test-1',
        name: 'Initial Name',
        createdAt: DateTime.now(),
        updatedAt: DateTime.now(),
      );
      final updated = project.copyWith(name: 'Updated Name', description: 'New desc');
      expect(updated.id, equals('test-1'));
      expect(updated.name, equals('Updated Name'));
      expect(updated.description, equals('New desc'));
    });
  });
}
