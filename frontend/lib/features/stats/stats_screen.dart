// Screen displaying token consumption and cost metrics from /api/cost.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'models/cost_summary.dart';
import 'stats_controller.dart';

class StatsScreen extends StatelessWidget {
  const StatsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(StatsController());

    return Scaffold(
      appBar: AppBar(
        title: const Text('Стоимость и токены'),
        actions: [
          IconButton(
            tooltip: 'Обновить',
            icon: const Icon(Icons.refresh),
            onPressed: controller.loadStats,
          ),
        ],
      ),
      body: Column(
        children: [
          _buildTimeframeFilter(context, controller),
          const Divider(height: 1),
          Expanded(
            child: Obx(() {
              if (controller.isLoading.value && controller.costSummary.value == null) {
                return const Center(child: CircularProgressIndicator());
              }

              final summary = controller.costSummary.value;
              if (summary == null || (summary.totalTokens == 0 && summary.byModel.isEmpty)) {
                return Center(
                  child: Padding(
                    padding: const EdgeInsets.all(24),
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(Icons.bar_chart_outlined, size: 64, color: Colors.grey.shade400),
                        const SizedBox(height: 16),
                        const Text(
                          'Нет данных о расходах',
                          style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
                        ),
                        const SizedBox(height: 8),
                        Text(
                          'Запросы к LLM-провайдерам будут автоматически учитываться здесь.',
                          textAlign: TextAlign.center,
                          style: TextStyle(color: Colors.grey.shade600),
                        ),
                      ],
                    ),
                  ),
                );
              }

              return RefreshIndicator(
                onRefresh: controller.loadStats,
                child: ListView(
                  padding: const EdgeInsets.all(16),
                  children: [
                    _buildOverviewCards(context, summary),
                    const SizedBox(height: 20),
                    _buildModelsSection(context, summary),
                  ],
                ),
              );
            }),
          ),
        ],
      ),
    );
  }

  Widget _buildTimeframeFilter(BuildContext context, StatsController controller) {
    final timeframes = [
      {'id': 'all', 'label': 'За всё время'},
      {'id': 'today', 'label': 'Сегодня'},
      {'id': 'week', 'label': '7 дней'},
      {'id': 'month', 'label': '30 дней'},
    ];

    return Obx(() {
      final selected = controller.selectedTimeframe.value;
      return SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: Row(
          children: timeframes.map((tf) {
            final isSelected = selected == tf['id'];
            return Padding(
              padding: const EdgeInsets.only(right: 8),
              child: ChoiceChip(
                label: Text(tf['label']!),
                selected: isSelected,
                onSelected: (_) => controller.setTimeframe(tf['id']!),
              ),
            );
          }).toList(),
        ),
      );
    });
  }

  Widget _buildOverviewCards(BuildContext context, CostSummary summary) {
    return Row(
      children: [
        Expanded(
          child: _metricCard(
            context,
            title: 'Всего токенов',
            value: _formatNumber(summary.totalTokens),
            icon: Icons.token,
            color: Colors.blue,
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: _metricCard(
            context,
            title: 'Расход USD',
            value: '\$${summary.totalCostUsd.toStringAsFixed(4)}',
            icon: Icons.attach_money,
            color: Colors.green,
          ),
        ),
      ],
    );
  }

  Widget _metricCard(
    BuildContext context, {
    required String title,
    required String value,
    required IconData icon,
    required Color color,
  }) {
    return Card(
      elevation: 1,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(icon, size: 20, color: color),
                const SizedBox(width: 8),
                Text(
                  title,
                  style: TextStyle(fontSize: 12, color: Colors.grey.shade600),
                ),
              ],
            ),
            const SizedBox(height: 10),
            Text(
              value,
              style: const TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildModelsSection(BuildContext context, CostSummary summary) {
    if (summary.byModel.isEmpty) return const SizedBox.shrink();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Расход по моделям',
          style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 10),
        ...summary.byModel.entries.map((entry) {
          final m = entry.value;
          return Card(
            elevation: 1,
            margin: const EdgeInsets.only(bottom: 8),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
            child: Padding(
              padding: const EdgeInsets.all(14),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.smart_toy_outlined, size: 18, color: Colors.indigo),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          m.model.isNotEmpty ? m.model : entry.key,
                          style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 14),
                        ),
                      ),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: Colors.indigo.withOpacity(0.1),
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Text(
                          '${m.requestCount} запр.',
                          style: const TextStyle(fontSize: 11, color: Colors.indigo),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        'Вход: ${_formatNumber(m.inputTokens)}',
                        style: TextStyle(fontSize: 12, color: Colors.grey.shade600),
                      ),
                      Text(
                        'Выход: ${_formatNumber(m.outputTokens)}',
                        style: TextStyle(fontSize: 12, color: Colors.grey.shade600),
                      ),
                      Text(
                        'Всего: ${_formatNumber(m.totalTokens)}',
                        style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          );
        }),
      ],
    );
  }

  String _formatNumber(int number) {
    if (number >= 1000000) {
      return '${(number / 1000000).toStringAsFixed(1)}M';
    }
    if (number >= 1000) {
      return '${(number / 1000).toStringAsFixed(1)}K';
    }
    return number.toString();
  }
}
