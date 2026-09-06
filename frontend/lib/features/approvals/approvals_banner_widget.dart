// Widget displaying an alert banner when human approval is needed for an agent action.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'approvals_controller.dart';
import 'models/pending_approval.dart';

class ApprovalsBannerWidget extends StatelessWidget {
  const ApprovalsBannerWidget({super.key});

  @override
  Widget build(BuildContext context) {
    // Put or find controller
    final controller = Get.isRegistered<ApprovalsController>()
        ? Get.find<ApprovalsController>()
        : Get.put(ApprovalsController());

    return Obx(() {
      if (controller.pending.isEmpty) {
        return const SizedBox.shrink();
      }

      return Container(
        width: double.infinity,
        color: Colors.amber.shade900,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: controller.pending.map((item) => _buildBannerItem(context, controller, item)).toList(),
        ),
      );
    });
  }

  Widget _buildBannerItem(
    BuildContext context,
    ApprovalsController controller,
    PendingApproval item,
  ) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      child: Row(
        children: [
          const Icon(Icons.warning_amber_rounded, color: Colors.white, size: 24),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Требуется подтверждение действия',
                  style: TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 13),
                ),
                Text(
                  '${item.sopName} (шаг ${item.step}/${item.totalSteps})',
                  style: TextStyle(color: Colors.amber.shade100, fontSize: 12),
                ),
              ],
            ),
          ),
          TextButton(
            style: TextButton.styleFrom(
              backgroundColor: Colors.white.withOpacity(0.2),
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            ),
            onPressed: () => controller.deny(item),
            child: const Text('Отклонить', style: TextStyle(color: Colors.white, fontSize: 12)),
          ),
          const SizedBox(width: 8),
          ElevatedButton(
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.green,
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            ),
            onPressed: () => controller.approve(item),
            child: const Text('Одобрить', style: TextStyle(color: Colors.white, fontSize: 12)),
          ),
        ],
      ),
    );
  }
}
