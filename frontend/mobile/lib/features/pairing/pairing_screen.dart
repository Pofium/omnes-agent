import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';

import '../../design_system/design_system.dart';
import 'pairing_controller.dart';

class PairingScreen extends StatelessWidget {
  const PairingScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(PairingController());

    return Scaffold(
      backgroundColor: ShadcnColors.background,
      appBar: AppBar(
        title: const Text('Сопряжение с шлюзом'),
      ),
      body: SafeArea(
        child: Center(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 440),
              child: ShadcnCard(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Container(
                      width: 64,
                      height: 64,
                      decoration: BoxDecoration(
                        color: ShadcnColors.primary.withOpacity(0.12),
                        shape: BoxShape.circle,
                        border: Border.all(color: ShadcnColors.primary.withOpacity(0.4)),
                      ),
                      child: const Center(
                        child: Icon(
                          FontAwesomeIcons.qrcode,
                          color: ShadcnColors.primary,
                          size: 28,
                        ),
                      ),
                    ),
                    const SizedBox(height: 18),
                    const Text(
                      'Сопряжение устройства',
                      style: TextStyle(
                        color: ShadcnColors.foreground,
                        fontSize: 20,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    const SizedBox(height: 8),
                    const Text(
                      'Введите код сопряжения, отображаемый в консоли сервера или сгенерированный командой «omnesagent pair»',
                      textAlign: TextAlign.center,
                      style: TextStyle(
                        color: ShadcnColors.foregroundMuted,
                        fontSize: 13,
                        height: 1.4,
                      ),
                    ),
                    const SizedBox(height: 24),
                    ShadcnInput(
                      controller: controller.codeController,
                      hintText: 'XXXX-XXXX или 6 цифр',
                      labelText: 'КОД СОПРЯЖЕНИЯ',
                      keyboardType: TextInputType.text,
                      autofocus: true,
                    ),
                    const SizedBox(height: 12),
                    Obx(() {
                      if (controller.errorMessage.value.isEmpty) {
                        return const SizedBox.shrink();
                      }
                      return Padding(
                        padding: const EdgeInsets.only(bottom: 12),
                        child: Row(
                          children: [
                            const Icon(Icons.error_outline, color: ShadcnColors.destructive, size: 16),
                            const SizedBox(width: 6),
                            Expanded(
                              child: Text(
                                controller.errorMessage.value,
                                style: const TextStyle(color: ShadcnColors.destructive, fontSize: 12),
                              ),
                            ),
                          ],
                        ),
                      );
                    }),
                    const SizedBox(height: 8),
                    Obx(() => ShadcnButton(
                          label: 'Подтвердить и подключить',
                          fullWidth: true,
                          isLoading: controller.isLoading.value,
                          onPressed: () => controller.submitPairing(),
                        )),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
