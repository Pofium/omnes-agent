// Unit tests for the chat-composer slash-command helpers and the image
// attachment preview path extraction.

import 'package:flutter_test/flutter_test.dart';
import 'package:omnes_shared/omnes_shared.dart';

void main() {
  group('slashCommandQuery', () {
    test('returns null when the input does not start with a slash', () {
      expect(slashCommandQuery('hello'), isNull);
      expect(slashCommandQuery(''), isNull);
      expect(slashCommandQuery(' /upload'), isNull);
    });

    test('opens the menu on a lone slash', () {
      expect(slashCommandQuery('/'), '');
    });

    test('returns the partial token while it is being typed', () {
      expect(slashCommandQuery('/up'), 'up');
      expect(slashCommandQuery('/upload'), 'upload');
    });

    test('closes the menu once whitespace follows the token', () {
      expect(slashCommandQuery('/upload '), isNull);
      expect(slashCommandQuery('/upload now'), isNull);
    });
  });

  group('matchSlashCommands', () {
    test('matches upload by prefix, case-insensitively', () {
      expect(matchSlashCommands('u').map((c) => c.name), contains('upload'));
      expect(matchSlashCommands('UP').map((c) => c.name), contains('upload'));
    });

    test('returns nothing for an unknown prefix', () {
      expect(matchSlashCommands('zzz'), isEmpty);
    });

    test('returns every command for an empty query', () {
      expect(matchSlashCommands('').length, kSlashCommands.length);
    });
  });

  group('isUploadCommand', () {
    test('accepts the command with padding and mixed case', () {
      expect(isUploadCommand('/upload'), isTrue);
      expect(isUploadCommand('  /upload  '), isTrue);
      expect(isUploadCommand('/Upload'), isTrue);
    });

    test('rejects literals and other commands', () {
      expect(isUploadCommand('/upload now'), isFalse);
      expect(isUploadCommand('upload'), isFalse);
      expect(isUploadCommand('/goal'), isFalse);
    });
  });

  group('imageAttachmentPath', () {
    test('extracts the path from a screenshot chip', () {
      expect(
        imageAttachmentPath(r'shot.png (C:\Users\u\.omnesagent\attachments\shot.png)'),
        r'C:\Users\u\.omnesagent\attachments\shot.png',
      );
    });

    test('strips the @ from a project-relative reference', () {
      expect(imageAttachmentPath('@assets/logo.png'), 'assets/logo.png');
    });

    test('keeps a plain absolute path verbatim', () {
      expect(imageAttachmentPath(r'C:\pics\photo.jpeg'), r'C:\pics\photo.jpeg');
    });

    test('unquotes a path with spaces', () {
      expect(imageAttachmentPath('"C:/my pics/a b.webp"'), 'C:/my pics/a b.webp');
    });

    test('returns null for non-image attachments', () {
      expect(imageAttachmentPath('@lib/main.dart'), isNull);
      expect(imageAttachmentPath('readme'), isNull);
      expect(imageAttachmentPath(''), isNull);
    });
  });
}
