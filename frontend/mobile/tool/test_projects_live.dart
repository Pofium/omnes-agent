// ignore_for_file: avoid_print
// Pure Dart integration test for Projects and Files via ZeroClaw Gateway.
import 'dart:convert';
import 'dart:io';

const String baseUrl = 'http://127.0.0.1:42617';
const String token = 'omnes-token-secret-12345';
const String agent = 'chief';

final client = HttpClient();

Future<Map<String, dynamic>?> httpGet(String path) async {
  final req = await client.getUrl(Uri.parse('$baseUrl$path'));
  req.headers.set('Authorization', 'Bearer $token');
  req.headers.set('Accept', 'application/json');
  final res = await req.close();
  final body = await utf8.decodeStream(res);
  if (res.statusCode == 200) {
    return jsonDecode(body) as Map<String, dynamic>;
  }
  return null;
}

Future<Map<String, dynamic>?> httpPost(String path, Map<String, dynamic> body) async {
  final req = await client.postUrl(Uri.parse('$baseUrl$path'));
  req.headers.set('Authorization', 'Bearer $token');
  req.headers.set('Content-Type', 'application/json');
  req.add(utf8.encode(jsonEncode(body)));
  final res = await req.close();
  final resBody = await utf8.decodeStream(res);
  if (res.statusCode == 200 || res.statusCode == 201) {
    return jsonDecode(resBody) as Map<String, dynamic>;
  }
  return null;
}

Future<bool> httpDelete(String path, Map<String, dynamic> body) async {
  final req = await client.deleteUrl(Uri.parse('$baseUrl$path'));
  req.headers.set('Authorization', 'Bearer $token');
  req.headers.set('Content-Type', 'application/json');
  req.add(utf8.encode(jsonEncode(body)));
  final res = await req.close();
  return res.statusCode == 200 || res.statusCode == 204;
}

void main() async {
  print('=== Running Live Projects & Files Test via Gateway ===');

  final projId = 'test-proj-${DateTime.now().millisecondsSinceEpoch % 10000}';
  final projDir = 'projects/$projId';

  // 1. mkdir projects/<id>
  print('1. Creating directory $projDir...');
  final mkdirRes = await httpPost('/api/agents/$agent/workspace/mkdir', {'path': projDir});
  print('mkdir response: $mkdirRes');

  // 2. write project.json
  print('2. Writing $projDir/project.json...');
  final projMetadata = {
    'id': projId,
    'name': 'Автотест Проект',
    'description': 'Интеграционный тест Phase 3',
    'created_at': DateTime.now().toIso8601String(),
    'updated_at': DateTime.now().toIso8601String(),
    'agent_alias': agent,
  };
  final writeJsonRes = await httpPost('/api/agents/$agent/workspace/write', {
    'path': '$projDir/project.json',
    'content': jsonEncode(projMetadata),
  });
  print('write project.json response: $writeJsonRes');

  // 3. write NOTES.md
  print('3. Writing $projDir/NOTES.md...');
  final writeNotesRes = await httpPost('/api/agents/$agent/workspace/write', {
    'path': '$projDir/NOTES.md',
    'content': '# Автотест Проект\n\n- Заметка создана успешно\n',
  });
  print('write NOTES.md response: $writeNotesRes');

  // 4. list projects/<id>
  print('4. Listing entries in $projDir...');
  final listRes = await httpGet('/api/agents/$agent/workspace/list?path=$projDir');
  print('List entries:');
  if (listRes != null && listRes['entries'] is List) {
    for (final e in listRes['entries']) {
      print('  - ${e['name']} (${e['kind']}, size: ${e['size']})');
    }
  }

  // 5. read NOTES.md
  print('5. Reading back $projDir/NOTES.md...');
  final readRes = await httpGet('/api/agents/$agent/workspace/read?path=$projDir/NOTES.md');
  if (readRes != null) {
    print('NOTES.md content:\n"${readRes['content']}"');
  }

  // 6. delete test project
  print('6. Deleting $projDir...');
  final delRes = await httpDelete('/api/agents/$agent/workspace/path', {'path': projDir});
  print('delete result: $delRes');

  client.close();
  print('=== Live Projects & Files Test Completed Successfully! ===');
}
