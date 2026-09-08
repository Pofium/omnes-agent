import 'dart:async';
import 'package:flutter/foundation.dart';

/// Callback invoked when a DOM element is picked in the browser.
typedef OnElementPickedCallback = void Function(String selector, String tag, String text);

/// Controller managing Web Browser view and iframe integration for OmnesAgent Web ADE.
class WebBrowserController extends ChangeNotifier {
  bool _isInitialized = false;
  bool _isLoading = false;
  bool _isPickerActive = false;
  String _currentUrl = 'http://localhost:3000';
  String? _hoveredSelector;
  String? _selectedSelector;
  String? _errorMessage;

  OnElementPickedCallback? onElementPicked;

  bool get isInitialized => _isInitialized;
  bool get isLoading => _isLoading;
  bool get isPickerActive => _isPickerActive;
  String get currentUrl => _currentUrl;
  String? get hoveredSelector => _hoveredSelector;
  String? get selectedSelector => _selectedSelector;
  String? get errorMessage => _errorMessage;

  /// Initializes the Web browser controller.
  Future<bool> initialize({String initialUrl = 'http://localhost:3000'}) async {
    _currentUrl = initialUrl;
    _isInitialized = true;
    _errorMessage = null;
    notifyListeners();
    return true;
  }

  /// Loads a URL into the browser iframe.
  Future<void> loadUrl(String url) async {
    var target = url.trim();
    if (target.isEmpty) return;
    if (!target.startsWith('http://') && !target.startsWith('https://')) {
      target = 'https://$target';
    }
    _isLoading = true;
    _currentUrl = target;
    _errorMessage = null;
    notifyListeners();
    _isLoading = false;
    notifyListeners();
  }

  /// Reloads current page.
  Future<void> reload() async {
    notifyListeners();
  }

  /// Toggles element picker mode.
  void toggleElementPicker() {
    _isPickerActive = !_isPickerActive;
    notifyListeners();
  }

  /// Sets picked DOM selector.
  void pickSelector(String selector, String tag, String text) {
    _selectedSelector = selector;
    onElementPicked?.call(selector, tag, text);
    notifyListeners();
  }
}
