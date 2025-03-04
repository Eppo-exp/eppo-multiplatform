import 'dart:io';
import 'dart:convert';

import 'package:test/test.dart';

import 'package:eppo_sdk/eppo_sdk.dart';

Future<void> main() async {
  final client = new EppoClient(
    sdkKey: 'test',
    baseUrl: Uri.parse('http://localhost:8378/ufc/api'),
    logger: AssignmentLogger());
  await client.whenReady();

  await for (final FileSystemEntity entity in Directory('../sdk-test-data/ufc/tests').list()) {
    if (entity is File) {
      final s = await entity.readAsString();
      final testFile = jsonDecode(s);

      group(entity.path, () {
          for (final testCase in testFile['subjects']) {
            test(testCase['subjectKey'], () {
                final subject = Subject(testCase['subjectKey']);
                for (final MapEntry(:key, :value) in testCase['subjectAttributes'].entries) {
                  if (value is String) {
                    subject.stringAttribute(key, value);
                  } else if (value is num) {
                    subject.numberAttribute(key, value.toDouble());
                  } else if (value is bool) {
                    subject.boolAttribute(key, value);
                  }
                }

                dynamic result;
                switch (testFile['variationType']) {
                  case 'STRING':
                  result = client.stringAssignment(testFile['flag'], subject, testFile['defaultValue']);
                  break;
                  case 'NUMERIC':
                  result = client.numericAssignment(testFile['flag'], subject, testFile['defaultValue']);
                  break;
                  case 'INTEGER':
                  result = client.integerAssignment(testFile['flag'], subject, testFile['defaultValue']);
                  break;
                  case 'BOOLEAN':
                  result = client.booleanAssignment(testFile['flag'], subject, testFile['defaultValue']);
                  break;
                  case 'JSON':
                  result = client.jsonAssignment(testFile['flag'], subject, testFile['defaultValue']);
                  break;
                }

                expect(result, testCase['assignment']);
            });
          }
      });
    }
  }
}
