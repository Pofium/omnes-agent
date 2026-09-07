import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:webview_windows/webview_windows.dart';

/// Callback invoked when a DOM element is picked in the browser.
typedef OnElementPickedCallback = void Function(String selector, String tag, String text);

/// Controller managing Microsoft Edge WebView2 instance and DOM Element Picker.
class DesktopWebviewController extends ChangeNotifier {
  final WebviewController _webview = WebviewController();
  
  bool _isInitialized = false;
  bool _isLoading = false;
  bool _isPickerActive = false;
  String _currentUrl = 'http://localhost:3000';
  String? _hoveredSelector;
  String? _selectedSelector;
  String? _errorMessage;

  StreamSubscription? _urlSub;
  StreamSubscription? _loadingSub;
  StreamSubscription? _messageSub;
  StreamSubscription? _errorSub;

  OnElementPickedCallback? onElementPicked;

  WebviewController get rawController => _webview;
  bool get isInitialized => _isInitialized;
  bool get isLoading => _isLoading;
  bool get isPickerActive => _isPickerActive;
  String get currentUrl => _currentUrl;
  String? get hoveredSelector => _hoveredSelector;
  String? get selectedSelector => _selectedSelector;
  String? get errorMessage => _errorMessage;

  /// Initializes the WebView2 environment and instance.
  Future<bool> initialize({String initialUrl = 'http://localhost:3000'}) async {
    if (_isInitialized) return true;
    _currentUrl = initialUrl;

    try {
      await _webview.initialize();
      _isInitialized = true;
      _errorMessage = null;

      _urlSub = _webview.url.listen((url) {
        _currentUrl = url;
        notifyListeners();
      });

      _loadingSub = _webview.loadingState.listen((state) {
        _isLoading = (state == LoadingState.loading);
        notifyListeners();
      });

      _errorSub = _webview.onLoadError.listen((error) {
        _errorMessage = 'Load error: $error';
        notifyListeners();
      });

      // Listen for messages from injected Element Picker JS
      _messageSub = _webview.webMessage.listen((dynamic raw) {
        _handleWebMessage(raw);
      });

      await loadUrl(_currentUrl);
      notifyListeners();
      return true;
    } catch (e) {
      _errorMessage = 'WebView2 init failed: $e';
      _isInitialized = false;
      notifyListeners();
      return false;
    }
  }

  /// Loads a URL into the browser.
  Future<void> loadUrl(String url) async {
    if (!_isInitialized) return;
    try {
      var target = url.trim();
      if (!target.startsWith('http://') && !target.startsWith('https://')) {
        target = 'https://$target';
      }
      _currentUrl = target;
      await _webview.loadUrl(target);
      _errorMessage = null;
    } catch (e) {
      _errorMessage = 'Failed to navigate: $e';
    }
    notifyListeners();
  }

  /// Reloads current page.
  Future<void> reload() async {
    if (!_isInitialized) return;
    try {
      await _webview.reload();
    } catch (_) {}
  }

  /// Toggles DOM Element Picker on / off.
  Future<void> toggleElementPicker() async {
    if (!_isInitialized) return;
    _isPickerActive = !_isPickerActive;
    if (_isPickerActive) {
      await _injectPickerScript();
    } else {
      await _removePickerScript();
      _hoveredSelector = null;
    }
    notifyListeners();
  }

  /// Handles incoming messages from WebView2 postMessage.
  void _handleWebMessage(dynamic raw) {
    try {
      Map<String, dynamic> data;
      if (raw is String) {
        data = jsonDecode(raw) as Map<String, dynamic>;
      } else if (raw is Map) {
        data = Map<String, dynamic>.from(raw);
      } else {
        return;
      }

      final type = data['type']?.toString();
      if (type == 'hover') {
        _hoveredSelector = data['selector']?.toString();
        notifyListeners();
      } else if (type == 'element_selected') {
        final sel = data['selector']?.toString() ?? '';
        final tag = data['tag']?.toString() ?? '';
        final text = data['text']?.toString() ?? '';
        _selectedSelector = sel;
        _hoveredSelector = sel;
        _isPickerActive = false; // deactivate picker upon selection
        _removePickerScript();
        notifyListeners();

        if (onElementPicked != null && sel.isNotEmpty) {
          onElementPicked!(sel, tag, text);
        }
      }
    } catch (_) {}
  }

  /// Injects hover outline and click interceptor into web page DOM.
  Future<void> _injectPickerScript() async {
    const js = '''
(function() {
  if (window.__omnesPickerActive) return;
  window.__omnesPickerActive = true;

  var overlay = document.getElementById('__omnes_picker_overlay');
  if (!overlay) {
    overlay = document.createElement('div');
    overlay.id = '__omnes_picker_overlay';
    overlay.style.position = 'fixed';
    overlay.style.pointerEvents = 'none';
    overlay.style.border = '2px solid #00D2FF';
    overlay.style.backgroundColor = 'rgba(0, 210, 255, 0.15)';
    overlay.style.zIndex = '2147483647';
    overlay.style.transition = 'all 0.05s ease';
    document.body.appendChild(overlay);
  }
  overlay.style.display = 'block';

  function getSelector(el) {
    if (!el || el === document.body || el === document.documentElement) return '';
    var tag = el.tagName.toLowerCase();
    if (el.id) return tag + '#' + el.id;
    if (el.className && typeof el.className === 'string' && el.className.trim()) {
      var classes = el.className.trim().split(/\\s+/).slice(0, 2).join('.');
      return tag + '.' + classes;
    }
    return tag;
  }

  window.__omnesMouseMove = function(e) {
    if (!window.__omnesPickerActive) return;
    var el = document.elementFromPoint(e.clientX, e.clientY);
    if (!el || el.id === '__omnes_picker_overlay') return;
    var rect = el.getBoundingClientRect();
    overlay.style.left = rect.left + 'px';
    overlay.style.top = rect.top + 'px';
    overlay.style.width = rect.width + 'px';
    overlay.style.height = rect.height + 'px';

    var sel = getSelector(el);
    if (window.chrome && window.chrome.webview) {
      window.chrome.webview.postMessage(JSON.stringify({
        type: 'hover',
        selector: sel
      }));
    }
  };

  window.__omnesMouseClick = function(e) {
    if (!window.__omnesPickerActive) return;
    e.preventDefault();
    e.stopPropagation();
    var el = document.elementFromPoint(e.clientX, e.clientY);
    if (!el) return;
    var sel = getSelector(el);
    var tag = el.tagName.toLowerCase();
    var text = (el.innerText || el.textContent || '').substring(0, 40).trim();

    if (window.chrome && window.chrome.webview) {
      window.chrome.webview.postMessage(JSON.stringify({
        type: 'element_selected',
        selector: sel,
        tag: tag,
        text: text
      }));
    }
  };

  document.addEventListener('mousemove', window.__omnesMouseMove, true);
  document.addEventListener('click', window.__omnesMouseClick, true);
})();
''';
    try {
      await _webview.executeScript(js);
    } catch (_) {}
  }

  /// Removes injected picker script and overlay.
  Future<void> _removePickerScript() async {
    const js = '''
(function() {
  window.__omnesPickerActive = false;
  var overlay = document.getElementById('__omnes_picker_overlay');
  if (overlay) overlay.style.display = 'none';
  if (window.__omnesMouseMove) {
    document.removeEventListener('mousemove', window.__omnesMouseMove, true);
  }
  if (window.__omnesMouseClick) {
    document.removeEventListener('click', window.__omnesMouseClick, true);
  }
})();
''';
    try {
      await _webview.executeScript(js);
    } catch (_) {}
  }

  @override
  void dispose() {
    _urlSub?.cancel();
    _loadingSub?.cancel();
    _messageSub?.cancel();
    _errorSub?.cancel();
    _webview.dispose();
    super.dispose();
  }
}
