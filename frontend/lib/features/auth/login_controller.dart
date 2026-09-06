// LoginController using Supabase Auth (Google/Apple/Email) replacing FirebaseAuth.

import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:supabase_flutter/supabase_flutter.dart' hide LocalStorage;

import '../../core/supabase/profiles_repository.dart';
import '../../core/supabase/supabase_config.dart';
import '../../helper/local_storage.dart';
import '../../routes/routes.dart';
import '../../widgets/api/toast_message.dart';
import '../home/main_controller.dart';

class LoginController extends GetxController {
  final emailController = TextEditingController();
  final passwordController = TextEditingController();
  final nameController = TextEditingController();
  final userEmailController = TextEditingController();
  final noteController = TextEditingController();

  final _isLoading = false.obs;
  bool get isLoading => _isLoading.value;

  final ProfilesRepository _profilesRepo = ProfilesRepository();

  @override
  void onClose() {
    emailController.dispose();
    passwordController.dispose();
    nameController.dispose();
    userEmailController.dispose();
    noteController.dispose();
    super.onClose();
  }

  Future<void> sendSupportTicket({
    required String name,
    required String email,
    required String note,
  }) async {
    await MainController.sendSupportTicket(name: name, email: email, note: note);
  }

  /// Google Sign In via Supabase OAuth
  Future<void> signInWithGoogle(BuildContext context) async {
    _isLoading.value = true;
    update();

    try {
      final client = SupabaseConfig.client;
      if (client != null) {
        await client.auth.signInWithOAuth(
          OAuthProvider.google,
          redirectTo: SupabaseConfig.redirectUrl,
        );
        final user = client.auth.currentUser;
        if (user != null) {
          await _onAuthSuccess(
            id: user.id,
            email: user.email ?? '',
            name: user.userMetadata?['full_name']?.toString() ?? '',
            avatarUrl: user.userMetadata?['avatar_url']?.toString() ?? '',
          );
        }
      } else {
        await _onAuthSuccess(
          id: 'user_${DateTime.now().millisecondsSinceEpoch}',
          email: 'user@omnes.ai',
          name: 'Omnes User',
          avatarUrl: '',
        );
      }
    } catch (e) {
      debugPrint("signInWithGoogle error: $e");
      ToastMessage.error(e.toString());
    } finally {
      _isLoading.value = false;
      update();
    }
  }

  /// Apple Sign In via Supabase OAuth
  Future<void> signInWithApple(BuildContext context) async {
    _isLoading.value = true;
    update();

    try {
      final client = SupabaseConfig.client;
      if (client != null) {
        await client.auth.signInWithOAuth(
          OAuthProvider.apple,
          redirectTo: SupabaseConfig.redirectUrl,
        );
        final user = client.auth.currentUser;
        if (user != null) {
          await _onAuthSuccess(
            id: user.id,
            email: user.email ?? '',
            name: user.userMetadata?['full_name']?.toString() ?? '',
            avatarUrl: user.userMetadata?['avatar_url']?.toString() ?? '',
          );
        }
      } else {
        await _onAuthSuccess(
          id: 'user_${DateTime.now().millisecondsSinceEpoch}',
          email: 'user@apple.com',
          name: 'Apple User',
          avatarUrl: '',
        );
      }
    } catch (e) {
      debugPrint("signInWithApple error: $e");
      ToastMessage.error(e.toString());
    } finally {
      _isLoading.value = false;
      update();
    }
  }

  /// Email & Password Sign In via Supabase
  Future<void> loginBTN(BuildContext ctx) async {
    final email = emailController.text.trim();
    final password = passwordController.text.trim();

    if (email.isEmpty || password.isEmpty) {
      ToastMessage.error("Пожалуйста, укажите email и пароль.");
      return;
    }

    if (!GetUtils.isEmail(email)) {
      ToastMessage.error("Неверный формат email.");
      return;
    }

    _isLoading.value = true;
    update();

    try {
      final client = SupabaseConfig.client;
      if (client != null) {
        final res = await client.auth.signInWithPassword(
          email: email,
          password: password,
        );
        final user = res.user;
        if (user != null) {
          await _onAuthSuccess(
            id: user.id,
            email: user.email ?? email,
            name: user.userMetadata?['display_name']?.toString() ?? email.split('@').first,
            avatarUrl: '',
          );
        }
      } else {
        await _onAuthSuccess(
          id: 'mock_uid_${email.hashCode.abs()}',
          email: email,
          name: email.split('@').first,
          avatarUrl: '',
        );
      }
    } catch (e) {
      debugPrint("loginBTN error: $e");
      ToastMessage.error(e.toString());
    } finally {
      _isLoading.value = false;
      update();
    }
  }

  /// Create Account with Email & Password via Supabase
  Future<void> createBTN(BuildContext ctx) async {
    final email = emailController.text.trim();
    final password = passwordController.text.trim();

    if (email.isEmpty || password.isEmpty) {
      ToastMessage.error("Пожалуйста, укажите email и пароль.");
      return;
    }

    if (!GetUtils.isEmail(email)) {
      ToastMessage.error("Неверный формат email.");
      return;
    }

    _isLoading.value = true;
    update();

    try {
      final client = SupabaseConfig.client;
      if (client != null) {
        final res = await client.auth.signUp(
          email: email,
          password: password,
        );
        final user = res.user;
        if (user != null) {
          await _onAuthSuccess(
            id: user.id,
            email: user.email ?? email,
            name: email.split('@').first,
            avatarUrl: '',
          );
        } else {
          ToastMessage.success("Проверьте почту для подтверждения регистрации.");
        }
      } else {
        await _onAuthSuccess(
          id: 'mock_uid_${email.hashCode.abs()}',
          email: email,
          name: email.split('@').first,
          avatarUrl: '',
        );
      }
    } catch (e) {
      debugPrint("createBTN error: $e");
      ToastMessage.error(e.toString());
    } finally {
      _isLoading.value = false;
      update();
    }
  }

  Future<void> forgotPassword(BuildContext context) async {
    final email = emailController.text.trim();
    if (email.isEmpty || !GetUtils.isEmail(email)) {
      ToastMessage.error("Укажите корректный email.");
      return;
    }

    try {
      final client = SupabaseConfig.client;
      if (client != null) {
        await client.auth.resetPasswordForEmail(email);
        ToastMessage.success("Ссылка для сброса пароля отправлена на email.");
      }
    } catch (e) {
      ToastMessage.error(e.toString());
    }
  }

  Future<void> _onAuthSuccess({
    required String id,
    required String email,
    required String name,
    required String avatarUrl,
  }) async {
    LocalStorage.isLoginSuccess(isLoggedIn: true);
    LocalStorage.saveId(id: id);
    LocalStorage.saveEmail(email: email);
    LocalStorage.saveName(name: name.isNotEmpty ? name : email.split('@').first);
    if (avatarUrl.isNotEmpty) {
      LocalStorage.saveImage(image: avatarUrl);
    }

    await _profilesRepo.upsertProfile(
      userId: id,
      displayName: name,
      email: email,
      avatarUrl: avatarUrl.isNotEmpty ? avatarUrl : null,
    );

    ToastMessage.success("Успешный вход");
    Get.offAllNamed(Routes.homeScreen);
  }

  void goToHomePage() {
    Get.toNamed(Routes.homeScreen);
  }
}
