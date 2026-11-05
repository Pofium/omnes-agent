// Pure helpers for `kind == "secret-map"` config rows.
//
// The gateway emits one container row per secret key/value map (MCP
// `env`/`headers`, provider `extra_headers`, plugin `config`, file-transfer
// `headers`, observability `otel_headers`, ...) — present even when the map is
// empty, so the UI can offer "add entry" — followed by one secret
// `<container>.<KEY>` row per existing entry. The editor renders the entries
// inside the container block instead of scattering them through the flat list.
//
// Mirrors upstream `web/src/components/sections/secretMap.logic.ts`.

/// Wire `kind` tag for a secret key/value map container row.
const String kSecretMapKind = 'secret-map';

/// Rejection reasons for a new entry name.
enum SecretMapKeyError { invalid, exists }

/// One `<container>.<KEY>` entry row belonging to a secret-map container.
class SecretMapEntry {
  /// Full config path of the entry row (`<container>.<KEY>`).
  final String path;

  /// Map key relative to the container (the `<KEY>` segment).
  final String key;

  /// Whether the entry currently holds a value (gateway `populated` flag).
  final bool populated;

  const SecretMapEntry({
    required this.path,
    required this.key,
    required this.populated,
  });
}

/// A secret-map container row plus the entry rows that belong to it.
class SecretMapGroup {
  /// Container config path, e.g. `mcp_servers.github.env`.
  final String path;

  /// Rust type signature for tooltips, e.g. `Option<HashMap<String,String>>`.
  final String typeHint;

  /// Human description emitted by the gateway, if any.
  final String description;

  /// Entry rows in list order.
  final List<SecretMapEntry> entries;

  const SecretMapGroup({
    required this.path,
    required this.typeHint,
    required this.description,
    required this.entries,
  });

  /// Number of entries currently present in the map.
  int get entryCount => entries.length;

  /// Keys of the current entries, in list order.
  List<String> get entryKeys => entries.map((e) => e.key).toList();
}

/// Groups `GET /api/config/list` entries into secret-map containers.
///
/// An entry belongs to the longest container whose path is a strict dotted
/// prefix of it; only secret rows qualify, so a sibling field can never be
/// swallowed by a container. Each returned group carries its own entries in
/// the original list order.
List<SecretMapGroup> groupSecretMapEntries(List<dynamic> entries) {
  final containers = <Map<dynamic, dynamic>>[];
  for (final raw in entries) {
    final map = _asMap(raw);
    if (map == null) continue;
    if (_str(map['kind']) == kSecretMapKind) containers.add(map);
  }
  if (containers.isEmpty) return const [];

  // Longest container path first, so an entry can never be claimed by a
  // shallower container that happens to share a prefix.
  final containerPaths = containers.map((c) => _str(c['path'])).toList()
    ..sort((a, b) => b.length.compareTo(a.length));

  final grouped = <String, List<SecretMapEntry>>{
    for (final c in containers) _str(c['path']): <SecretMapEntry>[],
  };

  for (final raw in entries) {
    final map = _asMap(raw);
    if (map == null) continue;
    if (_str(map['kind']) == kSecretMapKind) continue;
    if (map['is_secret'] != true) continue;
    final path = _str(map['path']);
    for (final containerPath in containerPaths) {
      if (path.startsWith('$containerPath.')) {
        grouped[containerPath]!.add(SecretMapEntry(
          path: path,
          key: secretMapEntryKey(containerPath, path),
          populated: map['populated'] == true,
        ));
        break;
      }
    }
  }

  return [
    for (final c in containers)
      SecretMapGroup(
        path: _str(c['path']),
        typeHint: _str(c['type_hint']),
        description: _str(c['description']),
        entries: grouped[_str(c['path'])]!,
      ),
  ];
}

/// Returns the entry key of [entryPath] relative to [containerPath].
String secretMapEntryKey(String containerPath, String entryPath) {
  final prefix = '$containerPath.';
  return entryPath.startsWith(prefix)
      ? entryPath.substring(prefix.length)
      : entryPath;
}

/// Builds the config path of one secret-map entry (`<container>.<KEY>`).
String secretMapEntryPath(String containerPath, String key) =>
    '$containerPath.$key';

/// Control characters plus whitespace and `=`, which can never be valid in a
/// map key on any surface this row type fronts.
final RegExp _invalidKeyPattern = RegExp(r'[\s=\u0000-\u001f\u007f]');

/// Validates a new entry name. Returns null when acceptable.
///
/// Deliberately loose — the same row type fronts env vars, HTTP headers and
/// plugin settings — so only names that can never be valid are refused.
SecretMapKeyError? secretMapKeyError(String key, List<String> existingKeys) {
  if (key.isEmpty || _invalidKeyPattern.hasMatch(key)) {
    return SecretMapKeyError.invalid;
  }
  if (existingKeys.contains(key)) return SecretMapKeyError.exists;
  return null;
}

Map<dynamic, dynamic>? _asMap(dynamic value) =>
    value is Map ? value : null;

String _str(dynamic value) => value?.toString() ?? '';
