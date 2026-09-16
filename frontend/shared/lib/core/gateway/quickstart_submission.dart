// Quickstart Submission DTOs matching Rust serde specifications in omnesagent-config::presets.

/// Selects either an existing configured value or a fresh builder value.
/// Serializes to `{"mode": "existing"|"fresh", "value": ...}` matching Rust `SelectorChoice<T>`.
class SelectorChoiceDto<T> {
  final String mode; // 'existing' | 'fresh'
  final T value;

  const SelectorChoiceDto({
    required this.mode,
    required this.value,
  });

  const SelectorChoiceDto.fresh(this.value) : mode = 'fresh';
  const SelectorChoiceDto.existing(this.value) : mode = 'existing';

  Map<String, dynamic> toJson(Object? Function(T value) valueToJson) => {
    'mode': mode,
    'value': valueToJson(value),
  };

  factory SelectorChoiceDto.fromJson(
    Map<String, dynamic> json,
    T Function(dynamic raw) valueFromJson,
  ) {
    return SelectorChoiceDto<T>(
      mode: json['mode'] as String? ?? 'fresh',
      value: valueFromJson(json['value']),
    );
  }
}

/// Model provider choice submitted in BuilderSubmissionDto.
class ModelProviderChoiceDto {
  final String providerType;
  final String alias;
  final String model;
  final Map<String, String> fields;

  const ModelProviderChoiceDto({
    required this.providerType,
    this.alias = 'default',
    required this.model,
    this.fields = const {},
  });

  Map<String, dynamic> toJson() => {
    'provider_type': providerType,
    'alias': alias,
    'model': model,
    if (fields.isNotEmpty) 'fields': fields,
  };

  factory ModelProviderChoiceDto.fromJson(Map<String, dynamic> json) {
    final rawFields = json['fields'] as Map<String, dynamic>?;
    final parsedFields = <String, String>{};
    if (rawFields != null) {
      for (final entry in rawFields.entries) {
        parsedFields[entry.key] = entry.value.toString();
      }
    }
    return ModelProviderChoiceDto(
      providerType: json['provider_type'] as String? ?? '',
      alias: json['alias'] as String? ?? 'default',
      model: json['model'] as String? ?? '',
      fields: parsedFields,
    );
  }
}

/// Channel selection submitted in BuilderSubmissionDto.
class ChannelQuickStartDto {
  final String channelType;
  final String alias;
  final Map<String, String> fields;

  const ChannelQuickStartDto({
    required this.channelType,
    required this.alias,
    this.fields = const {},
  });

  Map<String, dynamic> toJson() => {
    'channel_type': channelType,
    'alias': alias,
    if (fields.isNotEmpty) 'fields': fields,
  };

  factory ChannelQuickStartDto.fromJson(Map<String, dynamic> json) {
    final rawFields = json['fields'] as Map<String, dynamic>?;
    final parsedFields = <String, String>{};
    if (rawFields != null) {
      for (final entry in rawFields.entries) {
        parsedFields[entry.key] = entry.value.toString();
      }
    }
    return ChannelQuickStartDto(
      channelType: json['channel_type'] as String? ?? '',
      alias: json['alias'] as String? ?? '',
      fields: parsedFields,
    );
  }
}

/// Peer group entry staged in BuilderSubmissionDto.
class QuickstartPeerGroupDto {
  final String name;
  final String channel;
  final List<String> externalPeers;
  final List<String> ignore;

  const QuickstartPeerGroupDto({
    required this.name,
    required this.channel,
    this.externalPeers = const [],
    this.ignore = const [],
  });

  Map<String, dynamic> toJson() => {
    'name': name,
    'channel': channel,
    'external_peers': externalPeers,
    'ignore': ignore,
  };

  factory QuickstartPeerGroupDto.fromJson(Map<String, dynamic> json) {
    return QuickstartPeerGroupDto(
      name: json['name'] as String? ?? '',
      channel: json['channel'] as String? ?? '',
      externalPeers: (json['external_peers'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? const [],
      ignore: (json['ignore'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? const [],
    );
  }
}

/// Personality file staged for write in Quickstart apply.
class QuickstartPersonalityFileDto {
  final String filename;
  final String content;

  const QuickstartPersonalityFileDto({
    required this.filename,
    required this.content,
  });

  Map<String, dynamic> toJson() => {
    'filename': filename,
    'content': content,
  };

  factory QuickstartPersonalityFileDto.fromJson(Map<String, dynamic> json) {
    return QuickstartPersonalityFileDto(
      filename: json['filename'] as String? ?? '',
      content: json['content'] as String? ?? '',
    );
  }
}

/// Agent identity payload for BuilderSubmissionDto.
class AgentIdentityDto {
  final String name;
  final String systemPrompt;
  final String? personalityFile;
  final List<QuickstartPersonalityFileDto> personalityFiles;

  const AgentIdentityDto({
    required this.name,
    required this.systemPrompt,
    this.personalityFile,
    this.personalityFiles = const [],
  });

  Map<String, dynamic> toJson() => {
    'name': name,
    'system_prompt': systemPrompt,
    'personality_file': personalityFile,
    'personality_files': personalityFiles.map((f) => f.toJson()).toList(),
  };

  factory AgentIdentityDto.fromJson(Map<String, dynamic> json) {
    final rawFiles = json['personality_files'] as List<dynamic>?;
    return AgentIdentityDto(
      name: json['name'] as String? ?? 'omnes',
      systemPrompt: json['system_prompt'] as String? ?? '',
      personalityFile: json['personality_file'] as String?,
      personalityFiles: rawFiles != null
          ? rawFiles.map((f) => QuickstartPersonalityFileDto.fromJson(f as Map<String, dynamic>)).toList()
          : const [],
    );
  }
}

/// Complete builder submission consumed by POST /api/quickstart/apply and validate.
class BuilderSubmissionDto {
  final SelectorChoiceDto<ModelProviderChoiceDto> modelProvider;
  final SelectorChoiceDto<String> riskProfile;
  final SelectorChoiceDto<String> runtimeProfile;
  final SelectorChoiceDto<String> memory;
  final List<SelectorChoiceDto<ChannelQuickStartDto>> channels;
  final List<QuickstartPeerGroupDto> peerGroups;
  final AgentIdentityDto agent;

  const BuilderSubmissionDto({
    required this.modelProvider,
    required this.riskProfile,
    required this.runtimeProfile,
    required this.memory,
    this.channels = const [],
    this.peerGroups = const [],
    required this.agent,
  });

  /// Creates a submission with safe default presets (mirrors Rust system test submission()).
  factory BuilderSubmissionDto.withDefaults({
    required String agentName,
    required String systemPrompt,
    List<QuickstartPersonalityFileDto> personalityFiles = const [],
    String modelProviderType = 'anthropic',
    String modelName = 'claude-sonnet-4-5',
    Map<String, String>? modelFields,
    List<SelectorChoiceDto<ChannelQuickStartDto>> channels = const [],
  }) {
    return BuilderSubmissionDto(
      modelProvider: SelectorChoiceDto.fresh(
        ModelProviderChoiceDto(
          providerType: modelProviderType,
          alias: modelProviderType,
          model: modelName,
          fields: modelFields ?? {'api_key': 'placeholder-api-key'},
        ),
      ),
      riskProfile: const SelectorChoiceDto.fresh('balanced'),
      runtimeProfile: const SelectorChoiceDto.fresh('balanced'),
      memory: const SelectorChoiceDto.fresh('sqlite'),
      channels: channels,
      peerGroups: const [],
      agent: AgentIdentityDto(
        name: agentName,
        systemPrompt: systemPrompt,
        personalityFile: null,
        personalityFiles: personalityFiles,
      ),
    );
  }

  Map<String, dynamic> toJson() => {
    'model_provider': modelProvider.toJson((v) => v.toJson()),
    'risk_profile': riskProfile.toJson((v) => v),
    'runtime_profile': runtimeProfile.toJson((v) => v),
    'memory': memory.toJson((v) => v),
    'channels': channels.map((c) => c.toJson((v) => v.toJson())).toList(),
    'peer_groups': peerGroups.map((p) => p.toJson()).toList(),
    'agent': agent.toJson(),
  };

  factory BuilderSubmissionDto.fromJson(Map<String, dynamic> json) {
    return BuilderSubmissionDto(
      modelProvider: SelectorChoiceDto.fromJson(
        json['model_provider'] as Map<String, dynamic>,
        (v) => ModelProviderChoiceDto.fromJson(v as Map<String, dynamic>),
      ),
      riskProfile: SelectorChoiceDto.fromJson(
        json['risk_profile'] as Map<String, dynamic>,
        (v) => v.toString(),
      ),
      runtimeProfile: SelectorChoiceDto.fromJson(
        json['runtime_profile'] as Map<String, dynamic>,
        (v) => v.toString(),
      ),
      memory: SelectorChoiceDto.fromJson(
        json['memory'] as Map<String, dynamic>,
        (v) => v.toString(),
      ),
      channels: (json['channels'] as List<dynamic>?)
              ?.map((c) => SelectorChoiceDto.fromJson(
                    c as Map<String, dynamic>,
                    (v) => ChannelQuickStartDto.fromJson(v as Map<String, dynamic>),
                  ))
              .toList() ??
          const [],
      peerGroups: (json['peer_groups'] as List<dynamic>?)
              ?.map((p) => QuickstartPeerGroupDto.fromJson(p as Map<String, dynamic>))
              .toList() ??
          const [],
      agent: AgentIdentityDto.fromJson(json['agent'] as Map<String, dynamic>),
    );
  }
}

/// Error returned from validate or apply endpoints.
class QuickstartErrorDto {
  final String step;
  final String field;
  final String message;

  const QuickstartErrorDto({
    required this.step,
    required this.field,
    required this.message,
  });

  factory QuickstartErrorDto.fromJson(Map<String, dynamic> json) {
    return QuickstartErrorDto(
      step: json['step'] as String? ?? '',
      field: json['field'] as String? ?? '',
      message: json['message'] as String? ?? '',
    );
  }

  Map<String, dynamic> toJson() => {
    'step': step,
    'field': field,
    'message': message,
  };
}

/// Result of POST /api/quickstart/validate.
class ValidateResultDto {
  final bool isOk;
  final List<QuickstartErrorDto> errors;

  const ValidateResultDto({
    required this.isOk,
    this.errors = const [],
  });

  factory ValidateResultDto.fromJson(Map<String, dynamic> json) {
    final kind = json['kind'] as String? ?? 'ok';
    if (kind == 'ok') {
      return const ValidateResultDto(isOk: true, errors: []);
    }
    final rawErrors = json['errors'] as List<dynamic>?;
    return ValidateResultDto(
      isOk: false,
      errors: rawErrors != null
          ? rawErrors.map((e) => QuickstartErrorDto.fromJson(e as Map<String, dynamic>)).toList()
          : const [],
    );
  }
}

/// Information about the successfully applied agent from POST /api/quickstart/apply.
class AppliedAgentDto {
  final String alias;
  final String modelProvider;
  final String riskProfile;
  final String runtimeProfile;
  final List<String> channels;
  final String memoryBackend;

  const AppliedAgentDto({
    required this.alias,
    required this.modelProvider,
    required this.riskProfile,
    required this.runtimeProfile,
    required this.channels,
    required this.memoryBackend,
  });

  factory AppliedAgentDto.fromJson(Map<String, dynamic> json) {
    return AppliedAgentDto(
      alias: json['alias'] as String? ?? '',
      modelProvider: json['model_provider'] as String? ?? '',
      riskProfile: json['risk_profile'] as String? ?? '',
      runtimeProfile: json['runtime_profile'] as String? ?? '',
      channels: (json['channels'] as List<dynamic>?)?.map((e) => e.toString()).toList() ?? const [],
      memoryBackend: json['memory_backend'] as String? ?? '',
    );
  }
}

/// Result of POST /api/quickstart/apply.
class ApplyResultDto {
  final bool isApplied;
  final AppliedAgentDto? agent;
  final bool daemonRestarted;
  final List<QuickstartErrorDto> errors;

  const ApplyResultDto({
    required this.isApplied,
    this.agent,
    this.daemonRestarted = false,
    this.errors = const [],
  });

  factory ApplyResultDto.fromJson(Map<String, dynamic> json) {
    final kind = json['kind'] as String? ?? 'applied';
    if (kind == 'applied') {
      return ApplyResultDto(
        isApplied: true,
        agent: json['agent'] != null
            ? AppliedAgentDto.fromJson(json['agent'] as Map<String, dynamic>)
            : null,
        daemonRestarted: json['daemon_restarted'] as bool? ?? false,
        errors: const [],
      );
    }
    final rawErrors = json['errors'] as List<dynamic>?;
    return ApplyResultDto(
      isApplied: false,
      errors: rawErrors != null
          ? rawErrors.map((e) => QuickstartErrorDto.fromJson(e as Map<String, dynamic>)).toList()
          : const [],
    );
  }
}
