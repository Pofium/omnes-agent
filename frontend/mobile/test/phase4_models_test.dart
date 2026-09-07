import 'package:flutter_test/flutter_test.dart';
import 'package:omagent_front/features/approvals/models/pending_approval.dart';
import 'package:omagent_front/features/automation/models/cron_job.dart';
import 'package:omagent_front/features/memory/models/memory_entry.dart';
import 'package:omagent_front/features/stats/models/cost_summary.dart';

void main() {
  group('MemoryEntry model', () {
    test('parses from JSON correctly', () {
      final json = {
        'id': 'mem-1',
        'key': 'user_preference',
        'content': 'Предпочитает ответы на русском языке',
        'category': 'core',
        'timestamp': '2026-09-05T12:00:00.000Z',
        'importance': 0.9,
        'pinned': true,
        'agent_alias': 'chief',
      };

      final entry = MemoryEntry.fromJson(json);
      expect(entry.id, equals('mem-1'));
      expect(entry.key, equals('user_preference'));
      expect(entry.content, equals('Предпочитает ответы на русском языке'));
      expect(entry.category, equals('core'));
      expect(entry.importance, equals(0.9));
      expect(entry.pinned, isTrue);
      expect(entry.agentAlias, equals('chief'));
    });
  });

  group('CronJob & CronRun models', () {
    test('parses CronJob from JSON correctly', () {
      final json = {
        'id': 'job-morning',
        'name': 'Утренняя сводка',
        'expression': '0 9 * * *',
        'command': '',
        'prompt': 'Собери сводку новостей за 24 часа',
        'agent_alias': 'chief',
        'enabled': true,
        'last_status': 'ok',
      };

      final job = CronJob.fromJson(json);
      expect(job.id, equals('job-morning'));
      expect(job.name, equals('Утренняя сводка'));
      expect(job.expression, equals('0 9 * * *'));
      expect(job.prompt, equals('Собери сводку новостей за 24 часа'));
      expect(job.enabled, isTrue);
      expect(job.lastStatus, equals('ok'));
    });

    test('parses CronRun from JSON correctly', () {
      final json = {
        'id': 101,
        'job_id': 'job-morning',
        'started_at': '2026-09-05T09:00:00.000Z',
        'finished_at': '2026-09-05T09:00:05.000Z',
        'status': 'success',
        'output': 'Сводка новостей сформирована',
        'duration_ms': 5000,
      };

      final run = CronRun.fromJson(json);
      expect(run.id, equals(101));
      expect(run.jobId, equals('job-morning'));
      expect(run.status, equals('success'));
      expect(run.durationMs, equals(5000));
      expect(run.output, equals('Сводка новостей сформирована'));
    });
  });

  group('PendingApproval model', () {
    test('parses from JSON correctly', () {
      final json = {
        'run_id': 'run-sop-42',
        'sop_name': 'Deploy to Production',
        'step': 3,
        'total_steps': 5,
        'kind': 'approval',
      };

      final approval = PendingApproval.fromJson(json);
      expect(approval.runId, equals('run-sop-42'));
      expect(approval.sopName, equals('Deploy to Production'));
      expect(approval.step, equals(3));
      expect(approval.totalSteps, equals(5));
      expect(approval.kind, equals('approval'));
    });
  });

  group('CostSummary model', () {
    test('parses from JSON and aggregates correctly', () {
      final json = {
        'total_tokens': 17994,
        'monthly_cost_usd': 0.05,
        'request_count': 1,
        'by_model': {
          'deepseek-chat': {
            'model': 'deepseek-chat',
            'input_tokens': 17976,
            'output_tokens': 18,
            'total_tokens': 17994,
            'cost_usd': 0.05,
            'request_count': 1,
          }
        }
      };

      final cost = CostSummary.fromJson(json);
      expect(cost.totalTokens, equals(17994));
      expect(cost.totalCostUsd, equals(0.05));
      expect(cost.requestCount, equals(1));
      expect(cost.byModel.containsKey('deepseek-chat'), isTrue);

      final modelUsage = cost.byModel['deepseek-chat']!;
      expect(modelUsage.model, equals('deepseek-chat'));
      expect(modelUsage.inputTokens, equals(17976));
      expect(modelUsage.outputTokens, equals(18));
      expect(modelUsage.totalTokens, equals(17994));
    });
  });
}
