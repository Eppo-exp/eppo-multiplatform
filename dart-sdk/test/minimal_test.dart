import 'dart:async';
import 'dart:io';

import 'package:eppo_sdk/src/rust/api/client.dart';
import 'package:eppo_sdk/src/rust/frb_generated.dart';
import 'package:test/test.dart';

Future<void> main() async {
  print('Action: Init rust (before)');
  await EppoSdk.init();
  print('Action: Init rust (after)');

  var client = EppoClient(sdkKey: "<redacted>");

  await client.waitForConfiguration();

  var value = client.boolAssignment("a-boolean-flag", "subject1", false);
  print('value: ${value}');
}
