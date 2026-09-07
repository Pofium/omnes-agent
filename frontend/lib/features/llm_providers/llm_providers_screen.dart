// Screen for managing LLM Providers, Models, and Custom OpenAPI Endpoints.

import 'package:flutter/material.dart';
import 'package:font_awesome_flutter/font_awesome_flutter.dart';
import 'package:get/get.dart';

import '../../utils/custom_color.dart';
import '../../utils/strings.dart';
import '../../widgets/appbar/appbar_widget.dart';
import 'llm_providers_controller.dart';

class LlmProvidersScreen extends StatelessWidget {
  const LlmProvidersScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(LlmProvidersController());
    final isDark = Theme.of(context).brightness == Brightness.dark;

    return Scaffold(
      appBar: AppBarWidget(
        context: context,
        onBackClick: () => Get.back(),
        appTitle: Strings.llmProviders.tr,
        onPressed: () => controller.loadProviders(),
      ),
      body: Obx(() {
        if (controller.isLoading.value) {
          return const Center(child: CircularProgressIndicator());
        }

        return RefreshIndicator(
          onRefresh: controller.loadProviders,
          child: SingleChildScrollView(
            physics: const AlwaysScrollableScrollPhysics(),
            padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 20.0),
            child: Center(
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 800),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Section 1: Active Model
                    _buildActiveModelCard(context, controller, isDark),
                    const SizedBox(height: 24),

                    // Section 2: Catalog Providers
                    _buildSectionHeader(
                      context,
                      title: Strings.backendCatalog.tr,
                      icon: FontAwesomeIcons.server,
                      isDark: isDark,
                    ),
                    const SizedBox(height: 12),
                    _buildCatalogList(context, controller, isDark),
                    const SizedBox(height: 28),

                    // Section 3: Add Custom Provider Form
                    _buildSectionHeader(
                      context,
                      title: Strings.customProvider.tr,
                      icon: FontAwesomeIcons.plug,
                      isDark: isDark,
                    ),
                    const SizedBox(height: 12),
                    _buildCustomProviderForm(context, controller, isDark),
                    const SizedBox(height: 28),

                    // Section 4: Configured Custom Providers
                    _buildSectionHeader(
                      context,
                      title: Strings.configuredProviders.tr,
                      icon: FontAwesomeIcons.sliders,
                      isDark: isDark,
                    ),
                    const SizedBox(height: 12),
                    _buildConfiguredProvidersList(context, controller, isDark),
                    const SizedBox(height: 40),
                  ],
                ),
              ),
            ),
          ),
        );
      }),
    );
  }

  Widget _buildSectionHeader(
    BuildContext context, {
    required String title,
    required IconData icon,
    required bool isDark,
  }) {
    return Row(
      children: [
        Icon(icon, size: 16, color: CustomColor.primaryColor),
        const SizedBox(width: 8),
        Text(
          title,
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.bold,
            letterSpacing: 0.3,
            color: isDark ? Colors.white : Colors.black87,
          ),
        ),
      ],
    );
  }

  Widget _buildActiveModelCard(
    BuildContext context,
    LlmProvidersController controller,
    bool isDark,
  ) {
    return Container(
      padding: const EdgeInsets.all(18),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          colors: isDark
              ? [const Color(0xFF1E293B), const Color(0xFF0F172A)]
              : [const Color(0xFFEEF2FF), const Color(0xFFE0E7FF)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(
          color: CustomColor.primaryColor.withOpacity(0.3),
        ),
        boxShadow: [
          BoxShadow(
            color: CustomColor.primaryColor.withOpacity(0.1),
            blurRadius: 16,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: CustomColor.primaryColor.withOpacity(0.15),
              shape: BoxShape.circle,
            ),
            child: Icon(
              FontAwesomeIcons.brain,
              color: CustomColor.primaryColor,
              size: 24,
            ),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  Strings.activeModel.tr,
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.w500,
                    color: isDark ? Colors.white60 : Colors.black54,
                  ),
                ),
                const SizedBox(height: 4),
                Obx(() => Text(
                      controller.activeModel.value,
                      style: TextStyle(
                        fontSize: 17,
                        fontWeight: FontWeight.bold,
                        color: isDark ? Colors.white : Colors.black87,
                      ),
                    )),
              ],
            ),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
            decoration: BoxDecoration(
              color: Colors.green.withOpacity(0.2),
              borderRadius: BorderRadius.circular(20),
              border: Border.all(color: Colors.green.withOpacity(0.4)),
            ),
            child: const Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(Icons.check_circle, color: Colors.greenAccent, size: 14),
                SizedBox(width: 4),
                Text(
                  'ONLINE',
                  style: TextStyle(
                    color: Colors.greenAccent,
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCatalogList(
    BuildContext context,
    LlmProvidersController controller,
    bool isDark,
  ) {
    return ListView.separated(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: controller.catalogProviders.length,
      separatorBuilder: (_, __) => const SizedBox(height: 10),
      itemBuilder: (context, index) {
        final item = controller.catalogProviders[index];
        final name = item['name'] ?? 'Provider';
        final isConfigured = item['is_configured'] ?? false;
        final models = (item['models'] as List?)?.whereType<String>().toList() ?? [];

        return Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: isDark ? const Color(0xFF1E293B) : Colors.white,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: isDark ? Colors.white12 : Colors.black12,
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Text(
                    name,
                    style: TextStyle(
                      fontSize: 15,
                      fontWeight: FontWeight.bold,
                      color: isDark ? Colors.white : Colors.black87,
                    ),
                  ),
                  const Spacer(),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                    decoration: BoxDecoration(
                      color: isConfigured
                          ? Colors.blue.withOpacity(0.15)
                          : Colors.grey.withOpacity(0.15),
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Text(
                      isConfigured ? 'Available' : 'API Key Required',
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                        color: isConfigured ? Colors.blueAccent : Colors.grey,
                      ),
                    ),
                  ),
                ],
              ),
              if (models.isNotEmpty) ...[
                const SizedBox(height: 10),
                Wrap(
                  spacing: 6,
                  runSpacing: 6,
                  children: models.map((m) {
                    return Obx(() {
                      final isSelected = controller.activeModel.value == m;
                      return ActionChip(
                        label: Text(m),
                        labelStyle: TextStyle(
                          fontSize: 11,
                          fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                          color: isSelected ? Colors.white : (isDark ? Colors.white70 : Colors.black87),
                        ),
                        backgroundColor: isSelected
                            ? CustomColor.primaryColor
                            : (isDark ? Colors.white.withOpacity(0.06) : Colors.black.withOpacity(0.04)),
                        side: BorderSide(
                          color: isSelected
                              ? CustomColor.primaryColor
                              : (isDark ? Colors.white12 : Colors.black12),
                        ),
                        onPressed: () => controller.setActiveModel(m),
                      );
                    });
                  }).toList(),
                ),
              ],
            ],
          ),
        );
      },
    );
  }

  Widget _buildCustomProviderForm(
    BuildContext context,
    LlmProvidersController controller,
    bool isDark,
  ) {
    final fieldBg = isDark ? const Color(0xFF1E293B) : Colors.white;
    final borderColor = isDark ? Colors.white12 : Colors.black12;

    return Container(
      padding: const EdgeInsets.all(18),
      decoration: BoxDecoration(
        color: isDark ? const Color(0xFF0F172A) : Colors.grey.shade50,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: borderColor),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Wire Format dropdown
          Text(
            Strings.apiFormat.tr,
            style: TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w600,
              color: isDark ? Colors.white70 : Colors.black87,
            ),
          ),
          const SizedBox(height: 6),
          Obx(() => Container(
                padding: const EdgeInsets.symmetric(horizontal: 14),
                decoration: BoxDecoration(
                  color: fieldBg,
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(color: borderColor),
                ),
                child: DropdownButtonHideUnderline(
                  child: DropdownButton<String>(
                    isExpanded: true,
                    value: controller.selectedFormat.value,
                    dropdownColor: fieldBg,
                    items: controller.wireFormats
                        .map((f) => DropdownMenuItem(
                              value: f,
                              child: Text(f),
                            ))
                        .toList(),
                    onChanged: (val) {
                      if (val != null) controller.selectedFormat.value = val;
                    },
                  ),
                ),
              )),
          const SizedBox(height: 14),

          // Provider Name
          _buildTextField(
            controller: controller.nameController,
            label: Strings.providerName.tr,
            hint: 'e.g. Local vLLM / OpenAI Proxy',
            isDark: isDark,
          ),
          const SizedBox(height: 14),

          // Base URL
          _buildTextField(
            controller: controller.baseUrlController,
            label: Strings.baseUrl.tr,
            hint: 'https://api.openai.com/v1',
            isDark: isDark,
          ),
          const SizedBox(height: 14),

          // API Key
          _buildTextField(
            controller: controller.apiKeyController,
            label: Strings.apiKey.tr,
            hint: 'sk-...',
            isDark: isDark,
            obscureText: true,
          ),
          const SizedBox(height: 14),

          // Model Name
          _buildTextField(
            controller: controller.modelController,
            label: Strings.modelName.tr,
            hint: 'e.g. meta-llama/llama-3.1-70b-instruct',
            isDark: isDark,
          ),
          const SizedBox(height: 20),

          // Action buttons: Test Connection & Save Provider
          Row(
            children: [
              Expanded(
                child: Obx(() => OutlinedButton.icon(
                      style: OutlinedButton.styleFrom(
                        padding: const EdgeInsets.symmetric(vertical: 14),
                        shape: RoundedRectangleBorder(
                          borderRadius: BorderRadius.circular(10),
                        ),
                        side: BorderSide(color: CustomColor.primaryColor),
                      ),
                      onPressed: controller.isTesting.value
                          ? null
                          : controller.testConnection,
                      icon: controller.isTesting.value
                          ? const SizedBox(
                              width: 16,
                              height: 16,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : const Icon(Icons.speed, size: 18),
                      label: Text(Strings.testConnection.tr),
                    )),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: ElevatedButton.icon(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: CustomColor.primaryColor,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                  ),
                  onPressed: controller.saveCustomProvider,
                  icon: const Icon(Icons.add, size: 18),
                  label: Text(Strings.addCustomProvider.tr),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildTextField({
    required TextEditingController controller,
    required String label,
    required String hint,
    required bool isDark,
    bool obscureText = false,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: TextStyle(
            fontSize: 13,
            fontWeight: FontWeight.w600,
            color: isDark ? Colors.white70 : Colors.black87,
          ),
        ),
        const SizedBox(height: 6),
        TextFormField(
          controller: controller,
          obscureText: obscureText,
          style: TextStyle(
            color: isDark ? Colors.white : Colors.black87,
            fontSize: 14,
          ),
          decoration: InputDecoration(
            hintText: hint,
            hintStyle: TextStyle(
              color: isDark ? Colors.white30 : Colors.black26,
              fontSize: 13,
            ),
            filled: true,
            fillColor: isDark ? const Color(0xFF1E293B) : Colors.white,
            contentPadding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(10),
              borderSide: BorderSide(
                color: isDark ? Colors.white12 : Colors.black12,
              ),
            ),
            enabledBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(10),
              borderSide: BorderSide(
                color: isDark ? Colors.white12 : Colors.black12,
              ),
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(10),
              borderSide: BorderSide(
                color: CustomColor.primaryColor,
                width: 1.5,
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildConfiguredProvidersList(
    BuildContext context,
    LlmProvidersController controller,
    bool isDark,
  ) {
    if (controller.customProviders.isEmpty) {
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.all(20),
        decoration: BoxDecoration(
          color: isDark ? const Color(0xFF1E293B).withOpacity(0.5) : Colors.grey.shade100,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Center(
          child: Text(
            Strings.noConfiguredProviders.tr,
            style: TextStyle(
              color: isDark ? Colors.white54 : Colors.black45,
              fontSize: 13,
            ),
          ),
        ),
      );
    }

    return ListView.separated(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: controller.customProviders.length,
      separatorBuilder: (_, __) => const SizedBox(height: 10),
      itemBuilder: (context, index) {
        final p = controller.customProviders[index];
        final id = p['id'] ?? '';
        final name = p['name'] ?? '';
        final model = p['model'] ?? '';
        final format = p['format'] ?? 'OpenAI compatible';

        return Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: isDark ? const Color(0xFF1E293B) : Colors.white,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: isDark ? Colors.white12 : Colors.black12,
            ),
          ),
          child: Row(
            children: [
              Container(
                padding: const EdgeInsets.all(10),
                decoration: BoxDecoration(
                  color: CustomColor.primaryColor.withOpacity(0.15),
                  shape: BoxShape.circle,
                ),
                child: Icon(
                  FontAwesomeIcons.microchip,
                  size: 16,
                  color: CustomColor.primaryColor,
                ),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      name,
                      style: TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.bold,
                        color: isDark ? Colors.white : Colors.black87,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      '$model ($format)',
                      style: TextStyle(
                        fontSize: 12,
                        color: isDark ? Colors.white60 : Colors.black54,
                      ),
                    ),
                  ],
                ),
              ),
              Obx(() {
                final isSelected = controller.activeModel.value == model;
                return IconButton(
                  tooltip: Strings.setActiveModel.tr,
                  icon: Icon(
                    isSelected ? Icons.check_circle : Icons.radio_button_unchecked,
                    color: isSelected ? Colors.greenAccent : Colors.grey,
                  ),
                  onPressed: () => controller.setActiveModel(model),
                );
              }),
              IconButton(
                tooltip: 'Delete',
                icon: const Icon(Icons.delete_outline, color: Colors.redAccent, size: 20),
                onPressed: () => controller.deleteCustomProvider(id),
              ),
            ],
          ),
        );
      },
    );
  }
}
