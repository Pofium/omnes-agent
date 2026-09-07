// Suggested prompts repository for Supabase table 'suggested_prompts'.

import 'package:flutter/foundation.dart';
import 'package:supabase_flutter/supabase_flutter.dart';
import 'supabase_config.dart';

class SuggestedPrompt {
  final String id;
  final String category;
  final String title;
  final String prompt;
  final int sort;

  const SuggestedPrompt({
    required this.id,
    required this.category,
    required this.title,
    required this.prompt,
    this.sort = 0,
  });

  factory SuggestedPrompt.fromJson(Map<String, dynamic> json) {
    return SuggestedPrompt(
      id: json['id']?.toString() ?? '',
      category: json['category']?.toString() ?? 'Общие',
      title: json['title']?.toString() ?? '',
      prompt: json['prompt']?.toString() ?? '',
      sort: (json['sort'] as num?)?.toInt() ?? 0,
    );
  }
}

class SuggestedPromptsRepository {
  final SupabaseClient? _client;

  SuggestedPromptsRepository({SupabaseClient? client})
      : _client = client ?? SupabaseConfig.client;

  static const List<SuggestedPrompt> defaultPrompts = [
    SuggestedPrompt(
      id: 'default-1',
      category: 'Проекты',
      title: 'Создай план проекта',
      prompt: 'Помоги создать подробный пошаговый план нового проекта',
      sort: 1,
    ),
    SuggestedPrompt(
      id: 'default-2',
      category: 'Автоматизация',
      title: 'Напиши скрипт автоматизации',
      prompt: 'Напиши Bash/Python скрипт для регулярного бэкапа рабочей директории',
      sort: 2,
    ),
    SuggestedPrompt(
      id: 'default-3',
      category: 'Файлы',
      title: 'Проанализируй воркспейс',
      prompt: 'Посмотри файлы в текущей директории и расскажи об их структуре',
      sort: 3,
    ),
    SuggestedPrompt(
      id: 'default-4',
      category: 'Сводка',
      title: 'Сводка новостей',
      prompt: 'Подготовь краткую сводку последних обновлений и задач',
      sort: 4,
    ),
  ];

  /// Loads suggested prompts from Supabase or returns defaults if offline.
  Future<List<SuggestedPrompt>> getPrompts() async {
    final client = _client ?? SupabaseConfig.client;
    if (client == null) {
      return defaultPrompts;
    }

    try {
      final response = await client
          .from('suggested_prompts')
          .select()
          .order('sort', ascending: true);

      if (response.isNotEmpty) {
        return response
            .map((item) => SuggestedPrompt.fromJson(Map<String, dynamic>.from(item)))
            .toList();
      }
      return defaultPrompts;
    } catch (e) {
      debugPrint('SuggestedPromptsRepository.getPrompts fallback: $e');
      return defaultPrompts;
    }
  }
}
