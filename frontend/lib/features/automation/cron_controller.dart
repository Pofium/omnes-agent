// Controller managing state for scheduled automation tasks (Cron).

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../core/gateway/gateway_config.dart';
import 'models/cron_job.dart';
import 'cron_repository.dart';

class CronController extends GetxController {
  final CronRepository _repo;

  CronController({CronRepository? repo}) : _repo = repo ?? CronRepository();

  final RxList<CronJob> jobs = <CronJob>[].obs;
  final RxBool isLoading = false.obs;
  final RxString errorMessage = ''.obs;
  final RxString activeAgent = ''.obs;

  @override
  void onInit() {
    super.onInit();
    loadJobs();
  }

  /// Loads all cron jobs.
  Future<void> loadJobs() async {
    isLoading.value = true;
    errorMessage.value = '';
    try {
      activeAgent.value = await GatewayConfig.getActiveAgent();
      final list = await _repo.listJobs();
      jobs.assignAll(list);
    } catch (e) {
      errorMessage.value = 'Ошибка загрузки расписаний: $e';
    } finally {
      isLoading.value = false;
    }
  }

  /// Toggles job enabled state.
  Future<void> toggleJob(CronJob job, bool enabled) async {
    final idx = jobs.indexWhere((j) => j.id == job.id);
    if (idx != -1) {
      jobs[idx] = job.copyWith(enabled: enabled);
    }
    final success = await _repo.toggleJob(job.id, enabled);
    if (!success && idx != -1) {
      // Revert if failed
      jobs[idx] = job.copyWith(enabled: !enabled);
      Get.snackbar('Ошибка', 'Не удалось изменить статус расписания', snackPosition: SnackPosition.BOTTOM);
    }
  }

  /// Runs job manually right now.
  Future<void> runNow(CronJob job) async {
    Get.snackbar(
      'Запуск задачи',
      'Задача "${job.displayName}" отправлена на выполнение...',
      snackPosition: SnackPosition.BOTTOM,
      duration: const Duration(seconds: 2),
    );
    final success = await _repo.runJobNow(job.id);
    if (success) {
      Get.snackbar(
        'Успех',
        'Задача "${job.displayName}" запущена на шлюзе',
        snackPosition: SnackPosition.BOTTOM,
        backgroundColor: Colors.green.shade800,
        colorText: Colors.white,
      );
      await loadJobs();
    } else {
      Get.snackbar('Ошибка', 'Не удалось запустить задачу', snackPosition: SnackPosition.BOTTOM);
    }
  }

  /// Creates a new prompt-based cron job.
  Future<bool> createJob({
    required String name,
    required String schedule,
    required String prompt,
  }) async {
    if (schedule.trim().isEmpty || prompt.trim().isEmpty) return false;
    isLoading.value = true;
    try {
      final success = await _repo.createPromptJob(
        name: name.trim(),
        schedule: schedule.trim(),
        prompt: prompt.trim(),
        agentAlias: activeAgent.value,
      );
      if (success) {
        await loadJobs();
        Get.snackbar(
          'Расписание создано',
          'Задача успешно добавлена в cron',
          snackPosition: SnackPosition.BOTTOM,
          backgroundColor: Colors.green.shade800,
          colorText: Colors.white,
        );
        return true;
      }
    } catch (_) {} finally {
      isLoading.value = false;
    }
    return false;
  }

  /// Deletes a cron job.
  Future<void> deleteJob(CronJob job) async {
    final confirmed = await Get.dialog<bool>(
      AlertDialog(
        title: const Text('Удалить расписание?'),
        content: Text('Удалить задачу "${job.displayName}"?'),
        actions: [
          TextButton(onPressed: () => Get.back(result: false), child: const Text('Отмена')),
          ElevatedButton(
            style: ElevatedButton.styleFrom(backgroundColor: Colors.red),
            onPressed: () => Get.back(result: true),
            child: const Text('Удалить', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );

    if (confirmed != true) return;

    try {
      final success = await _repo.deleteJob(job.id);
      if (success) {
        jobs.removeWhere((j) => j.id == job.id);
        Get.snackbar('Удалено', 'Расписание удалено', snackPosition: SnackPosition.BOTTOM);
      }
    } catch (_) {}
  }

  @override
  void onClose() {
    _repo.dispose();
    super.onClose();
  }
}
