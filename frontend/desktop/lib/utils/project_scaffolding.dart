// Project Scaffolding and Lifecycle Management Service for OmnesAgent Desktop.
// Features intelligent Pre-production Analysis (automated stack & environment detection),
// Git init, AGENTS.md architecture rules creation, OB2H code-intelligence indexing,
// and native system integration (Explorer, VS Code, Terminal).
// STRICT: No emojis are used in the codebase, only SVG and Icon assets.

import 'package:universal_io/io.dart';

/// Result of automated pre-production analysis by the agent.
class ProjectAnalysisResult {
  final String detectedStack;
  final String iconKey; // 'rust', 'flutter', 'web', 'python', 'go', 'php', 'dotnet', 'java', 'cpp', 'folder'
  final String domain;
  final List<String> recommendedAgents;
  final List<String> recommendedTools;
  final List<String> recommendedSkills;
  final bool isDirectoryEmpty;
  final bool hasGit;
  final bool hasAgentsMd;
  final bool hasReadme;
  final String summary;

  const ProjectAnalysisResult({
    required this.detectedStack,
    required this.iconKey,
    required this.domain,
    required this.recommendedAgents,
    required this.recommendedTools,
    required this.recommendedSkills,
    required this.isDirectoryEmpty,
    required this.hasGit,
    required this.hasAgentsMd,
    required this.hasReadme,
    required this.summary,
  });
}

/// Automated Pre-production Analyzer: inspects project files and directory structure,
/// determining the stack, build tools, and AI agent configuration without manual selection.
class ProjectAnalyzer {
  static ProjectAnalysisResult analyze(String projectPath) {
    final cleanPath = projectPath.trim();
    if (cleanPath.isEmpty) {
      return const ProjectAnalysisResult(
        detectedStack: 'Не указан путь',
        iconKey: 'folder',
        domain: 'Общее',
        recommendedAgents: ['chief'],
        recommendedTools: ['filesystem', 'git'],
        recommendedSkills: ['ob2h'],
        isDirectoryEmpty: true,
        hasGit: false,
        hasAgentsMd: false,
        hasReadme: false,
        summary: 'Укажите путь к проекту на диске.',
      );
    }

    try {
      final dir = Directory(cleanPath);
      if (!dir.existsSync()) {
        return const ProjectAnalysisResult(
          detectedStack: 'Новый проект',
          iconKey: 'folder',
          domain: 'Общее',
          recommendedAgents: ['chief', 'code-agent'],
          recommendedTools: ['filesystem', 'git'],
          recommendedSkills: ['ob2h'],
          isDirectoryEmpty: true,
          hasGit: false,
          hasAgentsMd: false,
          hasReadme: false,
          summary: 'Папка будет создана. Агент самостоятельно определит стек и структуру по вашей первой задаче.',
        );
      }

      final entries = dir.listSync();
      final files = entries.map((e) => e.path.replaceAll(r'\', '/').split('/').last).toSet();
      final hasGit = files.contains('.git');
      final hasAgentsMd = files.any((f) => f.toLowerCase() == 'agents.md');
      final hasReadme = files.any((f) => f.toLowerCase() == 'readme.md');
      final isEmpty = files.isEmpty;

      if (isEmpty) {
        return ProjectAnalysisResult(
          detectedStack: 'Новый проект (Свободный стек)',
          iconKey: 'folder',
          domain: 'Общее',
          recommendedAgents: ['chief', 'code-agent'],
          recommendedTools: ['filesystem', 'git'],
          recommendedSkills: ['ob2h'],
          isDirectoryEmpty: true,
          hasGit: hasGit,
          hasAgentsMd: hasAgentsMd,
          hasReadme: hasReadme,
          summary: 'Папка пуста. Агент сформирует архитектуру, стек и зависимости на основе первой задачи.',
        );
      }

      // Detection markers
      final isRust = files.contains('Cargo.toml');
      final isWeb = files.contains('package.json');
      final isFlutter = files.contains('pubspec.yaml');
      final isPython = files.contains('pyproject.toml') ||
          files.contains('requirements.txt') ||
          files.contains('Pipfile') ||
          files.contains('setup.py');
      final isGo = files.contains('go.mod');
      final isPhp = files.contains('composer.json');
      final isDotNet = files.any((f) => f.endsWith('.sln') || f.endsWith('.csproj'));
      final isJava = files.contains('pom.xml') || files.contains('build.gradle') || files.contains('build.gradle.kts');
      final isCpp = files.contains('CMakeLists.txt') || files.contains('Makefile');

      if (isRust) {
        return ProjectAnalysisResult(
          detectedStack: 'Rust (Cargo Workspace / Crate)',
          iconKey: 'rust',
          domain: 'Rust / Системная разработка',
          recommendedAgents: ['chief', 'code-agent'],
          recommendedTools: ['cargo', 'rust-analyzer', 'git'],
          recommendedSkills: ['ob2h', 'testsprite'],
          isDirectoryEmpty: false,
          hasGit: hasGit,
          hasAgentsMd: hasAgentsMd,
          hasReadme: hasReadme,
          summary: 'Обнаружен Rust-проект (Cargo.toml). Подключены компилятор rustc, cargo и AST-анализ через OB2H.',
        );
      }

      if (isFlutter) {
        return ProjectAnalysisResult(
          detectedStack: 'Flutter / Dart Client',
          iconKey: 'flutter',
          domain: 'Веб-приложения / Frontend',
          recommendedAgents: ['chief', 'code-agent', 'web-agent'],
          recommendedTools: ['flutter', 'dart', 'git'],
          recommendedSkills: ['ob2h', 'dart-mcp-server'],
          isDirectoryEmpty: false,
          hasGit: hasGit,
          hasAgentsMd: hasAgentsMd,
          hasReadme: hasReadme,
          summary: 'Обнаружен проект Flutter (pubspec.yaml). Подключены инструменты Dart и Flutter CLI.',
        );
      }

      if (isWeb) {
        return ProjectAnalysisResult(
          detectedStack: 'Web / Node.js / TypeScript',
          iconKey: 'web',
          domain: 'Веб-приложения / Frontend',
          recommendedAgents: ['web-agent', 'code-agent'],
          recommendedTools: ['npm', 'chrome-devtools', 'playwright'],
          recommendedSkills: ['magicui', 'shadcn-ui', 'ob2h'],
          isDirectoryEmpty: false,
          hasGit: hasGit,
          hasAgentsMd: hasAgentsMd,
          hasReadme: hasReadme,
          summary: 'Обнаружен веб-проект (package.json). Подключены браузерные средства Playwright и Chrome DevTools.',
        );
      }

      if (isPython) {
        return ProjectAnalysisResult(
          detectedStack: 'Python / AI / Backend',
          iconKey: 'python',
          domain: 'AI / Data Science',
          recommendedAgents: ['chief', 'code-agent'],
          recommendedTools: ['python', 'ruff', 'fetch'],
          recommendedSkills: ['science', 'ob2h'],
          isDirectoryEmpty: false,
          hasGit: hasGit,
          hasAgentsMd: hasAgentsMd,
          hasReadme: hasReadme,
          summary: 'Обнаружен Python-проект (pyproject/requirements). Подключены инструменты Python 3.13 и ruff.',
        );
      }

      if (isGo) {
        return ProjectAnalysisResult(
          detectedStack: 'Go (Golang Module)',
          iconKey: 'go',
          domain: 'Rust / Системная разработка',
          recommendedAgents: ['chief', 'code-agent'],
          recommendedTools: ['go', 'git'],
          recommendedSkills: ['ob2h'],
          isDirectoryEmpty: false,
          hasGit: hasGit,
          hasAgentsMd: hasAgentsMd,
          hasReadme: hasReadme,
          summary: 'Обнаружен Go-модуль (go.mod).',
        );
      }

      if (isPhp) {
        return ProjectAnalysisResult(
          detectedStack: 'PHP (Composer)',
          iconKey: 'php',
          domain: 'Веб-приложения / Frontend',
          recommendedAgents: ['chief', 'code-agent'],
          recommendedTools: ['composer', 'php', 'git'],
          recommendedSkills: ['ob2h'],
          isDirectoryEmpty: false,
          hasGit: hasGit,
          hasAgentsMd: hasAgentsMd,
          hasReadme: hasReadme,
          summary: 'Обнаружен проект PHP 8.3/8.4 (composer.json).',
        );
      }

      if (isDotNet) {
        return ProjectAnalysisResult(
          detectedStack: '.NET / C#',
          iconKey: 'dotnet',
          domain: 'Rust / Системная разработка',
          recommendedAgents: ['chief', 'code-agent'],
          recommendedTools: ['dotnet', 'git'],
          recommendedSkills: ['ob2h'],
          isDirectoryEmpty: false,
          hasGit: hasGit,
          hasAgentsMd: hasAgentsMd,
          hasReadme: hasReadme,
          summary: 'Обнаружен проект Microsoft .NET / C#.',
        );
      }

      if (isJava) {
        return ProjectAnalysisResult(
          detectedStack: 'Java / Kotlin (Gradle/Maven)',
          iconKey: 'java',
          domain: 'Общее',
          recommendedAgents: ['chief', 'code-agent'],
          recommendedTools: ['java', 'git'],
          recommendedSkills: ['ob2h'],
          isDirectoryEmpty: false,
          hasGit: hasGit,
          hasAgentsMd: hasAgentsMd,
          hasReadme: hasReadme,
          summary: 'Обнаружен Java/Kotlin проект.',
        );
      }

      if (isCpp) {
        return ProjectAnalysisResult(
          detectedStack: 'C / C++ (CMake/Make)',
          iconKey: 'cpp',
          domain: 'Rust / Системная разработка',
          recommendedAgents: ['chief', 'code-agent'],
          recommendedTools: ['cmake', 'git'],
          recommendedSkills: ['ob2h'],
          isDirectoryEmpty: false,
          hasGit: hasGit,
          hasAgentsMd: hasAgentsMd,
          hasReadme: hasReadme,
          summary: 'Обнаружен проект C/C++.',
        );
      }

      return ProjectAnalysisResult(
        detectedStack: 'Существующий репозиторий',
        iconKey: 'folder',
        domain: 'Общее',
        recommendedAgents: ['chief', 'code-agent'],
        recommendedTools: ['filesystem', 'git'],
        recommendedSkills: ['ob2h'],
        isDirectoryEmpty: false,
        hasGit: hasGit,
        hasAgentsMd: hasAgentsMd,
        hasReadme: hasReadme,
        summary: 'Найдено ${files.length} элементов. Агент готов к контекстной работе с кодом.',
      );
    } catch (e) {
      return ProjectAnalysisResult(
        detectedStack: 'Ошибка анализа',
        iconKey: 'folder',
        domain: 'Общее',
        recommendedAgents: ['chief'],
        recommendedTools: ['filesystem'],
        recommendedSkills: ['ob2h'],
        isDirectoryEmpty: false,
        hasGit: false,
        hasAgentsMd: false,
        hasReadme: false,
        summary: 'Не удалось прочитать директорию: $e',
      );
    }
  }
}

/// Options selected by the user in the Project Wizard.
class ProjectScaffoldOptions {
  final bool initGit;
  final bool createAgentsMd;
  final bool createReadme;
  final bool runOb2hScan;
  final String? customInstructions;
  final String colorHex;
  final String iconName;

  const ProjectScaffoldOptions({
    this.initGit = true,
    this.createAgentsMd = true,
    this.createReadme = true,
    this.runOb2hScan = true,
    this.customInstructions,
    this.colorHex = '#00D2FF',
    this.iconName = 'code',
  });
}

class ProjectScaffoldingResult {
  final bool success;
  final String message;
  final ProjectAnalysisResult analysis;

  const ProjectScaffoldingResult({
    required this.success,
    required this.message,
    required this.analysis,
  });
}

class ProjectScaffoldingService {
  /// Scaffolds or connects a project with intelligent pre-production analysis.
  static Future<ProjectScaffoldingResult> scaffold({
    required String projectPath,
    required String projectName,
    required ProjectScaffoldOptions options,
  }) async {
    final dir = Directory(projectPath);
    if (!dir.existsSync()) {
      dir.createSync(recursive: true);
    }

    final analysis = ProjectAnalyzer.analyze(projectPath);

    // 1. .gitignore (if not present)
    final gitignoreFile = File('$projectPath/.gitignore');
    if (!gitignoreFile.existsSync()) {
      gitignoreFile.writeAsStringSync(_getUniversalGitignore(analysis.detectedStack));
    }

    // 2. README.md (if requested and not present)
    if (options.createReadme) {
      final readmeFile = File('$projectPath/README.md');
      if (!readmeFile.existsSync()) {
        readmeFile.writeAsStringSync(_getReadmeContent(projectName, analysis));
      }
    }

    // 3. AGENTS.md (OmnesAgent AI Rules & Architecture conventions)
    if (options.createAgentsMd) {
      final agentsFile = File('$projectPath/AGENTS.md');
      if (!agentsFile.existsSync()) {
        agentsFile.writeAsStringSync(_getAgentsMdContent(
          projectName: projectName,
          analysis: analysis,
          customInstructions: options.customInstructions,
        ));
      }
    }

    // 4. Git Init (if requested and .git missing)
    if (options.initGit && !analysis.hasGit) {
      try {
        await Process.run('git', ['init'], workingDirectory: projectPath);
      } catch (_) {}
    }

    // 5. OB2H Code Intelligence indexing
    if (options.runOb2hScan) {
      runOb2hScanAsync(projectPath);
    }

    return ProjectScaffoldingResult(
      success: true,
      message: 'Проект готов. Стек: ${analysis.detectedStack}.',
      analysis: analysis,
    );
  }

  static String _getUniversalGitignore(String stack) {
    return '''# OmnesAgent & IDE
.antigravity/
.mimosa/
.codegraph/
*.log
.env
.env.local
.DS_Store
Thumbs.db

# Build artifacts & Dependencies
/target/
node_modules/
dist/
build/
.dart_tool/
__pycache__/
*.py[cod]
.venv/
venv/
.pytest_cache/
.ruff_cache/
bin/
obj/
''';
  }

  static String _getReadmeContent(String name, ProjectAnalysisResult analysis) {
    final timeStr = DateTime.now().toLocal().toString().split('.')[0];
    return '''# $name

Проект подключен в **OmnesAgent**.

- **Стек**: ${analysis.detectedStack}
- **Дата подключения**: $timeStr

## Архитектура и правила
- `AGENTS.md` — архитектурный контекст и правила для ИИ-агентов.
- `README.md` — документация проекта.

## Работа с ИИ
Задавайте вопросы и ставьте задачи агенту прямо в OmnesAgent.
''';
  }

  static String _getAgentsMdContent({
    required String projectName,
    required ProjectAnalysisResult analysis,
    String? customInstructions,
  }) {
    final customSection = (customInstructions != null && customInstructions.trim().isNotEmpty)
        ? '''
## Специальные правила для этого проекта
${customInstructions.trim()}
'''
        : '';

    return '''# AGENTS.md — $projectName

Архитектурный контекст и инструкции для ИИ-ассистентов в репозитории `$projectName`.

## Стек и окружение
- Автоматически определенный стек: **${analysis.detectedStack}**
- Домен проекта: **${analysis.domain}**
- Single Source of Truth: не дублировать состояние и конфигурации.
- Все внешние данные валидируются на входе; ошибки строго обрабатываются.

$customSection

## Правила взаимодействия с кодовой базой
1. **Всегда проверяй кодовую базу** перед внесением правок (используй OB2H для AST-поиска).
2. **Closed-loop testing**: после любых изменений запускай валидацию синтаксиса или тесты.
3. **Безопасность**: никогда не фиксируй секреты, токены и приватные ключи в коде.
4. **Git коммиты**: Conventional Commits на русском языке (`feat(модуль): описание`).
''';
  }

  // --- Native System Integrations ---

  /// Opens the project folder in Windows File Explorer (or Finder / file manager).
  static Future<void> openInExplorer(String projectPath) async {
    try {
      if (Platform.isWindows) {
        await Process.run('explorer.exe', [projectPath]);
      } else if (Platform.isMacOS) {
        await Process.run('open', [projectPath]);
      } else if (Platform.isLinux) {
        await Process.run('xdg-open', [projectPath]);
      }
    } catch (_) {}
  }

  /// Opens the project folder in VS Code or Cursor.
  static Future<bool> openInCode(String projectPath, {bool preferCursor = false}) async {
    try {
      final editorCmd = preferCursor ? 'cursor' : 'code';
      final res = await Process.run(editorCmd, [projectPath], runInShell: true);
      if (res.exitCode == 0) return true;
      if (preferCursor) {
        final fallback = await Process.run('code', [projectPath], runInShell: true);
        return fallback.exitCode == 0;
      }
      return false;
    } catch (_) {
      return false;
    }
  }

  /// Opens a terminal window at the project root.
  static Future<void> openInTerminal(String projectPath) async {
    try {
      if (Platform.isWindows) {
        final wtRes = await Process.run('wt.exe', ['-d', projectPath], runInShell: true);
        if (wtRes.exitCode != 0) {
          await Process.start('powershell.exe', ['-NoExit', '-Command', 'Set-Location -LiteralPath "$projectPath"']);
        }
      } else if (Platform.isMacOS) {
        await Process.run('open', ['-a', 'Terminal', projectPath]);
      } else if (Platform.isLinux) {
        await Process.run('x-terminal-emulator', ['--working-directory=$projectPath']);
      }
    } catch (_) {}
  }

  /// Triggers asynchronous AST-indexing of the project via OB2H.
  static void runOb2hScanAsync(String projectPath) {
    try {
      final ob2hPaths = [
        'ob2h',
        r'C:\Users\ipres\.cargo\bin\ob2h.exe',
        r'C:\Projects\omnesbot_for_hermes\target\release\ob2h.exe',
      ];

      for (final bin in ob2hPaths) {
        try {
          Process.start(
            bin,
            ['project', 'scan', projectPath],
            environment: {'OB2H_DATA_DIR': r'C:\Projects\omnesbot_for_hermes\data'},
            runInShell: true,
          );
          break;
        } catch (_) {}
      }
    } catch (_) {}
  }
}
