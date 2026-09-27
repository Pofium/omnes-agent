// Editor session client: wires the buffer controller to the gateway
// (REST open + `/ws/editor/{buffer_id}` frames).
// FRONTEND_SPEC.md §5.5 (editor_client.dart): подписка, отправка edit_ops /
// cursor / folds, буферизация при реконнекте, rows_snapshot при
// рассинхронизации. Folds arrive with F1; cursor relay with F7.

import 'dart:async';

import 'package:omnes_shared/omnes_shared.dart';

import 'editor_controller.dart';

class EditorClient {
  final EditorBufferController controller;
  final EditorApi api;

  EditorWsClient? _ws;
  StreamSubscription<EditorFrame>? _frameSub;
  StreamSubscription<EditorEditOp>? _opSub;
  int _opSeq = 0;

  /// Optimistically sent, not-yet-confirmed edits as (opId, baseRev) pairs.
  /// Each pending op bumps the gateway revision by exactly one, so the base
  /// revision of the next op is predicted from the queue length; a frame
  /// carrying revision R confirms every op with `baseRev + 1 <= R`.
  final List<(int, int)> _pendingOps = <(int, int)>[];

  /// Result of the last open call (kept for the REST save fallback).
  EditorOpenResult? _openResult;

  EditorClient({required this.controller, EditorApi? api}) : api = api ?? EditorApi();

  /// Opens the buffer: REST open → controller state → WS attach.
  Future<void> open({required String path, String? root}) async {
    controller.status = EditorConnectionStatus.connecting;
    controller.lastError = null;
    controller.notifyChanged();
    try {
      final result = await api.openFile(path: path, root: root);
      _openResult = result;
      controller.loadFromOpen(result);
    } on EditorApiException catch (e) {
      controller.status = EditorConnectionStatus.offline;
      controller.setError(e.message);
      return;
    } catch (e) {
      controller.status = EditorConnectionStatus.offline;
      controller.setError('$e');
      return;
    }

    await _attachWs();
  }

  Future<void> _attachWs() async {
    final bufferId = controller.bufferId;
    if (bufferId == null) return;
    _frameSub?.cancel();
    _ws?.dispose();
    final ws = EditorWsClient(bufferId: bufferId);
    _ws = ws;
    _frameSub = ws.stream.listen(_onFrame);
    _opSub ??= controller.opStream.listen(_onLocalOp);
    await ws.connect();
    if (!ws.isConnected) {
      controller.markOffline();
    }
  }

  void _onLocalOp(EditorEditOp op) {
    final ws = _ws;
    if (ws == null || !ws.isConnected) {
      // Offline: keep the optimistic state; the resync after reconnect
      // reconciles rows and revision (правки не уходят «в никуда» —
      // FRONTEND_SPEC §5.5).
      return;
    }
    final base = controller.rev + _pendingOps.length;
    final opId = ++_opSeq;
    _pendingOps.add((opId, base));
    ws.send(editorEditOpsFrame(baseRev: base, opId: opId, ops: [op]));
  }

  void _onFrame(EditorFrame frame) {
    switch (frame.type) {
      case 'hello':
        controller.applyHello(frame);
        // Ensure the viewport rows match the authoritative revision.
        if (frame.rev != null && frame.rev != controller.rev) {
          requestFullResync();
        }
      case 'rows_snapshot':
        controller.applySnapshot(frame);
      case 'rows_changed':
        controller.applyRowsChanged(frame);
        _confirmPendingUpTo(frame.rev);
      case 'edit_ack':
        if (frame.applied == true) {
          _confirmPendingUpTo(frame.rev);
        } else {
          // Stale base (concurrent edit) — resync from the idempotent
          // snapshot; unconfirmed ops are dropped (§9.9 инвариант 1).
          _pendingOps.clear();
          requestFullResync();
        }
      case 'save_state':
        controller.applySaveState(frame);
      case 'error':
        if (frame.code == 'buffer_closed') {
          controller.markClosed();
        } else {
          controller.setError(frame.message ?? frame.code ?? 'unknown error');
        }
      default:
        break;
    }
  }

  void _confirmPendingUpTo(int? rev) {
    if (rev == null) return;
    if (rev > controller.rev) controller.rev = rev;
    // A confirming revision R settles every op whose applied revision
    // (baseRev + 1) is <= R — usually exactly one per frame.
    while (_pendingOps.isNotEmpty && rev >= _pendingOps.first.$2 + 1) {
      _pendingOps.removeAt(0);
    }
    controller.notifyChanged();
  }

  /// Idempotent full-document snapshot request (BACKEND_SPEC §9.9, inv. 3).
  void requestFullResync() {
    _ws?.send(editorRowsRequestFrame(from: 0, count: 1 << 20));
  }

  /// Save through the WS channel (server writes the buffer text preserving
  /// the file's EOL). REST fallback when the socket is down.
  Future<void> save() async {
    final ws = _ws;
    if (ws != null && ws.isConnected) {
      ws.send(editorSaveFrame());
      return;
    }
    final open = _openResult;
    if (open == null) return;
    try {
      await api.writeFile(
        path: open.path,
        content: controller.joinedText(),
        root: _rootHint,
        force: true,
      );
      controller.dirty = false;
      controller.notifyChanged();
    } on EditorApiException catch (e) {
      controller.setError(e.message);
    }
  }

  String? get _rootHint => null;

  Future<void> reconnect() => _attachWs();

  void dispose() {
    _frameSub?.cancel();
    _opSub?.cancel();
    _ws?.dispose();
  }
}
