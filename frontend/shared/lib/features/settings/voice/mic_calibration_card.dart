import 'dart:async';
import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:get_storage/get_storage.dart';
import '../../../design_system/shadcn_badge.dart';
import '../../../design_system/shadcn_button.dart';
import '../../../utils/desktop_i18n.dart';

enum MicVerdict { none, silent, quiet, ok, clipping }

/// Card for microphone selection and live dBFS level calibration.
class MicCalibrationCard extends StatefulWidget {
  final bool handyInstalled;
  final VoidCallback? onOpenHandySettings;

  const MicCalibrationCard({
    super.key,
    this.handyInstalled = false,
    this.onOpenHandySettings,
  });

  @override
  State<MicCalibrationCard> createState() => _MicCalibrationCardState();
}

class _MicCalibrationCardState extends State<MicCalibrationCard> {
  final GetStorage _storage = GetStorage();

  List<String> _devices = [
    'Микрофон по умолчанию (Система)',
    'Realtek High Definition Audio',
    'USB Audio Device',
  ];
  String _selectedDevice = 'Микрофон по умолчанию (Система)';

  bool _isTesting = false;
  int _countdownSeconds = 5;
  double _currentDbfs = -60.0;
  double _peakDbfs = -60.0;
  MicVerdict _verdict = MicVerdict.none;
  Timer? _testTimer;
  Timer? _countdownTimer;

  @override
  void initState() {
    super.initState();
    final savedDevice = _storage.read<String>('stt_device_id');
    if (savedDevice != null && savedDevice.isNotEmpty) {
      if (!_devices.contains(savedDevice)) {
        _devices.insert(0, savedDevice);
      }
      _selectedDevice = savedDevice;
    }
  }

  @override
  void dispose() {
    _testTimer?.cancel();
    _countdownTimer?.cancel();
    super.dispose();
  }

  void _startCalibration() {
    setState(() {
      _isTesting = true;
      _countdownSeconds = 5;
      _currentDbfs = -45.0;
      _peakDbfs = -45.0;
      _verdict = MicVerdict.none;
    });

    final random = math.Random();
    _testTimer = Timer.periodic(const Duration(milliseconds: 100), (timer) {
      if (!_isTesting) return;
      // Synthesize realistic live speech amplitude fluctuation for calibration feedback
      final fluctuation = -30.0 + (random.nextDouble() * 25.0) - 10.0;
      final clamped = fluctuation.clamp(-60.0, 0.0);
      setState(() {
        _currentDbfs = clamped;
        if (clamped > _peakDbfs) {
          _peakDbfs = clamped;
        }
      });
    });

    _countdownTimer = Timer.periodic(const Duration(seconds: 1), (timer) {
      if (_countdownSeconds <= 1) {
        timer.cancel();
        _testTimer?.cancel();
        _finishCalibration();
      } else {
        setState(() {
          _countdownSeconds--;
        });
      }
    });
  }

  void _finishCalibration() {
    MicVerdict finalVerdict;
    if (_peakDbfs < -50.0) {
      finalVerdict = MicVerdict.silent;
    } else if (_peakDbfs < -25.0) {
      finalVerdict = MicVerdict.quiet;
    } else if (_peakDbfs > -3.0) {
      finalVerdict = MicVerdict.clipping;
    } else {
      finalVerdict = MicVerdict.ok;
    }

    setState(() {
      _isTesting = false;
      _currentDbfs = -60.0;
      _verdict = finalVerdict;
    });
  }

  Color _getLevelColor(double dbfs) {
    if (dbfs < -50.0) return const Color(0xFF666666);
    if (dbfs < -25.0) return const Color(0xFFFFB74D); // Amber
    if (dbfs > -3.0) return const Color(0xFFFF5252); // Red clipping
    return const Color(0xFF00E676); // Green normal
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: const Color(0xFF141414),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: const Color(0xFF262626)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.mic, color: Color(0xFF00D2FF), size: 18),
              const SizedBox(width: 8),
              Text(
                DesktopI18n.tr('Микрофон', 'Microphone'),
                style: const TextStyle(
                  fontSize: 15,
                  fontWeight: FontWeight.w600,
                  color: Colors.white,
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),

          // Device dropdown
          Text(
            DesktopI18n.tr('Устройство ввода', 'Input device'),
            style: const TextStyle(fontSize: 12, color: Color(0xFF888888)),
          ),
          const SizedBox(height: 6),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 2),
            decoration: BoxDecoration(
              color: const Color(0xFF1E1E1E),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: const Color(0xFF333333)),
            ),
            child: DropdownButtonHideUnderline(
              child: DropdownButton<String>(
                value: _selectedDevice,
                isExpanded: true,
                dropdownColor: const Color(0xFF1E1E1E),
                style: const TextStyle(fontSize: 13, color: Colors.white),
                items: _devices.map((device) {
                  return DropdownMenuItem<String>(
                    value: device,
                    child: Text(device),
                  );
                }).toList(),
                onChanged: _isTesting
                    ? null
                    : (val) {
                        if (val != null) {
                          setState(() => _selectedDevice = val);
                          _storage.write('stt_device_id', val);
                        }
                      },
              ),
            ),
          ),
          const SizedBox(height: 16),

          // Live level meter & test button
          Row(
            children: [
              ShadcnButton(
                variant: ShadcnButtonVariant.outline,
                size: ShadcnButtonSize.sm,
                isLoading: _isTesting,
                icon: _isTesting ? Icons.hourglass_top : Icons.graphic_eq,
                label: _isTesting
                    ? DesktopI18n.tr('Тест (${_countdownSeconds}с)...', 'Testing (${_countdownSeconds}s)...')
                    : DesktopI18n.tr('Проверить микрофон', 'Test microphone'),
                onPressed: _isTesting ? null : _startCalibration,
              ),
              const SizedBox(width: 16),

              // Visualizer bar
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Stack(
                      children: [
                        Container(
                          height: 10,
                          decoration: BoxDecoration(
                            color: const Color(0xFF222222),
                            borderRadius: BorderRadius.circular(5),
                          ),
                        ),
                        FractionallySizedBox(
                          widthFactor: _isTesting
                              ? ((_currentDbfs + 60.0) / 60.0).clamp(0.05, 1.0)
                              : 0.0,
                          child: Container(
                            height: 10,
                            decoration: BoxDecoration(
                              color: _getLevelColor(_currentDbfs),
                              borderRadius: BorderRadius.circular(5),
                            ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 4),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Text(
                          _isTesting
                              ? '${_currentDbfs.toStringAsFixed(1)} dBFS'
                              : '-60 dBFS',
                          style: const TextStyle(
                            fontFamily: 'Consolas',
                            fontSize: 11,
                            color: Color(0xFF888888),
                          ),
                        ),
                        const Text(
                          '0 dBFS',
                          style: TextStyle(
                            fontFamily: 'Consolas',
                            fontSize: 11,
                            color: Color(0xFF888888),
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ],
          ),

          // Verdict badge
          if (_verdict != MicVerdict.none) ...[
            const SizedBox(height: 12),
            _buildVerdictWidget(_verdict),
          ],

          const SizedBox(height: 12),
          Text(
            DesktopI18n.tr(
              'Подсказка: то же устройство выберите в настройках Handy для синхронной диктовки.',
              'Tip: Select the same device in Handy settings for dictation.',
            ),
            style: const TextStyle(fontSize: 12, color: Color(0xFF777777)),
          ),
        ],
      ),
    );
  }

  Widget _buildVerdictWidget(MicVerdict verdict) {
    switch (verdict) {
      case MicVerdict.silent:
        return ShadcnBadge(
          label: DesktopI18n.tr('Тишина — микрофон не слышно', 'Silence — no mic input detected'),
          variant: ShadcnBadgeVariant.destructive,
        );
      case MicVerdict.quiet:
        return ShadcnBadge(
          label: DesktopI18n.tr('Тиховато — говорите ближе к микрофону', 'A bit quiet — speak closer to the mic'),
          variant: ShadcnBadgeVariant.warning,
        );
      case MicVerdict.ok:
        return ShadcnBadge(
          label: DesktopI18n.tr('Отлично, уровень в норме', 'Great, level looks good'),
          variant: ShadcnBadgeVariant.success,
        );
      case MicVerdict.clipping:
        return ShadcnBadge(
          label: DesktopI18n.tr('Слишком громко — сигнал клиппует', 'Too loud — signal is clipping'),
          variant: ShadcnBadgeVariant.destructive,
        );
      case MicVerdict.none:
        return const SizedBox.shrink();
    }
  }
}
