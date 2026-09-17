import 'package:flutter/material.dart';
import '../../../design_system/shadcn_button.dart';
import '../../../design_system/shadcn_colors.dart';
import '../../../utils/desktop_i18n.dart';

/// Confirmation dialog asking user permission to automatically install Handy.
class HandyInstallDialog extends StatefulWidget {
  final VoidCallback onConfirm;
  final ValueChanged<bool>? onDontAskAgainChanged;

  const HandyInstallDialog({
    super.key,
    required this.onConfirm,
    this.onDontAskAgainChanged,
  });

  static Future<bool?> show(
    BuildContext context, {
    required VoidCallback onConfirm,
    ValueChanged<bool>? onDontAskAgainChanged,
  }) {
    return showDialog<bool>(
      context: context,
      barrierDismissible: false,
      builder: (ctx) => HandyInstallDialog(
        onConfirm: onConfirm,
        onDontAskAgainChanged: onDontAskAgainChanged,
      ),
    );
  }

  @override
  State<HandyInstallDialog> createState() => _HandyInstallDialogState();
}

class _HandyInstallDialogState extends State<HandyInstallDialog> {
  bool _dontAskAgain = false;

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: ShadcnColors.card,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(14),
        side: BorderSide(color: ShadcnColors.border),
      ),
      insetPadding: const EdgeInsets.symmetric(horizontal: 24, vertical: 24),
      child: Container(
        width: 480,
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.mic, color: ShadcnColors.primary, size: 22),
                const SizedBox(width: 10),
                Text(
                  DesktopI18n.tr('Установить Handy?', 'Install Handy?'),
                  style: TextStyle(
                    fontWeight: FontWeight.w600,
                    fontSize: 18,
                    color: ShadcnColors.foreground,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 14),
            Text(
              DesktopI18n.tr(
                'Handy — бесплатное офлайн-приложение диктовки речи (MIT-лицензия). '
                'Агент скачает официальный релиз с GitHub (~20 МБ) и выполнит тихую установку для текущего пользователя без прав администратора.\n\n'
                'Аудио обрабатывается локально на вашем компьютере и никуда не отправляется.',
                'Handy is a free offline speech dictation app (MIT license). '
                'The agent will download the official release from GitHub (~20 MB) and silently install it for the current user without administrator privileges.\n\n'
                'Audio is processed completely locally on your computer and never leaves it.',
              ),
              style: TextStyle(fontSize: 13, height: 1.5, color: ShadcnColors.foregroundMuted),
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Checkbox(
                  value: _dontAskAgain,
                  activeColor: ShadcnColors.primary,
                  checkColor: Colors.black,
                  onChanged: (val) {
                    setState(() => _dontAskAgain = val ?? false);
                    widget.onDontAskAgainChanged?.call(_dontAskAgain);
                  },
                ),
                const SizedBox(width: 4),
                Expanded(
                  child: Text(
                    DesktopI18n.tr('Больше не спрашивать (автоустановка)', "Don't ask again (auto-install)"),
                    style: TextStyle(fontSize: 12, color: ShadcnColors.foregroundMuted),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 20),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                ShadcnButton(
                  variant: ShadcnButtonVariant.outline,
                  label: DesktopI18n.tr('Отмена', 'Cancel'),
                  onPressed: () => Navigator.of(context).pop(false),
                ),
                const SizedBox(width: 10),
                ShadcnButton(
                  variant: ShadcnButtonVariant.primary,
                  label: DesktopI18n.tr('Установить', 'Install'),
                  onPressed: () {
                    Navigator.of(context).pop(true);
                    widget.onConfirm();
                  },
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
