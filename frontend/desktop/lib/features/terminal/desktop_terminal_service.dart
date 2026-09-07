// Real bidirectional Terminal Service for OmnesAgent Desktop ADE on Windows.
// Spawns powershell.exe / cmd.exe processes with streaming stdout/stderr and stdin writing.

import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:get/get.dart';

class TerminalSession {
  final String id;
  final String title;
  final Process process;
  final List<String> lines = [];
  final StreamController<String> outputController = StreamController<String>.broadcast();

  TerminalSession({
    required this.id,
    required this.title,
    required this.process,
  });

  void appendLine(String line) {
    lines.add(line);
    if (lines.length > 2000) {
      lines.removeAt(0);
    }
    outputController.add(line);
  }

  void writeInput(String input) {
    process.stdin.writeln(input);
  }

  void kill() {
    process.kill(ProcessSignal.sigint);
  }

  void dispose() {
    kill();
    outputController.close();
  }
}

class DesktopTerminalService extends GetxController {
  static DesktopTerminalService get to => Get.isRegistered<DesktopTerminalService>()
      ? Get.find<DesktopTerminalService>()
      : Get.put(DesktopTerminalService());

  final sessions = <TerminalSession>[].obs;
  final activeSessionIndex = 0.obs;

  TerminalSession? get activeSession =>
      sessions.isNotEmpty && activeSessionIndex.value < sessions.length
          ? sessions[activeSessionIndex.value]
          : null;

  @override
  void onInit() {
    super.onInit();
    startNewSession(title: '1: PowerShell');
  }

  /// Spawns a real shell process.
  Future<TerminalSession?> startNewSession({String? title, String? workingDir}) async {
    try {
      final shell = Platform.isWindows ? 'powershell.exe' : 'bash';
      final List<String> args = Platform.isWindows ? <String>['-NoLogo'] : <String>[];
      final id = DateTime.now().millisecondsSinceEpoch.toString();
      final sessionTitle = title ?? '${sessions.length + 1}: $shell';

      final process = await Process.start(
        shell,
        args,
        workingDirectory: workingDir ?? Directory.current.path,
        runInShell: true,
      );

      final session = TerminalSession(
        id: id,
        title: sessionTitle,
        process: process,
      );

      // Listen to stdout
      process.stdout.transform(utf8.decoder).transform(const LineSplitter()).listen(
        (line) => session.appendLine(line),
        onError: (err) => session.appendLine('[STDOUT ERROR] $err'),
      );

      // Listen to stderr
      process.stderr.transform(utf8.decoder).transform(const LineSplitter()).listen(
        (line) => session.appendLine('[STDERR] $line'),
        onError: (err) => session.appendLine('[STDERR ERROR] $err'),
      );

      process.exitCode.then((code) {
        session.appendLine('[Process exited with code $code]');
      });

      session.appendLine('OmnesAgent ADE Terminal Engine connected to $shell [PID: ${process.pid}]');
      session.appendLine('Working Directory: ${workingDir ?? Directory.current.path}');
      session.appendLine('---');

      sessions.add(session);
      activeSessionIndex.value = sessions.length - 1;
      return session;
    } catch (e) {
      Get.snackbar('Ошибка запуска терминала', '$e');
      return null;
    }
  }

  /// Sends a command line to the active session stdin.
  void sendCommand(String cmd) {
    final session = activeSession;
    if (session == null) return;
    session.appendLine('> $cmd');
    session.writeInput(cmd);
  }

  /// Interrupts the active process (Ctrl+C).
  void interruptActiveSession() {
    activeSession?.kill();
    activeSession?.appendLine('^C [Process interrupted]');
  }

  /// Clears lines in active session.
  void clearActiveSession() {
    activeSession?.lines.clear();
  }

  /// Closes session.
  void closeSession(int index) {
    if (index >= 0 && index < sessions.length) {
      final s = sessions.removeAt(index);
      s.dispose();
      if (activeSessionIndex.value >= sessions.length) {
        activeSessionIndex.value = (sessions.length - 1).clamp(0, 999);
      }
    }
  }

  @override
  void onClose() {
    for (final s in sessions) {
      s.dispose();
    }
    sessions.clear();
    super.onClose();
  }
}
