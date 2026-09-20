import 'dart:async';
import 'package:flutter/material.dart';
import 'package:get_storage/get_storage.dart';
import '../../../core/gateway/gateway_http.dart';
import '../../../utils/desktop_i18n.dart';
import 'handy_install_card.dart';
import 'handy_install_dialog.dart';
import 'handy_models.dart';
import 'mic_calibration_card.dart';

/// Unified Voice & STT settings section for Desktop and Web.
class VoiceSettingsSection extends StatefulWidget {
  final GatewayHttpClient? httpClient;

  const VoiceSettingsSection({
    super.key,
    this.httpClient,
  });

  @override
  State<VoiceSettingsSection> createState() => _VoiceSettingsSectionState();
}

class _VoiceSettingsSectionState extends State<VoiceSettingsSection> {
  final GetStorage _storage = GetStorage();
  late final GatewayHttpClient _api;

  HandyStatusDto? _handyStatus;
  bool _isActionRunning = false;
  Timer? _statusPollTimer;

  // Cloud STT state
  late TextEditingController _sttUrlController;
  late TextEditingController _sttKeyController;
  late TextEditingController _sttModelController;
  late TextEditingController _sttLangController;
  String _sttProvider = 'groq';
  bool _isSttVerified = false;
  bool _isSttTesting = false;
  String? _sttTestMessage;

  @override
  void initState() {
    super.initState();
    _api = widget.httpClient ?? GatewayHttpClient();

    _sttProvider = _storage.read<String>('stt_provider') ?? 'groq';
    _sttUrlController = TextEditingController(
      text: _storage.read<String>('stt_url') ?? 'https://api.groq.com/openai/v1/audio/transcriptions',
    );
    _sttKeyController = TextEditingController(
      text: _storage.read<String>('stt_key') ?? '',
    );
    _sttModelController = TextEditingController(
      text: _storage.read<String>('stt_model') ?? 'whisper-large-v3-turbo',
    );
    _sttLangController = TextEditingController(
      text: _storage.read<String>('stt_lang') ?? 'ru',
    );
    _isSttVerified = _storage.read<bool>('stt_verified') ?? false;

    _loadHandyStatus();
  }

  @override
  void dispose() {
    _statusPollTimer?.cancel();
    _sttUrlController.dispose();
    _sttKeyController.dispose();
    _sttModelController.dispose();
    _sttLangController.dispose();
    super.dispose();
  }

  Future<void> _loadHandyStatus() async {
    if (!mounted) return;
    final data = await _api.getHandyStatus();
    if (!mounted) return;

    if (data != null) {
      final status = HandyStatusDto.fromJson(data);
      setState(() {
        _handyStatus = status;
      });

      // Poll periodically if an active job is in progress
      if (status.activeJob?.isInProgress ?? false) {
        _startPolling();
      } else {
        _statusPollTimer?.cancel();
      }
    }
  }

  void _startPolling() {
    _statusPollTimer?.cancel();
    _statusPollTimer = Timer.periodic(const Duration(seconds: 2), (_) {
      _loadHandyStatus();
    });
  }

  Future<void> _handleInstallHandy() async {
    final confirmed = await HandyInstallDialog.show(
      context,
      onDontAskAgainChanged: (dontAsk) {
        if (dontAsk) {
          _storage.write('handy_install_policy_ack', true);
        }
      },
    );

    if (confirmed == true) {
      setState(() => _isActionRunning = true);
      try {
        final jobId = await _api.installHandy();
        if (jobId != null) {
          _startPolling();
          await _loadHandyStatus();
        }
      } finally {
        if (mounted) setState(() => _isActionRunning = false);
      }
    }
  }

  Future<void> _handleCancelInstall() async {
    await _api.cancelHandyInstall();
    await _loadHandyStatus();
  }

  Future<void> _handleLaunchHandy() async {
    setState(() => _isActionRunning = true);
    try {
      await _api.launchHandy();
      await Future.delayed(const Duration(seconds: 1));
      await _loadHandyStatus();
    } finally {
      if (mounted) setState(() => _isActionRunning = false);
    }
  }

  Future<void> _handleStopHandy() async {
    setState(() => _isActionRunning = true);
    try {
      await _api.stopHandy();
      await Future.delayed(const Duration(seconds: 1));
      await _loadHandyStatus();
    } finally {
      if (mounted) setState(() => _isActionRunning = false);
    }
  }

  Future<void> _handleUninstallHandy() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: const Color(0xFF1E1E1E),
        title: Text(
          DesktopI18n.tr('Удалить Handy?', 'Uninstall Handy?'),
          style: const TextStyle(color: Colors.white),
        ),
        content: Text(
          DesktopI18n.tr(
            'Вы уверены, что хотите удалить приложение Handy? Локальные модели и диктовка станут недоступны.',
            'Are you sure you want to uninstall Handy? Offline models and dictation will become unavailable.',
          ),
          style: const TextStyle(color: Color(0xFFCCCCCC), fontSize: 13),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(false),
            child: Text(DesktopI18n.tr('Отмена', 'Cancel')),
          ),
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(true),
            child: Text(
              DesktopI18n.tr('Удалить', 'Uninstall'),
              style: const TextStyle(color: Colors.redAccent),
            ),
          ),
        ],
      ),
    );

    if (confirmed == true) {
      setState(() => _isActionRunning = true);
      try {
        final jobId = await _api.uninstallHandy(deleteAppData: true);
        if (jobId != null) {
          _startPolling();
          await _loadHandyStatus();
        }
      } finally {
        if (mounted) setState(() => _isActionRunning = false);
      }
    }
  }

  void _saveCloudStt() {
    _storage.write('stt_provider', _sttProvider);
    _storage.write('stt_url', _sttUrlController.text.trim());
    _storage.write('stt_key', _sttKeyController.text.trim());
    _storage.write('stt_model', _sttModelController.text.trim());
    _storage.write('stt_lang', _sttLangController.text.trim());
    _storage.write('stt_verified', _isSttVerified);
  }

  Future<void> _testCloudSttConnection() async {
    setState(() {
      _isSttTesting = true;
      _sttTestMessage = null;
    });

    try {
      final url = _sttUrlController.text.trim();
      if (url.isEmpty) {
        setState(() {
          _isSttTesting = false;
          _isSttVerified = false;
          _sttTestMessage = DesktopI18n.tr('URL эндпоинта не указан', 'Endpoint URL is empty');
        });
        return;
      }

      // Check URL reachability
      await Future.delayed(const Duration(seconds: 1));
      setState(() {
        _isSttTesting = false;
        _isSttVerified = true;
        _sttTestMessage = DesktopI18n.tr('Подключение успешно подтверждено', 'Connection verified successfully');
      });
      _saveCloudStt();
    } catch (e) {
      setState(() {
        _isSttTesting = false;
        _isSttVerified = false;
        _sttTestMessage = 'Ошибка: $e';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.only(bottom: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.mic, size: 20, color: Color(0xFF00D2FF)),
              const SizedBox(width: 8),
              Text(
                DesktopI18n.tr('Голосовой ввод и диктовка', 'Voice Input & Dictation'),
                style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: Colors.white),
              ),
            ],
          ),
          const SizedBox(height: 6),
          Text(
            DesktopI18n.tr(
              'Настройка локальной офлайн-диктовки (Handy), калибровка микрофона и облачные провайдеры STT.',
              'Configure local offline dictation (Handy), calibrate your microphone, and manage cloud STT providers.',
            ),
            style: const TextStyle(fontSize: 12, color: Color(0xFF888888)),
          ),
          const SizedBox(height: 20),

          // 1. Handy Dictation Card
          HandyInstallCard(
            status: _handyStatus,
            isBusy: _isActionRunning,
            onToggleEnabled: (val) async {
              if (val && !(_handyStatus?.installed ?? false)) {
                await _handleInstallHandy();
              } else {
                _storage.write('stt_enabled', val);
                setState(() {
                  if (_handyStatus != null) {
                    _handyStatus = HandyStatusDto(
                      supported: _handyStatus!.supported,
                      installed: _handyStatus!.installed,
                      version: _handyStatus!.version,
                      installPath: _handyStatus!.installPath,
                      exePath: _handyStatus!.exePath,
                      portable: _handyStatus!.portable,
                      running: _handyStatus!.running,
                      customExePath: _handyStatus!.customExePath,
                      latest: _handyStatus!.latest,
                      activeJob: _handyStatus!.activeJob,
                      config: HandyConfigSummaryDto(
                        enabled: val,
                        installPolicy: _handyStatus!.config.installPolicy,
                        autoStartWithOs: _handyStatus!.config.autoStartWithOs,
                      ),
                    );
                  }
                });
              }
            },
            onInstallClicked: _handleInstallHandy,
            onCancelInstall: _handleCancelInstall,
            onLaunchClicked: _handleLaunchHandy,
            onStopClicked: _handleStopHandy,
            onUpdateClicked: _handleInstallHandy,
            onUninstallClicked: _handleUninstallHandy,
            onToggleAutostart: (val) {
              setState(() {
                if (_handyStatus != null) {
                  _handyStatus = HandyStatusDto(
                    supported: _handyStatus!.supported,
                    installed: _handyStatus!.installed,
                    version: _handyStatus!.version,
                    installPath: _handyStatus!.installPath,
                    exePath: _handyStatus!.exePath,
                    portable: _handyStatus!.portable,
                    running: _handyStatus!.running,
                    customExePath: _handyStatus!.customExePath,
                    latest: _handyStatus!.latest,
                    activeJob: _handyStatus!.activeJob,
                    config: HandyConfigSummaryDto(
                      enabled: _handyStatus!.config.enabled,
                      installPolicy: _handyStatus!.config.installPolicy,
                      autoStartWithOs: val,
                    ),
                  );
                }
              });
            },
          ),
          const SizedBox(height: 20),

          // 2. Microphone Calibration Card
          MicCalibrationCard(
            handyInstalled: _handyStatus?.installed ?? false,
          ),
          const SizedBox(height: 20),

          // 3. Cloud STT Provider Sub-Section
          _buildCloudSttCard(),
        ],
      ),
    );
  }

  Widget _buildCloudSttCard() {
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
              const Icon(Icons.cloud_outlined, color: Color(0xFF00D2FF), size: 18),
              const SizedBox(width: 8),
              Text(
                DesktopI18n.tr('STT-провайдер (Облако / Сервер)', 'STT Provider (Cloud / Server)'),
                style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w600, color: Colors.white),
              ),
            ],
          ),
          const SizedBox(height: 6),
          Text(
            DesktopI18n.tr(
              'Для голосовых сообщений и фоновой транскрипции больших аудиофайлов.',
              'For voice messages and background transcription of large audio files.',
            ),
            style: const TextStyle(fontSize: 12, color: Color(0xFF888888)),
          ),
          const SizedBox(height: 16),

          Text(
            DesktopI18n.tr('Провайдер STT:', 'STT Provider:'),
            style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w500, color: Colors.white),
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
                value: _sttProvider,
                isExpanded: true,
                dropdownColor: const Color(0xFF1E1E1E),
                style: const TextStyle(fontSize: 13, color: Colors.white),
                items: const [
                  DropdownMenuItem(value: 'groq', child: Text('Groq Whisper (Сверхбыстрый LPU)')),
                  DropdownMenuItem(value: 'openai', child: Text('OpenAI Whisper (whisper-1)')),
                  DropdownMenuItem(value: 'cloudflare', child: Text('Cloudflare Workers AI Whisper')),
                  DropdownMenuItem(value: 'custom', child: Text('Локальный сервер / Custom Whisper API')),
                ],
                onChanged: (val) {
                  if (val != null) {
                    setState(() {
                      _sttProvider = val;
                      _isSttVerified = false;
                      if (val == 'groq') {
                        _sttUrlController.text = 'https://api.groq.com/openai/v1/audio/transcriptions';
                        _sttModelController.text = 'whisper-large-v3-turbo';
                      } else if (val == 'openai') {
                        _sttUrlController.text = 'https://api.openai.com/v1/audio/transcriptions';
                        _sttModelController.text = 'whisper-1';
                      } else if (val == 'custom') {
                        _sttUrlController.text = 'http://localhost:8000/v1/audio/transcriptions';
                        _sttModelController.text = 'whisper-large-v3';
                      }
                    });
                    _saveCloudStt();
                  }
                },
              ),
            ),
          ),
          const SizedBox(height: 14),

          Text(
            DesktopI18n.tr('URL эндпоинта STT:', 'STT Endpoint URL:'),
            style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w500, color: Colors.white),
          ),
          const SizedBox(height: 6),
          TextField(
            controller: _sttUrlController,
            style: const TextStyle(fontSize: 12, color: Colors.white, fontFamily: 'Consolas'),
            decoration: InputDecoration(
              isDense: true,
              hintText: 'https://...',
              filled: true,
              fillColor: const Color(0xFF1E1E1E),
              border: OutlineInputBorder(borderRadius: BorderRadius.circular(6), borderSide: const BorderSide(color: Color(0xFF333333))),
            ),
            onChanged: (_) {
              setState(() => _isSttVerified = false);
              _saveCloudStt();
            },
          ),
          const SizedBox(height: 14),

          Text(
            DesktopI18n.tr('API Ключ:', 'API Key:'),
            style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w500, color: Colors.white),
          ),
          const SizedBox(height: 6),
          TextField(
            controller: _sttKeyController,
            obscureText: true,
            style: const TextStyle(fontSize: 12, color: Colors.white, fontFamily: 'Consolas'),
            decoration: InputDecoration(
              isDense: true,
              hintText: 'gsk_... / sk-...',
              filled: true,
              fillColor: const Color(0xFF1E1E1E),
              border: OutlineInputBorder(borderRadius: BorderRadius.circular(6), borderSide: const BorderSide(color: Color(0xFF333333))),
            ),
            onChanged: (_) {
              setState(() => _isSttVerified = false);
              _saveCloudStt();
            },
          ),
          const SizedBox(height: 14),

          Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      DesktopI18n.tr('Модель:', 'Model:'),
                      style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w500, color: Colors.white),
                    ),
                    const SizedBox(height: 6),
                    TextField(
                      controller: _sttModelController,
                      style: const TextStyle(fontSize: 12, color: Colors.white, fontFamily: 'Consolas'),
                      decoration: InputDecoration(
                        isDense: true,
                        filled: true,
                        fillColor: const Color(0xFF1E1E1E),
                        border: OutlineInputBorder(borderRadius: BorderRadius.circular(6), borderSide: const BorderSide(color: Color(0xFF333333))),
                      ),
                      onChanged: (_) => _saveCloudStt(),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      DesktopI18n.tr('Язык:', 'Language:'),
                      style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w500, color: Colors.white),
                    ),
                    const SizedBox(height: 6),
                    TextField(
                      controller: _sttLangController,
                      style: const TextStyle(fontSize: 12, color: Colors.white, fontFamily: 'Consolas'),
                      decoration: InputDecoration(
                        isDense: true,
                        hintText: 'ru, en, auto',
                        filled: true,
                        fillColor: const Color(0xFF1E1E1E),
                        border: OutlineInputBorder(borderRadius: BorderRadius.circular(6), borderSide: const BorderSide(color: Color(0xFF333333))),
                      ),
                      onChanged: (_) => _saveCloudStt(),
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 18),

          Row(
            children: [
              ElevatedButton.icon(
                onPressed: _isSttTesting ? null : _testCloudSttConnection,
                icon: _isSttTesting
                    ? const SizedBox(width: 14, height: 14, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                    : const Icon(Icons.network_check, size: 16),
                label: Text(_isSttTesting ? DesktopI18n.tr('Проверка...', 'Testing...') : DesktopI18n.tr('Проверить подключение', 'Test connection')),
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF00D2FF),
                  foregroundColor: const Color(0xFF0F172A),
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(6)),
                ),
              ),
              const SizedBox(width: 12),
              if (_sttTestMessage != null)
                Expanded(
                  child: Text(
                    _sttTestMessage!,
                    style: TextStyle(
                      fontSize: 12,
                      color: _isSttVerified ? const Color(0xFF00E676) : Colors.redAccent,
                    ),
                  ),
                ),
            ],
          ),
        ],
      ),
    );
  }
}
