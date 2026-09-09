import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:get_storage/get_storage.dart';
import 'package:http/http.dart' as http;
import 'package:omnes_shared/omnes_shared.dart';

import 'features/auth/change_password_screen.dart';
import 'features/auth/login_screen.dart';
import 'features/desktop_shell.dart';
import 'theme/desktop_theme.dart';
import 'utils/desktop_i18n.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await GetStorage.init();
  DesktopI18n.init();
  Get.put(DesktopThemeController());

  runApp(const OmnesDesktopApp());
}

class OmnesDesktopApp extends StatelessWidget {
  const OmnesDesktopApp({super.key});

  @override
  Widget build(BuildContext context) {
    return GetMaterialApp(
      title: 'OmnesAgent Web ADE',
      debugShowCheckedModeBanner: false,
      theme: DesktopTheme.lightTheme,
      darkTheme: DesktopTheme.darkTheme,
      themeMode: ThemeMode.dark,
      translations: LocalString(),
      locale: const Locale('ru', 'RU'),
      fallbackLocale: const Locale('en', 'US'),
      home: const WebAuthGate(),
    );
  }
}

class WebAuthGate extends StatefulWidget {
  const WebAuthGate({super.key});

  @override
  State<WebAuthGate> createState() => _WebAuthGateState();
}

class _WebAuthGateState extends State<WebAuthGate> {
  bool _checking = true;
  bool _authenticated = false;
  bool _mustChangePassword = false;

  @override
  void initState() {
    super.initState();
    _checkAuth();
  }

  Future<void> _checkAuth() async {
    try {
      final baseUrl = GatewayConfig.getBaseUrl();
      final token = await GatewayConfig.getToken();
      final url = Uri.parse('$baseUrl/api/auth/me');
      final resp = await http.get(
        url,
        headers: {
          if (token.isNotEmpty) 'Authorization': 'Bearer $token',
        },
      ).timeout(const Duration(seconds: 4));

      if (resp.statusCode == 200) {
        final data = jsonDecode(resp.body) as Map<String, dynamic>;
        if (mounted) {
          setState(() {
            _authenticated = data['authenticated'] == true;
            _mustChangePassword = data['must_change_password'] == true;
            _checking = false;
          });
        }
        return;
      }
    } catch (_) {}

    if (mounted) {
      setState(() {
        _authenticated = false;
        _checking = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_checking) {
      return Scaffold(
        backgroundColor: DesktopTheme.bgCanvas,
        body: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              ClipRRect(
                borderRadius: BorderRadius.circular(10),
                child: Image.asset('assets/Logo/app_launcher.png', width: 44, height: 44),
              ),
              const SizedBox(height: 16),
              const SizedBox(
                width: 22,
                height: 22,
                child: CircularProgressIndicator(strokeWidth: 2, color: Color(0xFF00D2FF)),
              ),
            ],
          ),
        ),
      );
    }

    if (!_authenticated) {
      return const LoginScreen();
    }
    if (_mustChangePassword) {
      return const ChangePasswordScreen();
    }
    return const DesktopShell();
  }
}
