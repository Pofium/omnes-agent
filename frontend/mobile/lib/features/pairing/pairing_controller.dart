import 'package:flutter/material.dart';
import 'package:get/get.dart';
import '../../core/gateway/gateway_http.dart';
import '../../routes/routes.dart';

class PairingController extends GetxController {
  final GatewayHttpClient _http = GatewayHttpClient();

  final TextEditingController codeController = TextEditingController();
  final RxBool isLoading = false.obs;
  final RxString errorMessage = ''.obs;

  Future<void> submitPairing() async {
    final code = codeController.text.trim();
    if (code.isEmpty) {
      errorMessage.value = 'Введите 6-значный код сопряжения';
      return;
    }

    isLoading.value = true;
    errorMessage.value = '';

    try {
      final token = await _http.pairDevice(code);
      if (token != null && token.isNotEmpty) {
        Get.snackbar(
          'Успешное сопряжение',
          'Устройство авторизовано на сервере Omnes',
          snackPosition: SnackPosition.BOTTOM,
          backgroundColor: const Color(0xFF10B981),
          colorText: Colors.white,
        );
        Get.offAllNamed(Routes.homeScreen);
      } else {
        errorMessage.value = 'Неверный код или истекло время действия';
      }
    } catch (e) {
      errorMessage.value = 'Ошибка связи со шлюзом: $e';
    } finally {
      isLoading.value = false;
    }
  }

  @override
  void onClose() {
    codeController.dispose();
    _http.dispose();
    super.onClose();
  }
}
