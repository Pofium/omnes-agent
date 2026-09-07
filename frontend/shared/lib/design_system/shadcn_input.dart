// ShadcnInput — Minimalist text field with active border illumination.
import 'package:flutter/material.dart';
import 'shadcn_colors.dart';

class ShadcnInput extends StatefulWidget {
  final TextEditingController? controller;
  final String? hintText;
  final String? labelText;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;
  final Widget? prefixIcon;
  final Widget? suffixIcon;
  final bool obscureText;
  final TextInputType? keyboardType;
  final int maxLines;
  final bool autofocus;
  final bool readOnly;

  const ShadcnInput({
    super.key,
    this.controller,
    this.hintText,
    this.labelText,
    this.onChanged,
    this.onSubmitted,
    this.prefixIcon,
    this.suffixIcon,
    this.obscureText = false,
    this.keyboardType,
    this.maxLines = 1,
    this.autofocus = false,
    this.readOnly = false,
  });

  @override
  State<ShadcnInput> createState() => _ShadcnInputState();
}

class _ShadcnInputState extends State<ShadcnInput> {
  final FocusNode _focusNode = FocusNode();
  bool _isFocused = false;

  @override
  void initState() {
    super.initState();
    _focusNode.addListener(() {
      setState(() {
        _isFocused = _focusNode.hasFocus;
      });
    });
  }

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        if (widget.labelText != null) ...[
          Text(
            widget.labelText!,
            style: const TextStyle(
              color: ShadcnColors.foregroundMuted,
              fontSize: 12,
              fontWeight: FontWeight.w500,
            ),
          ),
          const SizedBox(height: 6),
        ],
        AnimatedContainer(
          duration: const Duration(milliseconds: 150),
          decoration: BoxDecoration(
            color: ShadcnColors.card,
            borderRadius: BorderRadius.circular(10),
            border: Border.all(
              color: _isFocused ? ShadcnColors.primary : ShadcnColors.border,
              width: _isFocused ? 1.5 : 1.0,
            ),
            boxShadow: _isFocused
                ? [
                    BoxShadow(
                      color: ShadcnColors.primaryGlow,
                      blurRadius: 8,
                      offset: const Offset(0, 1),
                    ),
                  ]
                : null,
          ),
          child: TextField(
            controller: widget.controller,
            focusNode: _focusNode,
            autofocus: widget.autofocus,
            readOnly: widget.readOnly,
            obscureText: widget.obscureText,
            keyboardType: widget.keyboardType,
            maxLines: widget.maxLines,
            onChanged: widget.onChanged,
            onSubmitted: widget.onSubmitted,
            style: const TextStyle(
              color: ShadcnColors.foreground,
              fontSize: 14,
            ),
            cursorColor: ShadcnColors.primary,
            decoration: InputDecoration(
              hintText: widget.hintText,
              hintStyle: const TextStyle(
                color: ShadcnColors.foregroundSubtle,
                fontSize: 13,
              ),
              prefixIcon: widget.prefixIcon,
              suffixIcon: widget.suffixIcon,
              isDense: true,
              contentPadding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
              border: InputBorder.none,
            ),
          ),
        ),
      ],
    );
  }
}
