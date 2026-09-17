// OmnesAgent Web ADE — User Onboarding Gate Screen.
// Displayed immediately after initial admin password change on VPS before opening the main workspace.

import 'package:flutter/material.dart';
import 'package:omnes_shared/omnes_shared.dart';

import '../../theme/desktop_theme.dart';
import '../desktop_shell.dart';

class OnboardingGateScreen extends StatefulWidget {
  final UserProfileData? initialProfile;

  const OnboardingGateScreen({super.key, this.initialProfile});

  @override
  State<OnboardingGateScreen> createState() => _OnboardingGateScreenState();
}

class _OnboardingGateScreenState extends State<OnboardingGateScreen> {
  late UserProfileData _profile;

  @override
  void initState() {
    super.initState();
    _profile = widget.initialProfile ?? UserProfileData();
  }

  void _navigateToWorkspace() {
    if (!mounted) return;
    Navigator.of(context).pushReplacement(
      MaterialPageRoute(builder: (_) => const DesktopShell()),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: DesktopTheme.bgCanvas,
      body: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 24),
          child: Center(
            child: Container(
              constraints: const BoxConstraints(maxWidth: 820, maxHeight: 760),
              decoration: BoxDecoration(
                color: DesktopTheme.bgSurface,
                borderRadius: BorderRadius.circular(16),
                border: Border.all(color: DesktopTheme.borderSubtle, width: 1.2),
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withOpacity(0.6),
                    blurRadius: 36,
                    spreadRadius: 6,
                    offset: const Offset(0, 10),
                  ),
                  BoxShadow(
                    color: const Color(0xFF00D2FF).withOpacity(0.08),
                    blurRadius: 40,
                    spreadRadius: 2,
                  ),
                ],
              ),
              child: ClipRRect(
                borderRadius: BorderRadius.circular(16),
                child: UserOnboardingDialog(
                  initialProfile: _profile,
                  onSave: (updated) {
                    _profile = updated;
                    _navigateToWorkspace();
                  },
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
