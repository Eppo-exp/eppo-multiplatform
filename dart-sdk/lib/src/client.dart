import 'dart:convert' show jsonDecode;
import 'package:meta/meta.dart' show internal;

import './rust/frb_generated.dart' as core;
import './rust/api/client.dart' as core;

import './subject.dart' show Subject;
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

  @override
  String toString() => 'variation: $variation, action: $action';

  @override
  operator ==(o) => o is EvaluationResult<T> && o.variation == this.variation && o.action == this.action;

  @override
  int get hashCode => Object.hash(variation, action);
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
      String logLevel = 'info',
  }) : _logger = logger {
    this._initialized = globalInit().then((_) async {
        this._core = core.CoreClient(
          sdkKey: sdkKey,
          baseUrl: baseUrl?.toString(),
          pollIntervalMs: pollInterval.inMilliseconds,
          pollJitterMs: pollJitter.inMilliseconds,
          logLevel: logLevel,
        );
    });
  }

  /// Returns a Future that resolves when the client completes initialization,
  /// receives a configuration, and is ready to serve assignment and bandits.
  Future<void> whenReady() async {
    await this._initialized;
    await this._core!.waitForInitialization();
  }

  String stringAssignment(String flagKey, Subject subject, String defaultValue) {
    final core = this._core;
    if (core == null) {
      return defaultValue;
    }

    final (result, event) = core.stringAssignment(flagKey, subject.key, subject.attributes.coreAttributes);
    this.logAssignmentEvent(event);
    return result ?? defaultValue;
  }

  double numericAssignment(String flagKey, Subject subject, double defaultValue) {
    final core = this._core;
    if (core == null) {
      return defaultValue;
    }

    final (result, event) = core.numericAssignment(flagKey, subject.key, subject.attributes.coreAttributes);
    this.logAssignmentEvent(event);
    return result ?? defaultValue;
  }

  int integerAssignment(String flagKey, Subject subject, int defaultValue) {
    final core = this._core;
    if (core == null) {
      return defaultValue;
    }

    final (result, event) = core.integerAssignment(flagKey, subject.key, subject.attributes.coreAttributes);
    this.logAssignmentEvent(event);
    return result ?? defaultValue;
  }

  bool booleanAssignment(String flagKey, Subject subject, bool defaultValue) {
    final core = this._core;
    if (core == null) {
      return defaultValue;
    }

    final (result, event) = core.booleanAssignment(flagKey, subject.key, subject.attributes.coreAttributes);
    this.logAssignmentEvent(event);
    return result ?? defaultValue;
  }

  dynamic jsonAssignment(String flagKey, Subject subject, dynamic defaultValue) {
    final core = this._core;
    if (core == null) {
      return defaultValue;
    }

    final (result, event) = core.jsonAssignment(flagKey, subject.key, subject.attributes.coreAttributes);
    this.logAssignmentEvent(event);
    if (result != null) {
      return jsonDecode(result);
    } else {
      return defaultValue;
    }
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

    this.logAssignmentEvent(result.assignmentEvent);
    this.logBanditEvent(result.banditEvent);

    return EvaluationResult(
      variation: result.variation,
      action: result.action,
    );
  }

  @internal
  void logAssignmentEvent(Map<String, dynamic>? event) {
    if (event != null) {
      try {
        this._logger?.logAssignment(event);
      } catch (e) {
      }
    }
  }

  @internal
  void logBanditEvent(Map<String, dynamic>? event) {
    if (event != null) {
      try {
        this._logger?.logBanditAction(event);
      } catch (e) {
      }
    }
  }
}
