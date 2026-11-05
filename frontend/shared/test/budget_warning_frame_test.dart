import 'package:flutter_test/flutter_test.dart';
import 'package:omnes_shared/core/gateway/models/gateway_frame.dart';

void main() {
  group('BudgetWarningFrame (#11692)', () {
    test('parses the daily warning emitted by the gateway', () {
      final frame = GatewayFrame.fromJson({
        'type': 'budget_warning',
        'current_usd': 8.25,
        'limit_usd': 10.0,
        'period': 'day',
      });

      expect(frame, isA<BudgetWarningFrame>());
      final warning = frame as BudgetWarningFrame;
      expect(warning.currentUsd, equals(8.25));
      expect(warning.limitUsd, equals(10.0));
      expect(warning.period, equals('day'));
    });

    test('parses the monthly warning period', () {
      final frame = GatewayFrame.fromJson({
        'type': 'budget_warning',
        'current_usd': 240,
        'limit_usd': 300,
        'period': 'month',
      });

      final warning = frame as BudgetWarningFrame;
      expect(warning.period, equals('month'));
      // Integer JSON numbers are coerced to double.
      expect(warning.currentUsd, equals(240.0));
    });

    test('defaults missing fields instead of throwing', () {
      final frame = GatewayFrame.fromJson({'type': 'budget_warning'});

      expect(frame, isA<BudgetWarningFrame>());
      final warning = frame as BudgetWarningFrame;
      expect(warning.currentUsd, equals(0.0));
      expect(warning.limitUsd, equals(0.0));
      expect(warning.period, equals(''));
    });
  });
}
