import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../utils/custom_color.dart';
import 'agent_controller.dart';

class AgentPickerDialog extends StatelessWidget {
  const AgentPickerDialog({super.key});

  static void show(BuildContext context) {
    showModalBottomSheet(
      context: context,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (_) => const AgentPickerDialog(),
    );
  }

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(AgentController());

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 20),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                "Выбор агента Omnes",
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
              ),
              IconButton(
                icon: const Icon(Icons.close),
                onPressed: () => Get.back(),
              ),
            ],
          ),
          const SizedBox(height: 10),
          Obx(() {
            if (controller.isLoading.value && controller.availableAgents.isEmpty) {
              return const Center(
                child: Padding(
                  padding: EdgeInsets.all(20),
                  child: CircularProgressIndicator(),
                ),
              );
            }

            final agents = controller.availableAgents.isEmpty
                ? ['chief']
                : controller.availableAgents;

            return ListView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              itemCount: agents.length,
              itemBuilder: (context, index) {
                final alias = agents[index];
                final isSelected = controller.currentAgent.value == alias;

                return ListTile(
                  leading: CircleAvatar(
                    backgroundColor: isSelected
                        ? CustomColor.primaryColor
                        : Colors.grey.withOpacity(0.2),
                    child: Icon(
                      Icons.smart_toy_outlined,
                      color: isSelected ? Colors.white : Colors.grey[700],
                    ),
                  ),
                  title: Text(
                    alias,
                    style: TextStyle(
                      fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                    ),
                  ),
                  trailing: isSelected
                      ? const Icon(Icons.check_circle, color: CustomColor.primaryColor)
                      : null,
                  onTap: () {
                    controller.selectAgent(alias);
                    Get.back();
                  },
                );
              },
            );
          }),
        ],
      ),
    );
  }
}
