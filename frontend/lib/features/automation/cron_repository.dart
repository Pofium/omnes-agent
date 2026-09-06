// Repository for managing automated cron schedules via /api/cron.

import '../../core/gateway/gateway_config.dart';
import '../../core/gateway/gateway_http.dart';
import 'models/cron_job.dart';

class CronRepository {
  final GatewayHttpClient _http;

  CronRepository({GatewayHttpClient? http}) : _http = http ?? GatewayHttpClient();

  /// Lists all cron jobs.
  Future<List<CronJob>> listJobs() async {
    final rawList = await _http.cronList();
    return rawList.map((j) => CronJob.fromJson(j)).toList();
  }

  /// Adds a new agent prompt cron job.
  Future<bool> createPromptJob({
    required String name,
    required String schedule,
    required String prompt,
    String? agentAlias,
  }) async {
    final alias = (agentAlias != null && agentAlias.isNotEmpty)
        ? agentAlias
        : await GatewayConfig.getActiveAgent();

    final body = {
      'agent': alias,
      'name': name,
      'schedule': {'cron': schedule},
      'prompt': prompt,
      'job_type': 'agent',
      'command': '',
    };
    return _http.cronAdd(body);
  }

  /// Toggles enabled state of a job.
  Future<bool> toggleJob(String id, bool enabled) async {
    return _http.cronPatch(id, {'enabled': enabled});
  }

  /// Triggers a job execution immediately.
  Future<bool> runJobNow(String id) async {
    return _http.cronRun(id);
  }

  /// Deletes a cron job.
  Future<bool> deleteJob(String id) async {
    return _http.cronDelete(id);
  }

  /// Gets recent execution runs for a cron job.
  Future<List<CronRun>> getJobRuns(String id) async {
    final rawList = await _http.cronRuns(id);
    return rawList.map((j) => CronRun.fromJson(j)).toList();
  }

  void dispose() {
    _http.dispose();
  }
}
