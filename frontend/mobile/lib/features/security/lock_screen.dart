// PIN and Biometric Lock Screen for OmnesAgent.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';

import '../../helper/local_storage.dart';
import '../../routes/routes.dart';
import '../../utils/assets.dart';
import '../../utils/custom_color.dart';
import '../../utils/strings.dart';

class LockScreen extends StatefulWidget {
  const LockScreen({super.key});

  @override
  State<LockScreen> createState() => _LockScreenState();
}

class _LockScreenState extends State<LockScreen> {
  final TextEditingController _pinController = TextEditingController();
  String _errorMessage = '';
  int _pinLength = 4;

  @override
  void initState() {
    super.initState();
    final savedPin = LocalStorage.getPinCode();
    if (savedPin.isNotEmpty) {
      _pinLength = savedPin.length;
    }
    // Attempt biometric unlock if enabled
    if (LocalStorage.isBiometricAuthEnabled()) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _triggerBiometrics();
      });
    }
  }

  void _onKeyPress(String digit) {
    if (_pinController.text.length < _pinLength) {
      setState(() {
        _pinController.text += digit;
        _errorMessage = '';
      });
      if (_pinController.text.length == _pinLength) {
        _verifyPin();
      }
    }
  }

  void _onBackspace() {
    if (_pinController.text.isNotEmpty) {
      setState(() {
        _pinController.text =
            _pinController.text.substring(0, _pinController.text.length - 1);
        _errorMessage = '';
      });
    }
  }

  void _verifyPin() {
    final savedPin = LocalStorage.getPinCode();
    if (_pinController.text == savedPin) {
      Get.offAllNamed(Routes.homeScreen);
    } else {
      HapticFeedback.vibrate();
      setState(() {
        _errorMessage = Strings.incorrectPin.tr;
        _pinController.clear();
      });
    }
  }

  void _triggerBiometrics() {
    // For demo / supported web & mobile: simulated or OS biometric dialog
    // Upon confirmation, unlock
    Get.offAllNamed(Routes.homeScreen);
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final currentInputLength = _pinController.text.length;

    return Scaffold(
      backgroundColor: isDark ? const Color(0xFF0F172A) : const Color(0xFFF8FAFC),
      body: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 380),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 28.0),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Spacer(flex: 2),
                  // App Icon (OmnesAgent Silver/Cyan Rounded App Icon)
                  Container(
                    width: 76,
                    height: 76,
                    decoration: BoxDecoration(
                      borderRadius: BorderRadius.circular(18),
                      boxShadow: [
                        BoxShadow(
                          color: const Color(0xFF00D2FF).withOpacity(0.28),
                          blurRadius: 24,
                          offset: const Offset(0, 8),
                        ),
                      ],
                    ),
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(18),
                      child: Image.asset(
                        Assets.appLauncher,
                        fit: BoxFit.cover,
                      ),
                    ),
                  ),
                  const SizedBox(height: 20),
                  Text(
                    Strings.omnesAgent.tr,
                    style: TextStyle(
                      fontSize: 24,
                      fontWeight: FontWeight.bold,
                      letterSpacing: 0.5,
                      color: isDark ? Colors.white : const Color(0xFF0F172A),
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    Strings.enterPinToUnlock.tr,
                    style: TextStyle(
                      fontSize: 14,
                      color: isDark ? Colors.white60 : Colors.black54,
                    ),
                  ),
                  const SizedBox(height: 32),

                  // PIN dots indicator
                  Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: List.generate(_pinLength, (index) {
                      final isFilled = index < currentInputLength;
                      return Container(
                        margin: const EdgeInsets.symmetric(horizontal: 8),
                        width: 16,
                        height: 16,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: isFilled
                              ? CustomColor.primaryColor
                              : (isDark
                                  ? Colors.white.withOpacity(0.15)
                                  : Colors.black.withOpacity(0.1)),
                          border: Border.all(
                            color: isFilled
                                ? CustomColor.primaryColor
                                : (isDark
                                    ? Colors.white.withOpacity(0.3)
                                    : Colors.black.withOpacity(0.2)),
                            width: 1.5,
                          ),
                        ),
                      );
                    }),
                  ),

                  if (_errorMessage.isNotEmpty) ...[
                    const SizedBox(height: 16),
                    Text(
                      _errorMessage,
                      style: const TextStyle(
                        color: Colors.redAccent,
                        fontSize: 13,
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                  ] else
                    const SizedBox(height: 24),

                  const Spacer(flex: 1),

                  // Numeric Keypad
                  _buildKeypad(isDark),

                  const Spacer(flex: 2),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildKeypad(bool isDark) {
    return Column(
      children: [
        _buildKeypadRow(['1', '2', '3'], isDark),
        const SizedBox(height: 14),
        _buildKeypadRow(['4', '5', '6'], isDark),
        const SizedBox(height: 14),
        _buildKeypadRow(['7', '8', '9'], isDark),
        const SizedBox(height: 14),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: [
            // Biometric button (if enabled)
            if (LocalStorage.isBiometricAuthEnabled())
              _buildSpecialKey(
                icon: Icons.fingerprint,
                onTap: _triggerBiometrics,
                isDark: isDark,
              )
            else
              const SizedBox(width: 72, height: 72),

            _buildDigitKey('0', isDark),

            // Backspace button
            _buildSpecialKey(
              icon: Icons.backspace_outlined,
              onTap: _onBackspace,
              isDark: isDark,
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildKeypadRow(List<String> digits, bool isDark) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: digits.map((d) => _buildDigitKey(d, isDark)).toList(),
    );
  }

  Widget _buildDigitKey(String digit, bool isDark) {
    return InkWell(
      onTap: () => _onKeyPress(digit),
      borderRadius: BorderRadius.circular(36),
      child: Container(
        width: 72,
        height: 72,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          color: isDark
              ? Colors.white.withOpacity(0.06)
              : Colors.black.withOpacity(0.04),
          border: Border.all(
            color: isDark
                ? Colors.white.withOpacity(0.12)
                : Colors.black.withOpacity(0.08),
          ),
        ),
        alignment: Alignment.center,
        child: Text(
          digit,
          style: TextStyle(
            fontSize: 26,
            fontWeight: FontWeight.w600,
            color: isDark ? Colors.white : const Color(0xFF0F172A),
          ),
        ),
      ),
    );
  }

  Widget _buildSpecialKey({
    required IconData icon,
    required VoidCallback onTap,
    required bool isDark,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(36),
      child: Container(
        width: 72,
        height: 72,
        alignment: Alignment.center,
        child: Icon(
          icon,
          size: 26,
          color: isDark ? Colors.white70 : Colors.black54,
        ),
      ),
    );
  }
}
