// DTO models for Handy offline STT dictation and mic calibration.

class HandyStatusDto {
  final bool supported;
  final bool installed;
  final String? version;
  final String? installPath;
  final String? exePath;
  final bool portable;
  final bool running;
  final String? customExePath;
  final HandyLatestManifestDto? latest;
  final HandyJobStatusDto? activeJob;
  final HandyConfigSummaryDto config;

  const HandyStatusDto({
    required this.supported,
    required this.installed,
    this.version,
    this.installPath,
    this.exePath,
    required this.portable,
    required this.running,
    this.customExePath,
    this.latest,
    this.activeJob,
    required this.config,
  });

  factory HandyStatusDto.fromJson(Map<String, dynamic> json) {
    return HandyStatusDto(
      supported: json['supported'] as bool? ?? false,
      installed: json['installed'] as bool? ?? false,
      version: json['version'] as String?,
      installPath: json['install_path'] as String?,
      exePath: json['exe_path'] as String?,
      portable: json['portable'] as bool? ?? false,
      running: json['running'] as bool? ?? false,
      customExePath: json['custom_exe_path'] as String?,
      latest: json['latest'] != null
          ? HandyLatestManifestDto.fromJson(json['latest'] as Map<String, dynamic>)
          : null,
      activeJob: json['active_job'] != null
          ? HandyJobStatusDto.fromJson(json['active_job'] as Map<String, dynamic>)
          : null,
      config: json['config'] != null
          ? HandyConfigSummaryDto.fromJson(json['config'] as Map<String, dynamic>)
          : const HandyConfigSummaryDto(),
    );
  }
}

class HandyLatestManifestDto {
  final String version;
  final String? pubDate;
  final bool updateAvailable;
  final String url;
  final String? signature;
  final int? sizeBytes;

  const HandyLatestManifestDto({
    required this.version,
    this.pubDate,
    required this.updateAvailable,
    required this.url,
    this.signature,
    this.sizeBytes,
  });

  factory HandyLatestManifestDto.fromJson(Map<String, dynamic> json) {
    return HandyLatestManifestDto(
      version: json['version'] as String? ?? '',
      pubDate: json['pub_date'] as String?,
      updateAvailable: json['update_available'] as bool? ?? false,
      url: json['url'] as String? ?? '',
      signature: json['signature'] as String?,
      sizeBytes: json['size_bytes'] as int?,
    );
  }
}

class HandyJobStatusDto {
  final String jobId;
  final String kind; // "install" | "update" | "uninstall"
  final String stage; // "download" | "verify" | "install" | "postcheck" | "done" | "failed"
  final int percent;
  final int downloadedBytes;
  final int totalBytes;
  final String message;
  final String? error;

  const HandyJobStatusDto({
    required this.jobId,
    required this.kind,
    required this.stage,
    required this.percent,
    required this.downloadedBytes,
    required this.totalBytes,
    required this.message,
    this.error,
  });

  factory HandyJobStatusDto.fromJson(Map<String, dynamic> json) {
    return HandyJobStatusDto(
      jobId: json['job_id'] as String? ?? '',
      kind: json['kind'] as String? ?? 'install',
      stage: json['stage'] as String? ?? 'idle',
      percent: (json['percent'] as num?)?.toInt() ?? 0,
      downloadedBytes: (json['downloaded_bytes'] as num?)?.toInt() ?? 0,
      totalBytes: (json['total_bytes'] as num?)?.toInt() ?? 0,
      message: json['message'] as String? ?? '',
      error: json['error'] as String?,
    );
  }

  bool get isInProgress => stage == 'download' || stage == 'verify' || stage == 'install' || stage == 'postcheck';
  bool get isDone => stage == 'done';
  bool get isFailed => stage == 'failed';
}

class HandyConfigSummaryDto {
  final bool enabled;
  final String installPolicy;
  final bool autoStartWithOs;

  const HandyConfigSummaryDto({
    this.enabled = false,
    this.installPolicy = 'ask',
    this.autoStartWithOs = false,
  });

  factory HandyConfigSummaryDto.fromJson(Map<String, dynamic> json) {
    return HandyConfigSummaryDto(
      enabled: json['enabled'] as bool? ?? false,
      installPolicy: json['install_policy'] as String? ?? 'ask',
      autoStartWithOs: json['auto_start_with_os'] as bool? ?? false,
    );
  }
}
