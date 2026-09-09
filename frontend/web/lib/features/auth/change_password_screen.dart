// OmnesAgent Web ADE — Change Admin Password Screen.
// Forces password change upon first login with temporary password.

import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;
import 'package:omnes_shared/omnes_shared.dart';

import '../../theme/desktop_theme.dart';
import '../desktop_shell.dart';

class ChangePasswordScreen extends StatefulWidget {
  final String? currentTempPassword;

  const ChangePasswordScreen({super.key, this.currentTempPassword});

  @override
  State<ChangePasswordScreen> createState() => _ChangePasswordScreenState();
}

class _ChangePasswordScreenState extends State<ChangePasswordScreen> {
  late final TextEditingController _oldPasswordController;
  final _newPasswordController = TextEditingController();
  final _confirmPasswordController = TextEditingController();

  bool _isLoading = false;
  bool _obscureOld = true;
  bool _obscureNew = true;
  String? _errorMessage;

  @override
  void initState() {
    super.initState();
    _oldPasswordController = TextEditingController(text: widget.currentTempPassword ?? '');
  }

  Future<void> _submitChangePassword() async {
    final oldPw = _oldPasswordController.text.trim();
    final newPw = _newPasswordController.text.trim();
    final confirmPw = _confirmPasswordController.text.trim();

    if (oldPw.isEmpty) {
      setState(() => _errorMessage = 'Введите текущий пароль');
      return;
    }
    if (newPw.length < 8) {
      setState(() => _errorMessage = 'Новый пароль должен содержать не менее 8 символов');
      return;
    }
    if (newPw == oldPw) {
      setState(() => _errorMessage = 'Новый пароль не должен совпадать с текущим');
      return;
    }
    if (newPw != confirmPw) {
      setState(() => _errorMessage = 'Пароли не совпадают');
      return;
    }

    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    try {
      final baseUrl = GatewayConfig.getBaseUrl();
      final token = await GatewayConfig.getToken();
      final url = Uri.parse('$baseUrl/api/auth/change-password');
      final response = await http.post(
        url,
        headers: {
          'Content-Type': 'application/json',
          if (token.isNotEmpty) 'Authorization': 'Bearer $token',
        },
        body: jsonEncode({
          'old_password': oldPw,
          'new_password': newPw,
        }),
      );

      if (!mounted) return;

      if (response.statusCode == 200) {
        // Successfully changed password! Proceed to DesktopShell
        Navigator.of(context).pushReplacement(
          MaterialPageRoute(builder: (_) => const DesktopShell()),
        );
      } else {
        final data = jsonDecode(response.body) as Map<String, dynamic>;
        final msg = data['message'] as String? ?? 'Ошибка при смене пароля';
        setState(() {
          _errorMessage = msg;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _errorMessage = 'Ошибка соединения с шлюзом: $e';
          _isLoading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: DesktopTheme.bgCanvas,
      body: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Container(
            width: 440,
            padding: const EdgeInsets.all(32),
            decoration: BoxDecoration(
              color: DesktopTheme.bgSurface,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: DesktopTheme.borderSubtle, width: 1.2),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.55),
                  blurRadius: 36,
                  spreadRadius: 4,
                  offset: const Offset(0, 12),
                ),
                BoxShadow(
                  color: const Color(0xFFF59E0B).withOpacity(0.08),
                  blurRadius: 24,
                  spreadRadius: 1,
                ),
              ],
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                // Warning / Security Icon
                Center(
                  child: Container(
                    width: 52,
                    height: 52,
                    decoration: BoxDecoration(
                      color: const Color(0xFFF59E0B).withOpacity(0.12),
                      borderRadius: BorderRadius.circular(12),
                      border: Border.all(color: const Color(0xFFF59E0B).withOpacity(0.3)),
                    ),
                    child: const Center(
                      child: Icon(Icons.shield_outlined, color: Color(0xFFF59E0B), size: 28),
                    ),
                  ),
                ),
                const SizedBox(height: 20),

                // Title
                Center(
                  child: Text(
                    'Смена пароля',
                    style: TextStyle(
                      color: DesktopTheme.textPrimary,
                      fontSize: 20,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
                const SizedBox(height: 6),
                Center(
                  child: Text(
                    'Требуется установить постоянный пароль для продолжения работы',
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      color: DesktopTheme.textMuted,
                      fontSize: 12,
                    ),
                  ),
                ),
                const SizedBox(height: 24),

                // Error Message Banner
                if (_errorMessage != null) ...[
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                    decoration: BoxDecoration(
                      color: const Color(0xFFEF4444).withOpacity(0.12),
                      borderRadius: BorderRadius.circular(8),
                      border: Border.all(color: const Color(0xFFEF4444).withOpacity(0.3)),
                    ),
                    child: Row(
                      children: [
                        const Icon(Icons.error_outline, color: Color(0xFFEF4444), size: 18),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            _errorMessage!,
                            style: const TextStyle(color: Color(0xFFEF4444), fontSize: 12),
                          ),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 18),
                ],

                // Current Password Field
                Text(
                  'Текущий (временный) пароль',
                  style: TextStyle(color: DesktopTheme.textSecondary, fontSize: 12, fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 6),
                TextField(
                  controller: _oldPasswordController,
                  obscureText: _obscureOld,
                  style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 13),
                  decoration: InputDecoration(
                    hintText: 'Текущий пароль...',
                    hintStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 13),
                    filled: true,
                    fillColor: DesktopTheme.bgSurfaceElevated,
                    prefixIcon: Icon(Icons.vpn_key_outlined, size: 16, color: DesktopTheme.textMuted),
                    suffixIcon: IconButton(
                      icon: Icon(
                        _obscureOld ? Icons.visibility_outlined : Icons.visibility_off_outlined,
                        size: 16,
                        color: DesktopTheme.textMuted,
                      ),
                      onPressed: () => setState(() => _obscureOld = !_obscureOld),
                    ),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
                    enabledBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
                  ),
                ),
                const SizedBox(height: 16),

                // New Password Field
                Text(
                  'Новый пароль (от 8 символов)',
                  style: TextStyle(color: DesktopTheme.textSecondary, fontSize: 12, fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 6),
                TextField(
                  controller: _newPasswordController,
                  obscureText: _obscureNew,
                  style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 13),
                  decoration: InputDecoration(
                    hintText: 'Новый надёжный пароль...',
                    hintStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 13),
                    filled: true,
                    fillColor: DesktopTheme.bgSurfaceElevated,
                    prefixIcon: Icon(Icons.lock_reset, size: 16, color: DesktopTheme.textMuted),
                    suffixIcon: IconButton(
                      icon: Icon(
                        _obscureNew ? Icons.visibility_outlined : Icons.visibility_off_outlined,
                        size: 16,
                        color: DesktopTheme.textMuted,
                      ),
                      onPressed: () => setState(() => _obscureNew = !_obscureNew),
                    ),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
                    enabledBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
                  ),
                ),
                const SizedBox(height: 16),

                // Confirm Password Field
                Text(
                  'Повторите новый пароль',
                  style: TextStyle(color: DesktopTheme.textSecondary, fontSize: 12, fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 6),
                TextField(
                  controller: _confirmPasswordController,
                  obscureText: true,
                  style: TextStyle(color: DesktopTheme.textPrimary, fontSize: 13),
                  onSubmitted: (_) => _submitChangePassword(),
                  decoration: InputDecoration(
                    hintText: 'Повторите новый пароль...',
                    hintStyle: TextStyle(color: DesktopTheme.textMuted, fontSize: 13),
                    filled: true,
                    fillColor: DesktopTheme.bgSurfaceElevated,
                    prefixIcon: Icon(Icons.check_circle_outline, size: 16, color: DesktopTheme.textMuted),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                    border: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
                    enabledBorder: OutlineInputBorder(borderRadius: BorderRadius.circular(8), borderSide: BorderSide(color: DesktopTheme.borderSubtle)),
                  ),
                ),
                const SizedBox(height: 24),

                // Submit Button
                ElevatedButton(
                  onPressed: _isLoading ? null : _submitChangePassword,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: const Color(0xFF00D2FF),
                    foregroundColor: Colors.black,
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                    elevation: 0,
                  ),
                  child: _isLoading
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2, color: Colors.black),
                        )
                      : const Text(
                          'Установить пароль и войти',
                          style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold),
                        ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
