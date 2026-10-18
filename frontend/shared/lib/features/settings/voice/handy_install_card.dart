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
    final isLight = Theme.of(context).brightness == Brightness.light;
    final cardBg = isLight ? Colors.white : const Color(0xFF141414);
    final cardBorder = isLight ? const Color(0xFFCBD5E1) : const Color(0xFF262626);
    final textPrimary = isLight ? const Color(0xFF020617) : Colors.white;
    final textMuted = isLight ? const Color(0xFF334155) : const Color(0xFF888888);

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
        color: cardBg,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: cardBorder),
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
                      style: TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                        color: textPrimary,
                      ),
                    ),
                    Text(
                      DesktopI18n.tr(
                        'Офлайн-распознавание речи сторонним приложением',
                        'Offline speech recognition via a third-party app',
                      ),
                      style: TextStyle(fontSize: 12, color: textMuted),
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
            _buildInstallingState(context, job),
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
    final isLight = Theme.of(context).brightness == Brightness.light;
    final subCardBg = isLight ? const Color(0xFFF8FAFC) : const Color(0xFF1E1E1E);
    final subCardBorder = isLight ? const Color(0xFFCBD5E1) : const Color(0xFF2E2E2E);
    final textPrimary = isLight ? const Color(0xFF020617) : Colors.white;
    final textSubtle = isLight ? const Color(0xFF475569) : const Color(0xFFAAAAAA);

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: subCardBg,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: subCardBorder),
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
                style: TextStyle(fontWeight: FontWeight.w500, fontSize: 13, color: textPrimary),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            DesktopI18n.tr(
              'Для диктовки требуется бесплатное локальное приложение Handy (~20 МБ, офлайн-распознавание).',
              'Handy is a free offline dictation app (~20 MB, offline recognition).',
            ),
            style: TextStyle(fontSize: 12, color: textSubtle),
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

  Widget _buildInstallingState(BuildContext context, HandyJobStatusDto job) {
    final isLight = Theme.of(context).brightness == Brightness.light;
    final subCardBg = isLight ? const Color(0xFFF8FAFC) : const Color(0xFF1E1E1E);
    final subCardBorder = isLight ? const Color(0xFFCBD5E1) : const Color(0xFF2E2E2E);
    final textPrimary = isLight ? const Color(0xFF020617) : Colors.white;

    final pct = job.percent.clamp(0, 100);

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: subCardBg,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: subCardBorder),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                job.message.isNotEmpty ? job.message : DesktopI18n.tr('Установка...', 'Installing...'),
                style: TextStyle(fontSize: 13, fontWeight: FontWeight.w500, color: textPrimary),
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
            backgroundColor: isLight ? const Color(0xFFE2E8F0) : const Color(0xFF2E2E2E),
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
    final isLight = Theme.of(context).brightness == Brightness.light;
    final subCardBg = isLight ? const Color(0xFFF8FAFC) : const Color(0xFF1E1E1E);
    final subCardBorder = isLight ? const Color(0xFFCBD5E1) : const Color(0xFF2E2E2E);
    final textPrimary = isLight ? const Color(0xFF020617) : Colors.white;
    final textSubtle = isLight ? const Color(0xFF475569) : const Color(0xFFAAAAAA);
    final textMuted = isLight ? const Color(0xFF334155) : const Color(0xFF888888);
    final hotkeyBg = isLight ? const Color(0xFFE2E8F0) : const Color(0xFF262626);

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: subCardBg,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: subCardBorder),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                'Handy v$version',
                style: TextStyle(fontFamily: 'Consolas', fontWeight: FontWeight.w600, fontSize: 13, color: textPrimary),
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
                style: TextStyle(fontSize: 12, color: textSubtle),
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: hotkeyBg,
                  borderRadius: BorderRadius.circular(4),
                ),
                child: const Text(
                  'Ctrl+Space',
                  style: TextStyle(fontFamily: 'Consolas', fontSize: 11, fontWeight: FontWeight.w600, color: Color(0xFF0284C7)),
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
                style: TextStyle(fontSize: 12, color: textMuted),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
