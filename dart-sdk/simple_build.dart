// This file is a copy from flutter_rust_bridge_utils[1] to break a
// dependency on unpublished package.
//
// [1]: https://github.com/fzyzcjy/flutter_rust_bridge/blob/05247dd90a7fcb2512c5ede13cf00271b0eb548e/frb_utils/lib/src/simple_build_utils.dart

// Copyright (c) 2021 fzyzcjy
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

import 'dart:io';

import 'package:flutter_rust_bridge/src/cli/run_command.dart';
import 'package:native_assets_cli/native_assets_cli.dart';

/// Utilities that can be used in `build.dart`.
/// Do not export this function for public use yet, since Dart's `build.dart` support
/// is still experimental.
// ref: https://github.com/dart-lang/native/blob/main/pkgs/native_assets_cli/example/native_add_library/build.dart
void simpleBuild(List<String> args, {List<String> features = const []}) async {
  final buildConfig = await BuildConfig.fromArgs(args);
  final buildOutput = BuildOutput();

  final rustCrateDir = buildConfig.packageRoot.resolve('rust');

  final cargoNightly =
      Platform.environment['FRB_SIMPLE_BUILD_CARGO_NIGHTLY'] == '1';
  final cargoExtraArgs =
      Platform.environment['FRB_SIMPLE_BUILD_CARGO_EXTRA_ARGS']?.split(' ') ??
          const <String>[];
  final skip = Platform.environment['FRB_SIMPLE_BUILD_SKIP'] == '1';
  final rustflags = Platform.environment['RUSTFLAGS'];

  if (skip) {
    print(
        'frb_utils::simpleBuild SKIP BUILD since environment variable requires this');
  } else {
    final featureArgs = features.expand((x) => ['--features', x]).toList();
    await runCommand(
      'cargo',
      [
        if (cargoNightly) '+nightly',
        'build',
        '--release',
        ...cargoExtraArgs,
        ...featureArgs,
      ],
      pwd: rustCrateDir.toFilePath(),
      printCommandInStderr: true,
      env: {
        // Though runCommand auto pass environment variable to commands,
        // we do this to explicitly show this important flag
        if (rustflags != null) 'RUSTFLAGS': rustflags,
      },
    );
  }

  final dependencies = {
    rustCrateDir,
    buildConfig.packageRoot.resolve('build.rs'),
  };
  print('dependencies: $dependencies');
  buildOutput.dependencies.dependencies.addAll(dependencies);

  await buildOutput.writeToFile(outDir: buildConfig.outDir);
}
