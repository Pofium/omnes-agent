import 'dart:io';

import 'package:flutter/material.dart';
import 'package:get/get.dart';

import '../../core/supabase/profiles_repository.dart';
import '../../helper/local_storage.dart';
import '../../model/user_model/user_model.dart';
import '../../routes/routes.dart';
import '../../utils/strings.dart';
import '../../widgets/api/toast_message.dart';

class UpdateProfileController extends GetxController {
  late UserModel userModel;

  final userUid = LocalStorage.getId();
  final ProfilesRepository _profilesRepo = ProfilesRepository();

  final _isLoading = false.obs;
  bool get isLoading => _isLoading.value;

  final _isBtnLoading = false.obs;
  bool get isBtnLoading => _isBtnLoading.value;

  @override
  void onInit() {
    getUserData();
    super.onInit();
  }

  getUserData() async {
    _isLoading.value = true;
    try {
      final uid = userUid ?? '';
      final profile = await _profilesRepo.getProfile(uid);

      final name = profile?['display_name'] ?? LocalStorage.getName();
      final email = profile?['email'] ?? LocalStorage.getEmail();
      final imageUrl = profile?['avatar_url'] ?? LocalStorage.getImage();

      userModel = UserModel(
        name: name,
        uniqueId: uid,
        email: email,
        phoneNumber: "",
        isActive: true,
        imageUrl: imageUrl,
        isPremium: false,
      );

      nameController.text = userModel.name;
      numberController.text = userModel.phoneNumber;
      emailController.text = userModel.email;

      isEmailHave.value = userModel.email.isNotEmpty;

      return userModel;
    } catch (e) {
      debugPrint("Error from getUserData => $e");
      return null;
    } finally {
      _isLoading.value = false;
      update();
    }
  }

  updateUserData() async {
    try {
      _isBtnLoading.value = true;
      update();

      final uid = userUid ?? '';
      await _profilesRepo.upsertProfile(
        userId: uid,
        displayName: nameController.text.trim(),
        email: emailController.text.trim(),
      );

      LocalStorage.saveName(name: nameController.text.trim());
      LocalStorage.saveEmail(email: emailController.text.trim());

      ToastMessage.success(Strings.updateProfile);
      Get.offAllNamed(Routes.homeScreen);
      return true;
    } catch (e) {
      debugPrint("Error from updateUserData => $e");
      return false;
    } finally {
      _isBtnLoading.value = false;
      update();
    }
  }

  RxBool imageSelected = false.obs;
  late File file;
  RxString filePathString = ''.obs;

  updateUserDataWithImage() async {
    try {
      _isBtnLoading.value = true;
      update();

      final uid = userUid ?? '';
      final bytes = await file.readAsBytes();
      final ext = file.path.split('.').last;

      final avatarUrl = await _profilesRepo.uploadAvatar(
        userId: uid,
        bytes: bytes,
        fileExtension: ext,
      );

      await _profilesRepo.upsertProfile(
        userId: uid,
        displayName: nameController.text.trim(),
        email: emailController.text.trim(),
        avatarUrl: avatarUrl,
      );

      LocalStorage.saveName(name: nameController.text.trim());
      LocalStorage.saveEmail(email: emailController.text.trim());
      if (avatarUrl != null) {
        LocalStorage.saveImage(image: avatarUrl);
      }

      ToastMessage.success(Strings.updateProfile);
      Get.offAllNamed(Routes.homeScreen);
      return true;
    } catch (e) {
      debugPrint("Error from updateUserDataWithImage => $e");
      return false;
    } finally {
      _isBtnLoading.value = false;
      update();
    }
  }

  final nameController = TextEditingController();
  final numberController = TextEditingController();
  final emailController = TextEditingController();
  RxBool isEmailHave = true.obs;
}
