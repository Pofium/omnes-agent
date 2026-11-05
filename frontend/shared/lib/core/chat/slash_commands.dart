// Slash-command catalogue and attachment helpers shared by the desktop and
// web chat composers.
//
// The compositors recognise a command typed at the very start of the input.
// While the command token is still being written the UI shows a suggestion
// menu; once submitted the command runs its action instead of sending the
// literal text as a prompt (mirrors upstream web `slashCommands.ts`).

/// A command a user can invoke from the chat input, e.g. `/upload`.
class SlashCommand {
  /// Canonical name without the leading slash (`upload`).
  final String name;

  /// Rendered usage form, e.g. `/upload`.
  final String usage;

  /// Russian description shown in the suggestion menu.
  final String ruDescription;

  /// English description shown in the suggestion menu.
  final String enDescription;

  const SlashCommand({
    required this.name,
    required this.usage,
    required this.ruDescription,
    required this.enDescription,
  });
}

/// Commands offered in the composer's `/` suggestion menu, in display order.
const List<SlashCommand> kSlashCommands = <SlashCommand>[
  SlashCommand(
    name: 'upload',
    usage: '/upload',
    ruDescription: 'Прикрепить изображение через выбор файла',
    enDescription: 'Attach an image via the file picker',
  ),
];

/// The leading `/<token>` currently being typed, or `null` when the input is
/// not a slash command in progress.
///
/// Returning `''` for a lone `/` lets the menu open as soon as the user types
/// the slash. Once whitespace follows the token the command is considered
/// complete and the menu closes (the command itself is executed on submit).
String? slashCommandQuery(String text) {
  if (!text.startsWith('/')) return null;
  final body = text.substring(1);
  if (body.isEmpty) return '';
  if (RegExp(r'\s').hasMatch(body)) return null;
  return body;
}

/// Commands whose [SlashCommand.name] starts with [query] (case-insensitive),
/// preserving declaration order.
List<SlashCommand> matchSlashCommands(String query) {
  final q = query.toLowerCase();
  return kSlashCommands
      .where((c) => c.name.startsWith(q))
      .toList(growable: false);
}

/// Whether [text] is exactly the `/upload` command, ignoring surrounding
/// whitespace and case.
bool isUploadCommand(String text) => text.trim().toLowerCase() == '/upload';

const Set<String> _kImageExtensions = <String>{
  '.png',
  '.jpg',
  '.jpeg',
  '.gif',
  '.webp',
};

/// Extracts the filesystem path an image attachment refers to, or `null` when
/// the attachment is not a previewable image.
///
/// Recognises the three shapes the composers store:
///   * `name.png (C:\path\to\name.png)` — clipboard screenshots,
///   * `@relative/path.png` — project-relative references,
///   * `C:\path\to\name.png` or `"/path/with spaces.png"` — plain paths.
String? imageAttachmentPath(String attachment) {
  final raw = attachment.trim();
  if (raw.isEmpty) return null;

  String candidate = raw;
  final paren = RegExp(r'^.*\((.+)\)$').firstMatch(raw);
  if (paren != null) {
    candidate = paren.group(1)!.trim();
  } else if (raw.startsWith('@')) {
    candidate = raw.substring(1);
  }
  if (candidate.length >= 2 &&
      candidate.startsWith('"') &&
      candidate.endsWith('"')) {
    candidate = candidate.substring(1, candidate.length - 1);
  }

  final lower = candidate.toLowerCase();
  final dot = lower.lastIndexOf('.');
  if (dot < 0) return null;
  if (!_kImageExtensions.contains(lower.substring(dot))) return null;
  return candidate;
}
