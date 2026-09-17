import 'package:flutter/material.dart';
import '../../../design_system/shadcn_badge.dart';
import '../../../design_system/shadcn_button.dart';
import '../../../utils/desktop_i18n.dart';
import 'handy_models.dart';

/// Card presenting the state machine for Handy installation, updates, and controls.
class HandyInstallCard extends StatelessWidget {
  final HandyStatusDto? status;
  final bool isBusy;
  final ValueChanged<bool>? onToggleEnabled;
  final VoidCallback? onInstallClicked;
  final VoidCallback? onCancelInstall;
  final VoidCallback? onLaunchClicked;
  final VoidCallback? onStopClicked;
  final VoidCallback? onUpdateClicked;
  final VoidCallback? onUninstallClicked;
  final ValueChanged<bool>? onToggleAutostart;

  const HandyInstallCard({
    super.key,
    required this.status,
    this.isBusy = false,
    this.onToggleEnabled,
    this.onInstallClicked,
    this.onCancelInstall,
    this.onLaunchClicked,
    this.onStopClicked,
    this.onUpdateClicked,
    this.onUninstallClicked,
    this.onToggleAutostart,
  });

  @override
  Widget build(BuildContext context) {
    final st = status;
    final isSupported = st?.supported ?? true;
    final isInstalled = st?.installed ?? false;
    final isRunning = st?.running ?? false;
    final version = st?.version ?? '0.9.6';
    final job = st?.activeJob;
    final inProgress = job != null && job.isInProgress;

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
          // Header with switch
          Row(
            children: [
              const Icon(Icons.keyboard_voice, color: Color(0xFF00D2FF), size: 18),
              const SizedBox(width: 8),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      DesktopI18n.tr('Диктовка (Handy)', 'Dictation (Handy)'),
                      style: const TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: Colors.white,
                      ),
                    ),
                    Text(
                      DesktopI18n.tr(
                        'Офлайн-распознавание речи сторонним приложением',
                        'Offline speech recognition via a third-party app',
                      ),
                      style: const TextStyle(fontSize: 12, color: Color(0xFF888888)),
                    ),
                  ],
                ),
              ),
              Switch(
                value: st?.config.enabled ?? false,
                activeColor: const Color(0xFF00D2FF),
                onChanged: inProgress || !isSupported ? null : onToggleEnabled,
              ),
            ],
          ),
          const SizedBox(height: 16),

          if (!isSupported) ...[
            ShadcnBadge(
              label: DesktopI18n.tr('Не поддерживается на этой платформе', 'Not supported on this platform'),
              variant: ShadcnBadgeVariant.neutral,
            ),
          ] else if (inProgress) ...[
            _buildInstallingState(job),
          ] else if (!isInstalled) ...[
            _buildNotInstalledState(context),
          ] else ...[
            _buildInstalledState(context, version, isRunning, st?.latest?.updateAvailable ?? false, st?.config.autoStartWithOs ?? false),
          ],
        ],
      ),
    );
  }

  Widget _buildNotInstalledState(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: const Color(0xFF1E1E1E),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: const Color(0xFF2E2E2E)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.info_outline, color: Color(0xFF00D2FF), size: 16),
              const SizedBox(width: 6),
              Text(
                DesktopI18n.tr('Handy не установлен', 'Handy is not installed'),
                style: const TextStyle(fontWeight: FontWeight.w500, fontSize: 13, color: Colors.white),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            DesktopI18n.tr(
              'Для диктовки требуется бесплатное локальное приложение Handy (~20 МБ, офлайн-распознавание).',
              'Handy is a free offline dictation app (~20 MB, offline recognition).',
            ),
            style: const TextStyle(fontSize: 12, color: Color(0xFFAAAAAA)),
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              ShadcnButton(
                variant: ShadcnButtonVariant.primary,
                size: ShadcnButtonSize.sm,
                icon: Icons.download,
                label: DesktopI18n.tr('Установить автоматически', 'Install automatically'),
                onPressed: isBusy ? null : onInstallClicked,
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildInstallingState(HandyJobStatusDto job) {
    final pct = job.percent.clamp(0, 100);

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: const Color(0xFF1E1E1E),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: const Color(0xFF2E2E2E)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                job.message.isNotEmpty ? job.message : DesktopI18n.tr('Установка...', 'Installing...'),
                style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w500, color: Colors.white),
              ),
              Text(
                '$pct%',
                style: const TextStyle(fontFamily: 'Consolas', fontSize: 12, color: Color(0xFF00D2FF)),
              ),
            ],
          ),
          const SizedBox(height: 8),
          LinearProgressIndicator(
            value: pct / 100.0,
            backgroundColor: const Color(0xFF2E2E2E),
            valueColor: const AlwaysStoppedAnimation<Color>(Color(0xFF00D2FF)),
          ),
          const SizedBox(height: 12),
          if (job.stage == 'download' || job.stage == 'verify') ...[
            ShadcnButton(
              variant: ShadcnButtonVariant.outline,
              size: ShadcnButtonSize.sm,
              label: DesktopI18n.tr('Отменить', 'Cancel'),
              onPressed: onCancelInstall,
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildInstalledState(
    BuildContext context,
    String version,
    bool isRunning,
    bool updateAvailable,
    bool autoStartWithOs,
  ) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: const Color(0xFF1E1E1E),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: const Color(0xFF2E2E2E)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                'Handy v$version',
                style: const TextStyle(fontFamily: 'Consolas', fontWeight: FontWeight.w600, fontSize: 13, color: Colors.white),
              ),
              const SizedBox(width: 8),
              if (isRunning)
                ShadcnBadge(
                  label: DesktopI18n.tr('запущен', 'running'),
                  variant: ShadcnBadgeVariant.success,
                )
              else
                ShadcnBadge(
                  label: DesktopI18n.tr('не запущен', 'stopped'),
                  variant: ShadcnBadgeVariant.neutral,
                ),
              if (updateAvailable) ...[
                const SizedBox(width: 6),
                ShadcnBadge(
                  label: DesktopI18n.tr('доступно обновление', 'update available'),
                  variant: ShadcnBadgeVariant.warning,
                ),
              ],
            ],
          ),
          const SizedBox(height: 8),

          Row(
            children: [
              Text(
                DesktopI18n.tr('Горячая клавиша диктовки: ', 'Dictation hotkey: '),
                style: const TextStyle(fontSize: 12, color: Color(0xFFAAAAAA)),
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: const Color(0xFF262626),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: const Text(
                  'Ctrl+Space',
                  style: TextStyle(fontFamily: 'Consolas', fontSize: 11, fontWeight: FontWeight.w600, color: Color(0xFF00D2FF)),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),

          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              if (isRunning)
                ShadcnButton(
                  variant: ShadcnButtonVariant.outline,
                  size: ShadcnButtonSize.sm,
                  icon: Icons.stop,
                  label: DesktopI18n.tr('Остановить', 'Stop'),
                  onPressed: isBusy ? null : onStopClicked,
                )
              else
                ShadcnButton(
                  variant: ShadcnButtonVariant.outline,
                  size: ShadcnButtonSize.sm,
                  icon: Icons.play_arrow,
                  label: DesktopI18n.tr('Запустить', 'Launch'),
                  onPressed: isBusy ? null : onLaunchClicked,
                ),
              if (updateAvailable)
                ShadcnButton(
                  variant: ShadcnButtonVariant.primary,
                  size: ShadcnButtonSize.sm,
                  icon: Icons.system_update,
                  label: DesktopI18n.tr('Обновить', 'Update'),
                  onPressed: isBusy ? null : onUpdateClicked,
                ),
              ShadcnButton(
                variant: ShadcnButtonVariant.outline,
                size: ShadcnButtonSize.sm,
                icon: Icons.delete_outline,
                label: DesktopI18n.tr('Удалить', 'Uninstall'),
                onPressed: isBusy ? null : onUninstallClicked,
              ),
            ],
          ),
          const SizedBox(height: 8),

          Row(
            children: [
              Checkbox(
                value: autoStartWithOs,
                activeColor: const Color(0xFF00D2FF),
                checkColor: Colors.black,
                onChanged: isBusy ? null : (val) => onToggleAutostart?.call(val ?? false),
              ),
              Text(
                DesktopI18n.tr('Запускать вместе с Windows', 'Start with Windows'),
                style: const TextStyle(fontSize: 12, color: Color(0xFF888888)),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
