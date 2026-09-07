// Helper functions for voice and speech utilities in OmnesAgent

class SpeechHelper {
  /// Strips Markdown tags and formatting so Text-To-Speech (TTS) engine
  /// pronounces the text naturally without reading punctuation and markdown symbols.
  static String stripMarkdownForSpeech(String text) {
    if (text.isEmpty) return '';

    String cleaned = text;

    // 1. Remove fenced code blocks ```code```
    cleaned = cleaned.replaceAll(RegExp(r'```[\s\S]*?```'), ' ');

    // 2. Remove inline code `code`
    cleaned = cleaned.replaceAllMapped(RegExp(r'`([^`]+)`'), (m) => m[1] ?? '');

    // 3. Remove markdown images ![alt](url)
    cleaned = cleaned.replaceAll(RegExp(r'!\[([^\]]*)\]\([^\)]+\)'), '');

    // 4. Convert markdown links [text](url) -> text
    cleaned = cleaned.replaceAllMapped(
      RegExp(r'\[([^\]]+)\]\([^\)]+\)'),
      (m) => m[1] ?? '',
    );

    // 5. Remove header markers (#, ##, ###, etc.)
    cleaned = cleaned.replaceAll(RegExp(r'^\s*#{1,6}\s+', multiLine: true), '');

    // 6. Remove bold/italic formatting (**text**, *text*, __text__, _text_)
    cleaned = cleaned.replaceAllMapped(
      RegExp(r'(\*\*|__)(.*?)\1'),
      (m) => m[2] ?? '',
    );
    cleaned = cleaned.replaceAllMapped(
      RegExp(r'(\*|_)(.*?)\1'),
      (m) => m[2] ?? '',
    );

    // 7. Remove blockquotes (> quote)
    cleaned = cleaned.replaceAll(RegExp(r'^\s*>\s+', multiLine: true), '');

    // 8. Remove list bullets (- item, * item, + item, 1. item)
    cleaned = cleaned.replaceAll(RegExp(r'^\s*[-*+]\s+', multiLine: true), '');
    cleaned = cleaned.replaceAll(RegExp(r'^\s*\d+\.\s+', multiLine: true), '');

    // 9. Remove horizontal rules (--- or ***)
    cleaned = cleaned.replaceAll(RegExp(r'^\s*[-*_]{3,}\s*$', multiLine: true), '');

    // 10. Collapse multiple spaces and newlines
    cleaned = cleaned.replaceAll(RegExp(r'[ \t]+'), ' ');
    cleaned = cleaned.replaceAll(RegExp(r'\n{2,}'), '\n');

    return cleaned.trim();
  }
}
