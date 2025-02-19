import 'package:meta/meta.dart';

import 'package:eppo_sdk/src/rust/api/attributes.dart' as core;
import 'package:eppo_sdk/src/rust/api/client.dart' as core;

enum AttributeKind {
  categorical,
  numeric,
}

final class Attributes {
  @internal
  final Map<String, core.AttributeValue> coreAttributes = new Map();

  Attributes();

  void stringAttribute(String key, String value) {
    this.coreAttributes[key] = core.stringAttribute(value);
  }

  void numberAttribute(String key, double value, {AttributeKind kind = AttributeKind.numeric}) {
    this.coreAttributes[key] = kind == AttributeKind.numeric ? core.numericAttribute(value) : core.categoricalNumberAttribute(value);
  }

  void boolAttribute(String key, bool value) {
    this.coreAttributes[key] = core.boolAttribute(value);
  }
}
