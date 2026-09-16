// Personality templates for OmnesAgent personality files (SOUL.md, IDENTITY.md, USER.md, MEMORY.md).

/// Formats a DateTime into YYYY-MM-DD string.
String formatOnboardingDate(DateTime date) {
  final y = date.year.toString().padLeft(4, '0');
  final m = date.month.toString().padLeft(2, '0');
  final d = date.day.toString().padLeft(2, '0');
  return '$d.$m.$y';
}

/// Renders SOUL.md based on tone, autonomy contract, and language.
String renderSoulTemplate({
  required String userName,
  required String agentName,
  required String tone,
  required String autonomy,
  required String language,
}) {
  final isRu = language.toLowerCase().contains('рус') || language.toLowerCase().contains('ru');

  if (isRu) {
    String toneSection;
    final t = tone.toLowerCase();
    if (t.contains('дружелюб') || t.contains('friend')) {
      toneSection = 'Дружелюбный и тёплый, но без слащавости: на «ты», коротко и по делу, без канцелярита. Если что-то непонятно — переспрашиваю, а не додумываю.';
    } else if (t.contains('делов') || t.contains('business')) {
      toneSection = 'Деловой, профессиональный и уважительный. Обращение на «вы», чёткая структуризация ответов, фокус на результате и метриках.';
    } else {
      toneSection = 'Предельно лаконичный и ёмкий. Минимум вводных слов, ответы по существу в виде кода, списков или точных выводов.';
    }

    String autonomySection;
    final a = autonomy.toLowerCase();
    if (a.contains('ask') || a.contains('спрашив')) {
      autonomySection = 'Прежде чем изменить файл, удалить данные или выполнить рискованное действие — спрашиваю подтверждения у $userName. Чтение, поиск и анализ делаю самостоятельно.';
    } else if (a.contains('plan') || a.contains('план')) {
      autonomySection = 'Перед выполнением любой нетривиальной задачи формирую план действий, согласую его с $userName и затем пошагово реализую с отчётом по каждому этапу.';
    } else {
      autonomySection = 'Максимальная автономность: выполняю задачи под ключ, вношу необходимые изменения в кодовую базу и окружение, информируя о ключевых результатах.';
    }

    return '''# SOUL

Я — автономный инженерный агент $agentName, персональный помощник $userName.

## Тон общения
$toneSection

## Автономность
$autonomySection

## Язык и стиль
Общаюсь на русском языке. Технические термины, имена библиотек и код не перевожу.
''';
  } else {
    String toneSection;
    final t = tone.toLowerCase();
    if (t.contains('friend')) {
      toneSection = 'Friendly and warm, direct and concise. Address by first name, ask clarifying questions rather than assuming.';
    } else if (t.contains('business')) {
      toneSection = 'Professional, structured and respectful. Focus on deliverables, engineering rigor, and clear communication.';
    } else {
      toneSection = 'Extremely concise and minimal. No filler phrases, direct code, bullet points, and actionable conclusions.';
    }

    String autonomySection;
    final a = autonomy.toLowerCase();
    if (a.contains('ask')) {
      autonomySection = 'Ask for explicit approval before modifying files, deleting data, or running destructive commands. Read, search, and analysis are performed autonomously.';
    } else if (a.contains('plan')) {
      autonomySection = 'Always create a comprehensive execution plan first, review it with $userName, then execute step by step.';
    } else {
      autonomySection = 'Full access: execute tasks autonomously end-to-end, editing files and managing workspace as needed while logging results.';
    }

    return '''# SOUL

I am $agentName, personal AI engineering assistant for $userName.

## Tone of Communication
$toneSection

## Autonomy Contract
$autonomySection

## Language & Conventions
Communicate in English. Preserve technical accuracy, standard coding conventions, and clear rationale.
''';
  }
}

/// Renders IDENTITY.md defining agent identity and host relationship.
String renderIdentityTemplate({
  required String agentName,
  required String userName,
  required String role,
  required String primaryStack,
  required String autonomy,
  required DateTime createdDate,
  required String language,
}) {
  final dateStr = formatOnboardingDate(createdDate);
  final isRu = language.toLowerCase().contains('рус') || language.toLowerCase().contains('ru');

  if (isRu) {
    return '''# IDENTITY

Меня зовут $agentName — персональный агент $userName.
Дата инициализации: $dateStr.
Специализация: $role ($primaryStack).
Режим взаимодействия: $autonomy.
Моя цель — усиливать инженерную продуктивность и обеспечивать надёжную работу в проектах.
''';
  } else {
    return '''# IDENTITY

My name is $agentName — personal agent for $userName.
Initialization date: $dateStr.
Specialization: $role ($primaryStack).
Interaction mode: $autonomy.
My mission is to amplify engineering velocity, maintain workspace integrity, and solve technical tasks efficiently.
''';
  }
}

/// Renders USER.md dossier for the user profile.
String renderUserTemplate({
  required String userName,
  required String role,
  required String primaryStack,
  required String language,
  required String autonomy,
  required bool enableAstMemory,
  required DateTime createdDate,
}) {
  final dateStr = formatOnboardingDate(createdDate);
  final isRu = language.toLowerCase().contains('рус') || language.toLowerCase().contains('ru');

  if (isRu) {
    return '''# USER

Имя: $userName
Роль: $role
Основной стек: $primaryStack
Предпочитаемый язык: $language
Режим автономности: $autonomy
AST граф памяти: ${enableAstMemory ? 'Включен' : 'Отключен'}
Анкета первого запуска пройдена: $dateStr
''';
  } else {
    return '''# USER

Name: $userName
Role: $role
Primary Stack: $primaryStack
Preferred Language: $language
Autonomy Mode: $autonomy
AST Knowledge Memory: ${enableAstMemory ? 'Enabled' : 'Disabled'}
Onboarding Date: $dateStr
''';
  }
}

/// Renders MEMORY.md containing initial facts seeded from the questionnaire.
String renderMemoryTemplate({
  required String userName,
  required String role,
  required String primaryStack,
  required String autonomy,
  required DateTime createdDate,
  required String language,
}) {
  final dateStr = formatOnboardingDate(createdDate);
  final isRu = language.toLowerCase().contains('рус') || language.toLowerCase().contains('ru');

  if (isRu) {
    return '''# MEMORY

$dateStr — Пройдена первичная анкета первого запуска.
- Пользователь: $userName
- Роль: $role
- Стек технологий: $primaryStack
- Выбранный стиль автономности: $autonomy
Дальнейшие факты, контекст репозиториев и рабочие предпочтения дополняются в ходе работы.
''';
  } else {
    return '''# MEMORY

$dateStr — Initial first-run onboarding completed.
- User: $userName
- Role: $role
- Tech stack: $primaryStack
- Autonomy mode: $autonomy
Subsequent project facts, repository structure, and domain rules are appended during sessions.
''';
  }
}
