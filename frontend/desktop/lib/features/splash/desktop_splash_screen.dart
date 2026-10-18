// Startup Splash Screen for OmnesAgent Desktop ADE.
// Displays an authentic cyber-minimal pulsing SVG logo with glowing aura
// while initializing backend, window geometry (full-screen maximize), and workspace state.

import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:get/get.dart';
import 'package:window_manager/window_manager.dart';

import '../../features/desktop_shell.dart';
import '../../utils/desktop_backend_manager.dart';

class DesktopSplashScreen extends StatefulWidget {
  const DesktopSplashScreen({super.key});

  @override
  State<DesktopSplashScreen> createState() => _DesktopSplashScreenState();
}

class _DesktopSplashScreenState extends State<DesktopSplashScreen>
    with TickerProviderStateMixin {
  late final AnimationController _pulseController;
  late final Animation<double> _scaleAnimation;
  late final Animation<double> _glowAnimation;

  late final AnimationController _exitController;
  late final Animation<double> _exitScaleAnimation;
  late final Animation<double> _exitOpacityAnimation;

  String _statusText = 'Инициализация рабочего пространства...';

  @override
  void initState() {
    super.initState();

    // 1. Циклическая пульсация логотипа и неонового свечения
    _pulseController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1400),
    )..repeat(reverse: true);

    _scaleAnimation = Tween<double>(begin: 0.94, end: 1.06).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOutSine),
    );

    _glowAnimation = Tween<double>(begin: 0.35, end: 0.85).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOutSine),
    );

    // 2. Анимация вылета (fly-out) на весь экран
    _exitController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 450),
    );

    _exitScaleAnimation = Tween<double>(begin: 1.0, end: 1.28).animate(
      CurvedAnimation(parent: _exitController, curve: Curves.easeInCubic),
    );

    _exitOpacityAnimation = Tween<double>(begin: 1.0, end: 0.0).animate(
      CurvedAnimation(parent: _exitController, curve: Curves.easeInQuad),
    );

    _initializeApp();
  }

  Future<void> _initializeApp() async {
    // Гарантируем разворот окна на весь экран в нативном слое
    try {
      if (await windowManager.isMinimized()) {
        await windowManager.restore();
      }
      await windowManager.maximize();
      await windowManager.focus();
    } catch (_) {}

    // Фоновая проверка/запуск бэкенда
    final start = DateTime.now();
    try {
      setState(() => _statusText = 'Проверка шлюза оркестратора...');
      await DesktopBackendManager.startBackendIfNeeded()
          .timeout(const Duration(seconds: 3));
    } catch (_) {}

    // Минимальное время показа для плавной анимации (1.4 с)
    final elapsed = DateTime.now().difference(start).inMilliseconds;
    if (elapsed < 1400) {
      await Future.delayed(Duration(milliseconds: 1400 - elapsed));
    }

    if (!mounted) return;

    setState(() => _statusText = 'Готово');

    // Анимация вылета
    await _exitController.forward();

    if (!mounted) return;

    // Бесшовный переход в DesktopShell
    Get.off(
      () => const DesktopShell(),
      transition: Transition.fadeIn,
      duration: const Duration(milliseconds: 300),
    );
  }

  @override
  void dispose() {
    _pulseController.dispose();
    _exitController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF090D16),
      body: AnimatedBuilder(
        animation: Listenable.merge([_pulseController, _exitController]),
        builder: (context, child) {
          final pulseScale = _scaleAnimation.value;
          final exitScale = _exitScaleAnimation.value;
          final combinedScale = pulseScale * exitScale;
          final opacity = _exitOpacityAnimation.value;
          final glow = _glowAnimation.value;

          return Opacity(
            opacity: opacity.clamp(0.0, 1.0),
            child: Container(
              width: double.infinity,
              height: double.infinity,
              decoration: const BoxDecoration(
                gradient: RadialGradient(
                  center: Alignment.center,
                  radius: 0.9,
                  colors: [
                    Color(0xFF131D31),
                    Color(0xFF090D16),
                  ],
                ),
              ),
              child: Stack(
                alignment: Alignment.center,
                children: [
                  // Сетка кибер-фона (тонкий паттерн)
                  Positioned.fill(
                    child: CustomPaint(
                      painter: _CyberGridPainter(),
                    ),
                  ),

                  // Центральный пульсирующий блок логотипа
                  Transform.scale(
                    scale: combinedScale,
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        // Неоновый ореол вокруг логотипа
                        Container(
                          width: 120,
                          height: 120,
                          decoration: BoxDecoration(
                            shape: BoxShape.circle,
                            boxShadow: [
                              BoxShadow(
                                color: const Color(0xFF00D2FF).withOpacity(glow * 0.5),
                                blurRadius: 40 * glow,
                                spreadRadius: 8 * glow,
                              ),
                              BoxShadow(
                                color: const Color(0xFF38BDF8).withOpacity(glow * 0.25),
                                blurRadius: 70 * glow,
                                spreadRadius: 15 * glow,
                              ),
                            ],
                          ),
                          child: Center(
                            child: SvgPicture.asset(
                              'assets/Logo/OA_icon.svg',
                              width: 84,
                              height: 84,
                              fit: BoxFit.contain,
                            ),
                          ),
                        ),
                        const SizedBox(height: 32),

                        // Бренд OmnesAgent
                        const Text(
                          'OMNESAGENT',
                          style: TextStyle(
                            fontFamily: 'Segoe UI',
                            fontSize: 22,
                            fontWeight: FontWeight.w800,
                            letterSpacing: 6,
                            color: Colors.white,
                          ),
                        ),
                        const SizedBox(height: 6),
                        Text(
                          'AUTONOMOUS DEVELOPMENT ENVIRONMENT',
                          style: TextStyle(
                            fontSize: 10,
                            fontWeight: FontWeight.w600,
                            letterSpacing: 2.5,
                            color: const Color(0xFF00D2FF).withOpacity(0.85),
                          ),
                        ),
                        const SizedBox(height: 28),

                        // Мини-прогресс бар с градиентом
                        SizedBox(
                          width: 180,
                          height: 3,
                          child: ClipRRect(
                            borderRadius: BorderRadius.circular(2),
                            child: const LinearProgressIndicator(
                              backgroundColor: Color(0xFF1E293B),
                              valueColor: AlwaysStoppedAnimation<Color>(Color(0xFF00D2FF)),
                            ),
                          ),
                        ),
                        const SizedBox(height: 12),

                        // Статус инициализации
                        Text(
                          _statusText,
                          style: const TextStyle(
                            fontSize: 11,
                            color: Color(0xFF64748B),
                            fontFamily: 'Consolas',
                          ),
                        ),
                      ],
                    ),
                  ),

                  // Нижняя плашка версии
                  Positioned(
                    bottom: 24,
                    child: Text(
                      'v1.0.0-PRO • DESKTOP WORKSTATION',
                      style: TextStyle(
                        fontSize: 10,
                        letterSpacing: 1.5,
                        color: const Color(0xFF475569).withOpacity(0.7),
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}

/// Декоративная кибер-сетка для фона
class _CyberGridPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = const Color(0xFF00D2FF).withOpacity(0.025)
      ..strokeWidth = 1.0;

    const step = 48.0;
    for (double x = 0; x < size.width; x += step) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }
    for (double y = 0; y < size.height; y += step) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
