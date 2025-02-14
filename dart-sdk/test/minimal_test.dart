import 'dart:async';
import 'dart:io';

import 'package:test/test.dart';

import 'package:eppo_sdk/eppo_sdk.dart';

class MyAssignmentLogger extends AssignmentLogger {
  const MyAssignmentLogger();
  void logAssignment(Map<String, dynamic> event) {
    print('MyAssignmentLogger.logAssignment($event)');
  }
  void logBanditAction(Map<String, dynamic> event) {
    print('MyAssignmentLogger.logBanditAction($event)');
  }
}

EppoClient makeClientFor(String suite, [AssignmentLogger logger = const MyAssignmentLogger()]) {
  return new EppoClient(
    sdkKey: 'test',
    baseUrl: Uri.parse('http://localhost:8378/${suite}/api'),
    logger: logger);
}

Future<void> main() async {
  test('default constructor succeeds', () {
      final client = new EppoClient(sdkKey: 'test');
  });

  test('has .whenReady()', () async {
      final client = makeClientFor('ufc');
      await client.whenReady();
  });

  test('serves default value before initialized', () {
      final client = makeClientFor('ufc');
      final assignment = client.boolAssignment('kill-switch', Subject('subject1'), true);
      expect(assignment, true);
  });

  test('serves non-default value after initialized', () async {
      final client = makeClientFor('ufc');
      await client.whenReady();
      final assignment = client.boolAssignment('kill-switch', Subject('subject1'), true);
      expect(assignment, false);
  });

  test('serves non-default value with targeting rules', () async {
      final client = makeClientFor('ufc');
      await client.whenReady();
      final assignment = client.boolAssignment(
        'kill-switch',
        Subject('subject1')
          ..stringAttribute('country', 'US'),
        false);
      expect(assignment, true);
  });

  test('returns bandit', () async {
      final client = makeClientFor('bandit');
      await client.whenReady();
      final bandit = client.banditAction(
        'banner_bandit_flag',
        Subject('alice')
          ..numberAttribute('age', 25)
          ..stringAttribute('country', 'USA')
          ..stringAttribute('genderIdentity', 'female'),
        {
          'nike': Attributes()
            ..numberAttribute('brand_affinity', 1.0)
            ..stringAttribute('loyalty_tier', 'gold')
            ..boolAttribute('purchased_last_30_days', true),
        },
        'control',
      );

      expect(bandit.variation, 'banner_bandit');
      expect(bandit.action, 'nike');
  });
}
