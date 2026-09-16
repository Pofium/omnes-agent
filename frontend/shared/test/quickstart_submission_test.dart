import 'package:flutter_test/flutter_test.dart';
import 'package:omnes_shared/core/gateway/quickstart_submission.dart';

void main() {
  group('QuickstartSubmission DTO Tests', () {
    test('BuilderSubmissionDto matches Rust serde snake_case contract exactly', () {
      final submission = BuilderSubmissionDto(
        modelProvider: const SelectorChoiceDto.fresh(
          ModelProviderChoiceDto(
            providerType: 'openai',
            alias: 'default',
            model: 'gpt-5',
            fields: {'api_key': 'sk-test-123'},
          ),
        ),
        riskProfile: const SelectorChoiceDto.fresh('balanced'),
        runtimeProfile: const SelectorChoiceDto.fresh('balanced'),
        memory: const SelectorChoiceDto.fresh('sqlite'),
        channels: const [
          SelectorChoiceDto.fresh(
            ChannelQuickStartDto(
              channelType: 'telegram',
              alias: 'tg',
              fields: {'bot_token': 'tg-token-123'},
            ),
          ),
        ],
        peerGroups: const [],
        agent: const AgentIdentityDto(
          name: 'omnes',
          systemPrompt: 'You are Omnes, the personal agent of Alice.',
          personalityFile: null,
          personalityFiles: [
            QuickstartPersonalityFileDto(filename: 'SOUL.md', content: 'Soul text'),
            QuickstartPersonalityFileDto(filename: 'IDENTITY.md', content: 'Identity text'),
            QuickstartPersonalityFileDto(filename: 'USER.md', content: 'User text'),
            QuickstartPersonalityFileDto(filename: 'MEMORY.md', content: 'Memory text'),
          ],
        ),
      );

      final json = submission.toJson();

      expect(json['model_provider']['mode'], equals('fresh'));
      expect(json['model_provider']['value']['provider_type'], equals('openai'));
      expect(json['model_provider']['value']['alias'], equals('default'));
      expect(json['model_provider']['value']['model'], equals('gpt-5'));
      expect(json['model_provider']['value']['fields']['api_key'], equals('sk-test-123'));

      expect(json['risk_profile']['mode'], equals('fresh'));
      expect(json['risk_profile']['value'], equals('balanced'));

      expect(json['runtime_profile']['mode'], equals('fresh'));
      expect(json['runtime_profile']['value'], equals('balanced'));

      expect(json['memory']['mode'], equals('fresh'));
      expect(json['memory']['value'], equals('sqlite'));

      expect(json['channels'], hasLength(1));
      expect(json['channels'][0]['mode'], equals('fresh'));
      expect(json['channels'][0]['value']['channel_type'], equals('telegram'));
      expect(json['channels'][0]['value']['alias'], equals('tg'));
      expect(json['channels'][0]['value']['fields']['bot_token'], equals('tg-token-123'));

      expect(json['peer_groups'], isEmpty);

      expect(json['agent']['name'], equals('omnes'));
      expect(json['agent']['system_prompt'], equals('You are Omnes, the personal agent of Alice.'));
      expect(json['agent']['personality_file'], isNull);
      expect(json['agent']['personality_files'], hasLength(4));
      expect(json['agent']['personality_files'][0]['filename'], equals('SOUL.md'));
      expect(json['agent']['personality_files'][0]['content'], equals('Soul text'));

      // Roundtrip check through fromJson
      final restored = BuilderSubmissionDto.fromJson(json);
      expect(restored.agent.name, equals('omnes'));
      expect(restored.agent.personalityFiles, hasLength(4));
      expect(restored.modelProvider.value.providerType, equals('openai'));
    });

    test('BuilderSubmissionDto.withDefaults generates valid minimal structure', () {
      final submission = BuilderSubmissionDto.withDefaults(
        agentName: 'omnes',
        systemPrompt: 'You are Omnes.',
        personalityFiles: const [
          QuickstartPersonalityFileDto(filename: 'SOUL.md', content: 'test'),
        ],
      );

      final json = submission.toJson();
      expect(json['model_provider']['value']['provider_type'], equals('anthropic'));
      expect(json['model_provider']['value']['fields']['api_key'], equals('placeholder-api-key'));
      expect(json['risk_profile']['value'], equals('balanced'));
      expect(json['memory']['value'], equals('sqlite'));
      expect(json['agent']['name'], equals('omnes'));
      expect(json['agent']['personality_files'], hasLength(1));
    });

    test('ValidateResultDto and ApplyResultDto parse backend JSON correctly', () {
      final valOk = ValidateResultDto.fromJson({'kind': 'ok'});
      expect(valOk.isOk, isTrue);
      expect(valOk.errors, isEmpty);

      final valErr = ValidateResultDto.fromJson({
        'kind': 'errors',
        'errors': [
          {'step': 'agent', 'field': 'name', 'message': 'Agent alias already in use'},
        ],
      });
      expect(valErr.isOk, isFalse);
      expect(valErr.errors, hasLength(1));
      expect(valErr.errors[0].step, equals('agent'));
      expect(valErr.errors[0].field, equals('name'));

      final applyOk = ApplyResultDto.fromJson({
        'kind': 'applied',
        'agent': {
          'alias': 'omnes',
          'model_provider': 'anthropic',
          'risk_profile': 'balanced',
          'runtime_profile': 'balanced',
          'channels': <String>[],
          'memory_backend': 'sqlite',
        },
        'daemon_restarted': true,
      });
      expect(applyOk.isApplied, isTrue);
      expect(applyOk.agent?.alias, equals('omnes'));
      expect(applyOk.daemonRestarted, isTrue);
    });
  });
}
