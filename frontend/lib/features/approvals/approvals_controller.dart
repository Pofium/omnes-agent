// Controller for monitoring and responding to human-in-the-loop approval gates.

import 'dart:async';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../core/gateway/gateway_http.dart';
import 'models/pending_approval.dart';

class ApprovalsController extends GetxController {
  final GatewayHttpClient _http;
  Timer? _pollTimer;

  ApprovalsController({GatewayHttpClient? http}) : _http = http ?? GatewayHttpClient();

  final RxList<PendingApproval> pending = <PendingApproval>[].obs;
  final RxBool isResolving = false.obs;

  @override
  void onInit() {
    super.onInit();
    checkPending();
    // Poll every 15 seconds
    _pollTimer = Timer.periodic(const Duration(seconds: 15), (_) => checkPending());
  }

  /// Checks /admin/sop/pending for blocked agent actions.
  Future<void> checkPending() async {
    try {
      final rawList = await _http.sopPending();
      final list = rawList.map((j) => PendingApproval.fromJson(j)).toList();
      pending.assignAll(list);
    } catch (_) {
      // Endpoint may return 400 if SOP subsystem is disabled, silently ignore
    }
  }

  /// Approves a pending action.
  Future<void> approve(PendingApproval item) async {
    isResolving.value = true;
    try {
      final success = await _http.sopApprove(item.runId);
      if (success) {
        pending.removeWhere((p) => p.runId == item.runId);
        Get.snackbar(
          'Одобрено',
          'Действие "${item.sopName}" успешно одобрено',
          snackPosition: SnackPosition.BOTTOM,
          backgroundColor: Colors.green.shade800,
          colorText: Colors.white,
        );
      }
    } catch (_) {} finally {
      isResolving.value = false;
    }
  }

  /// Denies/cancels a pending action.
  Future<void> deny(PendingApproval item) async {
    isResolving.value = true;
    try {
      final success = await _http.sopDeny(item.runId);
      if (success) {
        pending.removeWhere((p) => p.runId == item.runId);
        Get.snackbar(
          'Отклонено',
          'Действие "${item.sopName}" отменено',
          snackPosition: SnackPosition.BOTTOM,
          backgroundColor: Colors.red.shade800,
          colorText: Colors.white,
        );
      }
    } catch (_) {} finally {
      isResolving.value = false;
    }
  }

  @override
  void onClose() {
    _pollTimer?.cancel();
    _http.dispose();
    super.onClose();
  }
}
