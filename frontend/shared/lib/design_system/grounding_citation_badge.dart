import 'package:flutter/material.dart';
import '../core/gateway/models/grounding_citation.dart';
import 'shadcn_colors.dart';

/// GroundingCitationBadge displays a clickable interactive badge linking
/// verified claims to codebase files, AST nodes, or memory facts.
class GroundingCitationBadge extends StatelessWidget {
  final GroundingCitation citation;
  final int index;
  final VoidCallback? onTap;

  const GroundingCitationBadge({
    super.key,
    required this.citation,
    required this.index,
    this.onTap,
  });

  IconData _iconForSource(String source) {
    switch (source) {
      case 'file':
        return Icons.insert_drive_file_outlined;
      case 'ast_node':
        return Icons.account_tree_outlined;
      case 'memory':
        return Icons.memory;
      default:
        return Icons.link;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      preferBelow: false,
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
      decoration: BoxDecoration(
        color: ShadcnColors.cardElevated,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: ShadcnColors.borderActive.withOpacity(0.5)),
        boxShadow: const [
          BoxShadow(
            color: Color(0x40000000),
            blurRadius: 8,
            offset: Offset(0, 4),
          ),
        ],
      ),
      richMessage: TextSpan(
        children: [
          TextSpan(
            text: '[${citation.sourceType.toUpperCase()}] ',
            style: const TextStyle(
              fontSize: 10,
              fontWeight: FontWeight.bold,
              color: ShadcnColors.primary,
              fontFamily: 'JetBrains Mono',
            ),
          ),
          TextSpan(
            text: '${citation.title}\n',
            style: const TextStyle(
              fontSize: 11,
              fontWeight: FontWeight.w600,
              color: ShadcnColors.foreground,
              fontFamily: 'JetBrains Mono',
            ),
          ),
          if (citation.snippet != null && citation.snippet!.isNotEmpty)
            TextSpan(
              text: citation.snippet!,
              style: const TextStyle(
                fontSize: 10,
                color: ShadcnColors.foregroundMuted,
                fontStyle: FontStyle.italic,
              ),
            ),
        ],
      ),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(4),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
          margin: const EdgeInsets.symmetric(horizontal: 2),
          decoration: BoxDecoration(
            color: ShadcnColors.surface,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(
              color: ShadcnColors.primary.withOpacity(0.3),
              width: 1,
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                _iconForSource(citation.sourceType),
                size: 10,
                color: ShadcnColors.primary,
              ),
              const SizedBox(width: 3),
              Text(
                '$index',
                style: const TextStyle(
                  fontSize: 10,
                  fontWeight: FontWeight.bold,
                  color: ShadcnColors.primary,
                  fontFamily: 'JetBrains Mono',
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
