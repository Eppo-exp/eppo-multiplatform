import 'package:eppo_sdk/src/rust/frb_generated.dart' as core;
import 'package:eppo_sdk/src/rust/api.dart' show BanditResult;
import 'package:eppo_sdk/src/rust/api/client.dart' as core;
import 'package:eppo_sdk/src/rust/api/attributes.dart' as core;

import './subject.dart' show Subject, AttributeKind;
import './attributes.dart' show Attributes;
import './logger.dart' show AssignmentLogger;

Future<void>? libraryInitialized;
Future<void> globalInit() async {
  libraryInitialized ??= core.RustLib.init();
  await libraryInitialized;
}

final class EvaluationResult<T> {
  final T variation;
  final String? action;
  final Map<String, dynamic>? evaluationDetails;
  EvaluationResult({required this.variation, this.action, this.evaluationDetails});
}

final class EppoClient {
  core.CoreClient? _core;
  final AssignmentLogger? _logger;
  late Future<void> _initialized;

  EppoClient({
      required String sdkKey,
      AssignmentLogger? logger,
      Uri? baseUrl,
      Duration pollInterval = const Duration(seconds: 30),
      Duration pollJitter = const Duration(seconds: 3),
  }) : _logger = logger {
    this._initialized = globalInit().then((_) async {
        this._core = core.CoreClient(
          sdkKey: sdkKey,
          baseUrl: baseUrl.toString(),
          pollInterval: pollInterval,
          pollJitter: pollJitter,
        );
    });
  }

  /// Returns a Future that resolve when the client completed initialization,
  /// received a configuration, and is ready to serve assignment and bandits.
  Future<void> whenReady() async {
    await this._initialized;
    await this._core!.waitForInitialization();
  }

  bool boolAssignment(String flagKey, Subject subject, bool defaultValue) {
    final core = this._core;
    if (core == null) {
      return defaultValue;
    }

    final (result, event) = core.boolAssignment(flagKey, subject.key, subject.attributes.coreAttributes);

    if (event != null) {
      try {
        this._logger?.logAssignment(event);
      } catch (e) {
      }
    }

    return result ?? defaultValue;
  }

  EvaluationResult<String> banditAction(String flagKey, Subject subject, Map<String, Attributes> actions, String defaultVariation) {
    final core = this._core;
    if (core == null) {
      return EvaluationResult(variation: defaultVariation);
    }

    final result = core.banditAction(
      flagKey,
      subject.key,
      subject.attributes.coreAttributes,
      actions.map((key, value) => MapEntry(key, value.coreAttributes)),
      defaultVariation);

    final assignmentEvent = result.assignmentEvent;
    if (assignmentEvent != null) {
      this._logger?.logAssignment(assignmentEvent);
    }
    final banditEvent = result.banditEvent;
    if (banditEvent != null) {
      this._logger?.logBanditAction(banditEvent);
    }

    return EvaluationResult(
      variation: result.variation,
      action: result.action,
    );
  }
}
