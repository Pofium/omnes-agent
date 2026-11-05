// Unit tests for the secret-map grouping/validation helpers that back the
// config editor's secret key/value map block.

import 'package:flutter_test/flutter_test.dart';
import 'package:omnes_shared/omnes_shared.dart';

void main() {
  group('groupSecretMapEntries', () {
    test('keeps an empty container visible with no entries', () {
      final groups = groupSecretMapEntries([
        {'path': 'mcp_servers.github.env', 'kind': 'secret-map', 'type_hint': 'Option<HashMap<String,String>>'},
      ]);

      expect(groups, hasLength(1));
      expect(groups.single.path, 'mcp_servers.github.env');
      expect(groups.single.entryCount, 0);
      expect(groups.single.entries, isEmpty);
    });

    test('attaches only secret entry rows to their container', () {
      final groups = groupSecretMapEntries([
        {'path': 'mcp_servers.github.env', 'kind': 'secret-map', 'type_hint': 'Option<HashMap<String,String>>'},
        {'path': 'mcp_servers.github.env.GITHUB_TOKEN', 'kind': 'string', 'is_secret': true, 'populated': true},
        {'path': 'mcp_servers.github.env.LOG_LEVEL', 'kind': 'string', 'is_secret': true, 'populated': false},
        // Non-secret sibling must never be swallowed by the container.
        {'path': 'mcp_servers.github.env.NOT_SECRET', 'kind': 'string', 'is_secret': false},
      ]);

      expect(groups, hasLength(1));
      final entries = groups.single.entries;
      expect(entries.map((e) => e.key), ['GITHUB_TOKEN', 'LOG_LEVEL']);
      expect(entries.first.path, 'mcp_servers.github.env.GITHUB_TOKEN');
      expect(entries.first.populated, isTrue);
      expect(entries.last.populated, isFalse);
    });

    test('assigns an entry to the longest matching container', () {
      final groups = groupSecretMapEntries([
        {'path': 'a.headers', 'kind': 'secret-map'},
        {'path': 'a.b.headers', 'kind': 'secret-map'},
        {'path': 'a.b.headers.AUTHORIZATION', 'kind': 'string', 'is_secret': true},
      ]);

      final outer = groups.firstWhere((g) => g.path == 'a.headers');
      final inner = groups.firstWhere((g) => g.path == 'a.b.headers');
      expect(outer.entries, isEmpty);
      expect(inner.entries.map((e) => e.key), ['AUTHORIZATION']);
    });

    test('returns nothing when no container rows are present', () {
      expect(groupSecretMapEntries([
        {'path': 'model', 'kind': 'string'},
      ]), isEmpty);
    });
  });

  group('secretMapEntryKey / secretMapEntryPath', () {
    test('round-trips container and key', () {
      const container = 'mcp_servers.github.env';
      final path = secretMapEntryPath(container, 'GITHUB_TOKEN');
      expect(path, 'mcp_servers.github.env.GITHUB_TOKEN');
      expect(secretMapEntryKey(container, path), 'GITHUB_TOKEN');
    });

    test('returns the path unchanged when it is not under the container', () {
      expect(secretMapEntryKey('a.b', 'c.d'), 'c.d');
    });
  });

  group('secretMapKeyError', () {
    test('accepts a well-formed unique key', () {
      expect(secretMapKeyError('GITHUB_TOKEN', const ['OTHER']), isNull);
    });

    test('rejects empty and whitespace/=/control characters', () {
      expect(secretMapKeyError('', const []), SecretMapKeyError.invalid);
      expect(secretMapKeyError('has space', const []), SecretMapKeyError.invalid);
      expect(secretMapKeyError('has=eq', const []), SecretMapKeyError.invalid);
      expect(secretMapKeyError('tab\tkey', const []), SecretMapKeyError.invalid);
      expect(secretMapKeyError('ctrl\u0001key', const []), SecretMapKeyError.invalid);
    });

    test('rejects a duplicate key', () {
      expect(secretMapKeyError('DUP', const ['DUP']), SecretMapKeyError.exists);
    });
  });
}
