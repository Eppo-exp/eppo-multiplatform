import 'package:meta/meta.dart';

import './attributes.dart' show Attributes, AttributeKind;

final class Subject {
  @internal
  final String key;
  @internal
  final Attributes attributes;

  Subject(this.key, [attributes]) : attributes = attributes ?? Attributes();

  void stringAttribute(String key, String value) {
    this.attributes.stringAttribute(key, value);
  }

  void numberAttribute(String key, double value, {AttributeKind kind = AttributeKind.numeric}) {
    this.attributes.numberAttribute(key, value, kind: kind);
  }

  void boolAttribute(String key, bool value) {
    this.attributes.boolAttribute(key, value);
  }
}
