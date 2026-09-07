// Screen for viewing and managing automated cron schedules.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'cron_controller.dart';
import 'models/cron_job.dart';

class CronScreen extends StatelessWidget {
  const CronScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(CronController());

    return Scaffold(
      appBar: AppBar(
        title: const Text('Расписания (Cron)'),
        actions: [
          IconButton(
            tooltip: 'Обновить',
            icon: const Icon(Icons.refresh),
            onPressed: controller.loadJobs,
          ),
        ],
      ),
      body: Obx(() {
        if (controller.isLoading.value && controller.jobs.isEmpty) {
          return const Center(child: CircularProgressIndicator());
        }

        if (controller.errorMessage.isNotEmpty) {
          return Center(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(Icons.error_outline, size: 48, color: Colors.red),
                  const SizedBox(height: 8),
                  Text(controller.errorMessage.value, textAlign: TextAlign.center),
                  const SizedBox(height: 12),
                  ElevatedButton(
                    onPressed: controller.loadJobs,
                    child: const Text('Повторить'),
                  ),
                ],
              ),
            ),
          );
        }

        if (controller.jobs.isEmpty) {
          return Center(
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(Icons.schedule_outlined, size: 64, color: Colors.grey.shade400),
                  const SizedBox(height: 16),
                  const Text(
                    'Нет активных расписаний',
                    style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Запланируйте периодические действия агента: утренние сводки, резервные копии, мониторинг.',
                    textAlign: TextAlign.center,
                    style: TextStyle(color: Colors.grey.shade600),
                  ),
                  const SizedBox(height: 20),
                  ElevatedButton.icon(
                    onPressed: () => _showCreateJobDialog(context, controller),
                    icon: const Icon(Icons.add),
                    label: const Text('Создать задачу'),
                  ),
                ],
              ),
            ),
          );
        }

        return RefreshIndicator(
          onRefresh: controller.loadJobs,
          child: ListView.builder(
            padding: const EdgeInsets.all(12),
            itemCount: controller.jobs.length,
            itemBuilder: (context, index) {
              final job = controller.jobs[index];
              return _buildCronCard(context, controller, job);
            },
          ),
        );
      }),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _showCreateJobDialog(context, controller),
        icon: const Icon(Icons.add_alarm),
        label: const Text('Расписание'),
      ),
    );
  }

  Widget _buildCronCard(BuildContext context, CronController controller, CronJob job) {
    return Card(
      elevation: 1,
      margin: const EdgeInsets.only(bottom: 12),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  decoration: BoxDecoration(
                    color: Colors.blue.withOpacity(0.12),
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const Icon(Icons.alarm, size: 14, color: Colors.blue),
                      const SizedBox(width: 4),
                      Text(
                        job.expression.isNotEmpty ? job.expression : 'cron',
                        style: const TextStyle(
                          fontFamily: 'monospace',
                          fontWeight: FontWeight.bold,
                          fontSize: 12,
                          color: Colors.blue,
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 8),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.grey.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    job.agentAlias,
                    style: const TextStyle(fontSize: 11, color: Colors.grey),
                  ),
                ),
                const Spacer(),
                Switch(
                  value: job.enabled,
                  onChanged: (val) => controller.toggleJob(job, val),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              job.displayName,
              style: const TextStyle(fontSize: 15, fontWeight: FontWeight.bold),
            ),
            if (job.prompt != null && job.prompt!.isNotEmpty && job.name.isNotEmpty) ...[
              const SizedBox(height: 4),
              Text(
                job.prompt!,
                style: TextStyle(color: Colors.grey.shade600, fontSize: 13),
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ],
            const SizedBox(height: 10),
            const Divider(height: 1),
            const SizedBox(height: 8),
            Row(
              children: [
                if (job.lastStatus != null) ...[
                  Icon(
                    job.lastStatus == 'ok' || job.lastStatus == 'success'
                        ? Icons.check_circle_outline
                        : Icons.error_outline,
                    size: 14,
                    color: job.lastStatus == 'ok' || job.lastStatus == 'success'
                        ? Colors.green
                        : Colors.red,
                  ),
                  const SizedBox(width: 4),
                  Text(
                    'Статус: ${job.lastStatus}',
                    style: TextStyle(fontSize: 11, color: Colors.grey.shade600),
                  ),
                  const Spacer(),
                ] else
                  const Spacer(),
                TextButton.icon(
                  onPressed: () => controller.runNow(job),
                  icon: const Icon(Icons.play_arrow, size: 16),
                  label: const Text('Запустить сейчас'),
                ),
                IconButton(
                  icon: const Icon(Icons.delete_outline, size: 18, color: Colors.grey),
                  tooltip: 'Удалить',
                  onPressed: () => controller.deleteJob(job),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  void _showCreateJobDialog(BuildContext context, CronController controller) {
    final nameCtrl = TextEditingController();
    final promptCtrl = TextEditingController();
    final cronCtrl = TextEditingController(text: '0 9 * * *');

    final presets = [
      {'label': 'Каждый день в 9:00', 'cron': '0 9 * * *'},
      {'label': 'Каждый час', 'cron': '0 * * * *'},
      {'label': 'Каждый понедельник в 9:00', 'cron': '0 9 * * 1'},
      {'label': 'Каждые 30 минут', 'cron': '*/30 * * * *'},
    ];

    Get.dialog(
      StatefulBuilder(
        builder: (context, setDialogState) {
          return AlertDialog(
            title: const Text('Новая авто-задача'),
            content: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TextField(
                    controller: nameCtrl,
                    autofocus: true,
                    decoration: const InputDecoration(
                      labelText: 'Название расписания',
                      hintText: 'например: Утренняя сводка новостей',
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: promptCtrl,
                    maxLines: 2,
                    decoration: const InputDecoration(
                      labelText: 'Инструкция для агента (промпт)',
                      hintText: 'например: Собери сводку новостей и запиши в NOTES.md',
                    ),
                  ),
                  const SizedBox(height: 14),
                  const Text('Пресеты расписания:', style: TextStyle(fontSize: 12, color: Colors.grey)),
                  const SizedBox(height: 6),
                  Wrap(
                    spacing: 6,
                    runSpacing: 4,
                    children: presets.map((p) {
                      return ChoiceChip(
                        label: Text(p['label']!, style: const TextStyle(fontSize: 11)),
                        selected: cronCtrl.text == p['cron'],
                        onSelected: (selected) {
                          if (selected) {
                            setDialogState(() => cronCtrl.text = p['cron']!);
                          }
                        },
                      );
                    }).toList(),
                  ),
                  const SizedBox(height: 10),
                  TextField(
                    controller: cronCtrl,
                    decoration: const InputDecoration(
                      labelText: 'Cron-выражение (мин час день мес день_нед)',
                      hintText: '0 9 * * *',
                    ),
                    style: const TextStyle(fontFamily: 'monospace', fontWeight: FontWeight.bold),
                  ),
                ],
              ),
            ),
            actions: [
              TextButton(onPressed: () => Get.back(), child: const Text('Отмена')),
              ElevatedButton(
                onPressed: () async {
                  final prompt = promptCtrl.text.trim();
                  final schedule = cronCtrl.text.trim();
                  final name = nameCtrl.text.trim();
                  if (prompt.isNotEmpty && schedule.isNotEmpty) {
                    Get.back();
                    await controller.createJob(
                      name: name.isNotEmpty ? name : prompt,
                      schedule: schedule,
                      prompt: prompt,
                    );
                  }
                },
                child: const Text('Создать'),
              ),
            ],
          );
        },
      ),
    );
  }
}
