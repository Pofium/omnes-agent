// Controller managing the projects list and creation.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../core/gateway/gateway_config.dart';
import 'models/project_model.dart';
import 'project_repository.dart';

class ProjectsController extends GetxController {
  final ProjectRepository _repo;

  ProjectsController({ProjectRepository? repo}) : _repo = repo ?? ProjectRepository();

  final RxList<Project> projects = <Project>[].obs;
  final RxBool isLoading = false.obs;
  final RxString errorMessage = ''.obs;
  final RxString currentAgent = ''.obs;

  @override
  void onInit() {
    super.onInit();
    loadProjects();
  }

  /// Reloads projects for active agent.
  Future<void> loadProjects() async {
    isLoading.value = true;
    errorMessage.value = '';
    try {
      currentAgent.value = await GatewayConfig.getActiveAgent();
      final list = await _repo.listProjects(agentAlias: currentAgent.value);
      projects.assignAll(list);
    } catch (e) {
      errorMessage.value = 'Ошибка загрузки проектов: $e';
    } finally {
      isLoading.value = false;
    }
  }

  /// Creates a new project and refreshes the list.
  Future<Project?> createProject(String name, {String description = ''}) async {
    if (name.trim().isEmpty) return null;
    isLoading.value = true;
    try {
      final p = await _repo.createProject(
        name: name.trim(),
        description: description.trim(),
        agentAlias: currentAgent.value,
      );
      if (p != null) {
        projects.insert(0, p);
        Get.snackbar(
          'Проект создан',
          'Проект "${p.name}" успешно инициализирован',
          snackPosition: SnackPosition.BOTTOM,
          backgroundColor: Colors.green.shade800,
          colorText: Colors.white,
          duration: const Duration(seconds: 2),
        );
      }
      return p;
    } catch (e) {
      Get.snackbar(
        'Ошибка',
        'Не удалось создать проект: $e',
        snackPosition: SnackPosition.BOTTOM,
        backgroundColor: Colors.red.shade800,
        colorText: Colors.white,
      );
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  /// Deletes a project.
  Future<void> deleteProject(Project project) async {
    final confirmed = await Get.dialog<bool>(
      AlertDialog(
        title: const Text('Удалить проект?'),
        content: Text('Папка "projects/${project.id}" со всеми файлами и заметками будет удалена навсегда.'),
        actions: [
          TextButton(
            onPressed: () => Get.back(result: false),
            child: const Text('Отмена'),
          ),
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
      final success = await _repo.deleteProject(project.id, agentAlias: currentAgent.value);
      if (success) {
        projects.removeWhere((p) => p.id == project.id);
        Get.snackbar(
          'Проект удалён',
          'Папка проекта удалена из рабочей директории',
          snackPosition: SnackPosition.BOTTOM,
        );
      } else {
        Get.snackbar(
          'Ошибка',
          'Не удалось удалить проект',
          snackPosition: SnackPosition.BOTTOM,
          backgroundColor: Colors.red.shade800,
          colorText: Colors.white,
        );
      }
    } catch (e) {
      Get.snackbar(
        'Ошибка',
        'Ошибка: $e',
        snackPosition: SnackPosition.BOTTOM,
        backgroundColor: Colors.red.shade800,
        colorText: Colors.white,
      );
    }
  }

  @override
  void onClose() {
    _repo.dispose();
    super.onClose();
  }
}
