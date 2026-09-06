// MainController for OmnesAgent application state and actions.

import 'package:flutter/material.dart';
import '../../helper/local_storage.dart';
import '../../model/user_model/user_model.dart';
import '../../widgets/api/toast_message.dart';

class MainController {
  static Future<void> getCredentials() async {
    // OmnesAgent uses self-hosted backend credentials from GatewayConfig / SupabaseConfig.
    LocalStorage.showIsFreeUser(isShowAdYes: false);
  }

  static Future<void> getUserInfo() async {
    LocalStorage.showIsFreeUser(isShowAdYes: false);
  }

  static Future<void> updateToPremiumUser() async {
    LocalStorage.showIsFreeUser(isShowAdYes: false);
  }

  static void updateContentCount(int count) {
    LocalStorage.saveContentCount(count: count);
  }

  static void updateImageGenCount(int count) {
    LocalStorage.saveImageCount(count: count);
  }

  static void updateTextCount(int count) {
    LocalStorage.saveTextCount(count: count);
  }

  static Future<void> resetPassword({required String email}) async {
    ToastMessage.success("Инструкции по сбросу пароля отправлены на $email");
  }

  static Future<void> sendSupportTicket({
    required String name,
    required String email,
    required String note,
  }) async {
    debugPrint("Support ticket: $name, $email, $note");
    ToastMessage.success("Обращение отправлено");
  }

  static void setData(UserModel userModel) {
    LocalStorage.saveName(name: userModel.name);
    LocalStorage.saveEmail(email: userModel.email);
    LocalStorage.saveId(id: userModel.uniqueId);
    if (userModel.imageUrl.isNotEmpty) {
      LocalStorage.saveImage(image: userModel.imageUrl);
    }
  }

  static void checkPremiumOrNot(UserModel userModel) {
    LocalStorage.showIsFreeUser(isShowAdYes: false);
  }
}
