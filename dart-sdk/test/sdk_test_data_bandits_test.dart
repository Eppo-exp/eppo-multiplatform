import 'dart:io';
import 'dart:convert';

import 'package:test/test.dart';

import 'package:eppo_sdk/eppo_sdk.dart';

Future<void> main() async {
  final client = new EppoClient(
    sdkKey: 'test',
    baseUrl: Uri.parse('http://localhost:8378/bandit/api'),
    logger: AssignmentLogger());
  await client.whenReady();

  await for (final FileSystemEntity entity in Directory('../sdk-test-data/ufc/bandit-tests').list()) {
    if (entity is File) {
      final s = await entity.readAsString();
      final testFile = jsonDecode(s);

      group(entity.path, () {
          for (final testCase in testFile['subjects']) {
            test(testCase['subjectKey'], () {
                final subject = Subject(testCase['subjectKey'], contextAttributesToAttributes(testCase['subjectAttributes']));

                final dynamic actions = {for (final action in testCase['actions']) action['actionKey'] as String: contextAttributesToAttributes(action)};

                final result = client.banditAction(testFile['flag'], subject, actions, testFile['defaultValue']);

                expect(result, EvaluationResult<String>(variation: testCase['assignment']['variation'], action: testCase['assignment']['action']));
            });
          }
      });
    }
  }
}

Attributes contextAttributesToAttributes(dynamic context) {
  final attributes = Attributes();

  for (final MapEntry(:key, :value) in context['numericAttributes'].entries) {
    if (value is num) {
      attributes.numberAttribute(key, value.toDouble());
    }
  }

  for (final MapEntry(:key, :value) in context['categoricalAttributes'].entries) {
    if (value is String) {
      attributes.stringAttribute(key, value);
    } else if (value is bool) {
      attributes.boolAttribute(key, value);
    } else if (value is num) {
      attributes.numberAttribute(key, value.toDouble(), kind: AttributeKind.categorical);
    }
  }

  return attributes;
}
