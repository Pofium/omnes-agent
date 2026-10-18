import 'dart:async';
import 'dart:convert';
import 'package:get/get.dart';
import 'package:universal_io/io.dart' as universal_io;

/// Manages embedded terminal process execution, stdout/stderr streams, and interactive stdin.
class WorkspaceTerminalController {
  final isTerminalOpen = false.obs;
  final terminalLines = <String>[
    'OmnesAgent ADE Terminal ready. Enter a command below:',
  ].obs;
  final isTerminalRunning = false.obs;

  universal_io.Process? _activeTerminalProcess;
  StreamSubscription? _termStdoutSub;
  StreamSubscription? _termStderrSub;

  final String Function()? getWorkingDirectory;
  final void Function(bool isOpen)? onTerminalToggled;

  WorkspaceTerminalController({
    this.getWorkingDirectory,
    this.onTerminalToggled,
  });

  /// Toggles visibility of the terminal panel.
  void toggleTerminal() {
    isTerminalOpen.value = !isTerminalOpen.value;
    onTerminalToggled?.call(isTerminalOpen.value);
  }

  /// Sets terminal open state directly.
  void setTerminalOpen(bool open) {
    isTerminalOpen.value = open;
  }

  /// Executes an arbitrary shell command in the workspace terminal.
  Future<void> executeTerminalCommand(String rawCmd) async {
    final cmd = rawCmd.trim();
    if (cmd.isEmpty) return;

    // Route input to live stdin if a process is already running
    if (_activeTerminalProcess != null && isTerminalRunning.value) {
      sendTerminalStdin(cmd);
      return;
    }

    terminalLines.add('> $cmd');
    isTerminalRunning.value = true;
    try {
      final workDir = getWorkingDirectory?.call() ?? universal_io.Directory.current.path;

      if (cmd == 'cls' || cmd == 'clear') {
        clearTerminal();
        return;
      }

      try {
        final isWin = universal_io.Platform.isWindows;
        final exe = isWin ? 'powershell.exe' : 'bash';
        final args = isWin ? ['-NoLogo', '-Command', cmd] : ['-c', cmd];

        final process = await universal_io.Process.start(
          exe,
          args,
          workingDirectory: workDir,
          runInShell: true,
        );
        _activeTerminalProcess = process;

        _termStdoutSub?.cancel();
        _termStdoutSub = process.stdout
            .transform(utf8.decoder)
            .transform(const LineSplitter())
            .listen((line) {
          if (line.isNotEmpty) {
            terminalLines.add(line);
          }
        });

        _termStderrSub?.cancel();
        _termStderrSub = process.stderr
            .transform(utf8.decoder)
            .transform(const LineSplitter())
            .listen((line) {
          if (line.isNotEmpty) {
            terminalLines.add('[STDERR] $line');
          }
        });

        process.exitCode.then((code) {
          if (_activeTerminalProcess == process) {
            _activeTerminalProcess = null;
            isTerminalRunning.value = false;
            if (code != 0) {
              terminalLines.add('[Код завершения: $code]');
            }
          }
        }).catchError((_) {
          if (_activeTerminalProcess == process) {
            _activeTerminalProcess = null;
            isTerminalRunning.value = false;
          }
        });
      } catch (procErr) {
        terminalLines.add('[Terminal] Команда обработана: $cmd ($procErr)');
        isTerminalRunning.value = false;
        _activeTerminalProcess = null;
      }
    } catch (e) {
      terminalLines.add('[ОШИБКА] $e');
      isTerminalRunning.value = false;
      _activeTerminalProcess = null;
    }
  }

  /// Sends stdin input to the active terminal process or executes command
  void sendTerminalStdin(String input) {
    if (input == '^C' || input == 'Ctrl+C' || input == 'SIGINT') {
      terminalLines.add('^C');
      if (_activeTerminalProcess != null) {
        try {
          _activeTerminalProcess!.kill(universal_io.ProcessSignal.sigint);
        } catch (_) {
          _activeTerminalProcess!.kill();
        }
        _activeTerminalProcess = null;
        isTerminalRunning.value = false;
      }
      return;
    }

    if (_activeTerminalProcess != null && isTerminalRunning.value) {
      if (input.isNotEmpty) {
        terminalLines.add(input);
      }
      try {
        _activeTerminalProcess!.stdin.writeln(input);
      } catch (e) {
        terminalLines.add('[Ошибка отправки stdin: $e]');
      }
    } else {
      executeTerminalCommand(input);
    }
  }

  /// Clears terminal history and terminates any active child process.
  void clearTerminal() {
    if (_activeTerminalProcess != null) {
      try {
        _activeTerminalProcess!.kill();
      } catch (_) {}
      _activeTerminalProcess = null;
    }
    isTerminalRunning.value = false;
    terminalLines.clear();
    terminalLines.add('OmnesAgent ADE Terminal очищен.');
  }

  /// Disposes background streams and processes on controller destroy.
  void dispose() {
    _termStdoutSub?.cancel();
    _termStderrSub?.cancel();
    if (_activeTerminalProcess != null) {
      try {
        _activeTerminalProcess!.kill();
      } catch (_) {}
      _activeTerminalProcess = null;
    }
  }
}
