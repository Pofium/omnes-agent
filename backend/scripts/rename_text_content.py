#!/usr/bin/env python3
"""
Скрипт для замены всех упоминаний 'omnesagent' на 'omnesagent' во всех файлах проекта.
Поддерживает различные регистры и варианты написания:
- omnesagent -> omnesagent (lowercase)
- Omnesagent -> Omnesagent (capitalized)
- OMNESAGENT -> OMNESAGENT (uppercase)
- OmnesAgent -> OmnesAgent (PascalCase)
- omnes_agent -> omnes_agent (snake_case)
- omnes-agent -> omnes-agent (kebab-case)
- Omnes Agent -> Omnes Agent (с пробелом)
"""

import os
import re
import sys
from pathlib import Path
from typing import Dict, Tuple


def preserve_case_replace(text: str, old: str, new: str) -> str:
    """
    Заменяет old на new с сохранением регистра исходного текста.
    Поддерживает: lowercase, UPPERCASE, Capitalized, PascalCase.
    """
    # Точное совпадение регистра
    if text == old.lower():
        return new.lower()
    if text == old.upper():
        return new.upper()
    if text == old.capitalize():
        return new.capitalize()
    
    # PascalCase: OmnesAgent -> OmnesAgent
    old_pascal = ''.join(w.capitalize() for w in [old[:4], old[4:]])  # OmnesAgent
    new_pascal = ''.join(w.capitalize() for w in [new[:5], new[5:]])  # OmnesAgent
    if text == old_pascal:
        return new_pascal
    
    # camelCase: omnesAgent -> omnesAgent
    old_camel = old[:4].lower() + old[4:].capitalize()  # omnesAgent
    new_camel = new[:5].lower() + new[5:].capitalize()  # omnesAgent
    if text == old_camel:
        return new_camel
    
    # По умолчанию: lowercase замена
    return new.lower()


def generate_replacements() -> Dict[re.Pattern, str]:
    """
    Генерирует набор regex-паттернов для всех вариантов написания.
    """
    # Базовые варианты с различными разделителями
    patterns: Dict[re.Pattern, str] = {}
    
    old_base = 'omnesagent'
    new_base = 'omnesagent'
    
    # 1. Простые варианты без разделителей (умная замена с сохранением регистра)
    patterns[re.compile(r'omnesagent', re.IGNORECASE)] = '__SMART_CASE_1__'
    
    # 2. Snake case: omnes_agent -> omnes_agent
    patterns[re.compile(r'omnes_agent', re.IGNORECASE)] = '__SMART_SNAKE__'
    
    # 3. Kebab case: omnes-agent -> omnes-agent
    patterns[re.compile(r'omnes-agent', re.IGNORECASE)] = '__SMART_KEBAB__'
    
    # 4. Dot case: omnes.agent -> omnes.agent
    patterns[re.compile(r'zero\.claw', re.IGNORECASE)] = '__SMART_DOT__'
    
    # 5. Space separated: omnes agent -> omnes agent
    patterns[re.compile(r'zero\s+claw', re.IGNORECASE)] = '__SMART_SPACE__'
    
    return patterns


def apply_smart_replacements(content: str) -> Tuple[str, int]:
    """
    Применяет все замены с сохранением регистра.
    Возвращает (обработанный_текст, количество_замен).
    """
    total_replacements = 0
    
    # 1. Замена вариантов с разделителями (snake, kebab, dot, space)
    # Snake case: omnes_agent -> omnes_agent
    def snake_replace(match: re.Match) -> str:
        nonlocal total_replacements
        total_replacements += 1
        text = match.group(0)
        if text.isupper():
            return 'OMNES_AGENT'
        elif text[0].isupper():
            return 'Omnes_Agent'
        else:
            return 'omnes_agent'
    
    content = re.sub(r'omnes_agent', snake_replace, content, flags=re.IGNORECASE)
    
    # Kebab case: omnes-agent -> omnes-agent
    def kebab_replace(match: re.Match) -> str:
        nonlocal total_replacements
        total_replacements += 1
        text = match.group(0)
        if text.isupper():
            return 'OMNES-AGENT'
        elif text[0].isupper():
            return 'Omnes-Agent'
        else:
            return 'omnes-agent'
    
    content = re.sub(r'omnes-agent', kebab_replace, content, flags=re.IGNORECASE)
    
    # Dot case: omnes.agent -> omnes.agent
    def dot_replace(match: re.Match) -> str:
        nonlocal total_replacements
        total_replacements += 1
        text = match.group(0)
        if text.isupper():
            return 'OMNES.AGENT'
        elif text[0].isupper():
            return 'Omnes.Agent'
        else:
            return 'omnes.agent'
    
    content = re.sub(r'zero\.claw', dot_replace, content, flags=re.IGNORECASE)
    
    # Space separated: omnes agent -> omnes agent
    def space_replace(match: re.Match) -> str:
        nonlocal total_replacements
        total_replacements += 1
        text = match.group(0)
        ws = re.search(r'\s+', text).group(0)
        if text.isupper():
            return f'OMNES{ws}AGENT'
        elif text[0].isupper():
            return f'Omnes{ws}Agent'
        else:
            return f'omnes{ws}agent'
    
    content = re.sub(r'zero\s+claw', space_replace, content, flags=re.IGNORECASE)
    
    # 2. Замена слитного варианта omnesagent (сохраняем регистр)
    def smart_replace(match: re.Match) -> str:
        nonlocal total_replacements
        total_replacements += 1
        text = match.group(0)
        
        # Различные варианты регистра
        if text == 'omnesagent':
            return 'omnesagent'
        elif text == 'OMNESAGENT':
            return 'OMNESAGENT'
        elif text == 'Omnesagent':
            return 'Omnesagent'
        elif text == 'OmnesAgent':
            return 'OmnesAgent'
        elif text == 'omnesAgent':
            return 'omnesAgent'
        elif text == 'OMNESAGENT':
            return 'OMNESAGENT'
        elif text.islower():
            return 'omnesagent'
        elif text.isupper():
            return 'OMNESAGENT'
        elif text[0].isupper() and text[1:].islower():
            return 'Omnesagent'
        else:
            # Для смешанных регистров: PascalCase вариант
            return 'OmnesAgent'
    
    content = re.sub(r'omnesagent', smart_replace, content, flags=re.IGNORECASE)
    
    return content, total_replacements


# Список расширений файлов для обработки (текстовые файлы)
ALLOWED_EXTENSIONS = {
    '.rs', '.toml', '.md', '.txt', '.json', '.yaml', '.yml',
    '.py', '.sh', '.bat', '.ps1', '.html', '.css', '.js', '.ts',
    '.tsx', '.jsx', '.ftl', '.ini', '.cfg', '.conf', '.env',
    '.example', '.template', '.gitignore', '.gitattributes',
    '.dockerignore', '.Dockerfile', '.containerfile',
    '.proto', '.sql', '.nix', '.hcl', '.slint', '.ino',
    '.rules', '.plist', '.lock', '.h', '.hpp', '.c', '.cpp',
    'Dockerfile', 'Containerfile', 'Makefile', 'Justfile',
    '.SRCINFO', 'PKGBUILD'
}

# Директории, которые нужно пропустить
EXCLUDED_DIRS = {
    'target', '.git', 'node_modules', '.venv', 'venv',
    '__pycache__', 'dist', 'build', '.cargo', '.fingerprint',
    'deps', '.gitlab', '.idea'
}

# Исключаемые расширения (бинарные файлы)
EXCLUDED_EXTENSIONS = {
    '.png', '.jpg', '.jpeg', '.gif', '.ico', '.svg', '.pdf',
    '.mp3', '.wav', '.ogg', '.mp4', '.avi', '.mov',
    '.exe', '.dll', '.so', '.dylib', '.lib', '.a',
    '.class', '.jar', '.war', '.ear',
    '.zip', '.tar', '.gz', '.bz2', '.7z', '.rar',
    '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx',
    '.uf2', '.bin', '.hex', '.elf', '.o', '.obj',
    '.pdb', '.exp', '.dll.lib', '.wasm', '.ttf', '.woff',
    '.woff2', '.eot', '.otf',
}


def should_process_file(file_path: Path, root: Path) -> bool:
    """
    Определяет, нужно ли обрабатывать файл.
    """
    # Проверяем, не в исключённой ли директории
    try:
        rel_parts = file_path.relative_to(root).parts
    except ValueError:
        return False
    
    for part in rel_parts[:-1]:  # Без имени файла
        if part in EXCLUDED_DIRS:
            return False
    
    # Проверяем расширение или имя файла
    name = file_path.name
    suffix = file_path.suffix.lower()
    
    # Проверяем исключённые расширения
    if suffix in EXCLUDED_EXTENSIONS:
        return False
    
    # Проверяем разрешённые расширения и имена файлов
    if suffix in ALLOWED_EXTENSIONS:
        return True
    if name in ALLOWED_EXTENSIONS:  # Для файлов без расширения типа Dockerfile
        return True
    
    # Для остальных файлов проверяем по содержимому (текстовые ли они)
    try:
        with open(file_path, 'r', encoding='utf-8', errors='strict') as f:
            f.read(1024)  # Читаем небольшой кусок
        return True
    except (UnicodeDecodeError, IOError):
        return False


def process_directory(root_dir: str) -> Dict[str, int]:
    """
    Обрабатывает все файлы в директории, заменяя omnesagent -> omnesagent.
    """
    root_path = Path(root_dir).resolve()
    print(f"Начинаю обработку директории: {root_path}")
    
    stats = {
        'files_processed': 0,
        'files_modified': 0,
        'total_replacements': 0,
        'files_skipped': 0,
        'errors': 0
    }
    
    # Проходим по всем файлам
    for file_path in root_path.rglob('*'):
        if not file_path.is_file():
            continue
        
        if not should_process_file(file_path, root_path):
            stats['files_skipped'] += 1
            continue
        
        try:
            # Читаем содержимое файла
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Проверяем, есть ли что заменять (быстрая проверка перед regex)
            if not re.search(r'omnesagent|omnes.agent|omnes_agent|omnes-agent|zero\s+claw',
                           content, re.IGNORECASE):
                stats['files_processed'] += 1
                continue
            
            # Применяем замены
            new_content, replacements = apply_smart_replacements(content)
            
            stats['files_processed'] += 1
            
            if replacements > 0:
                # Записываем обратно
                with open(file_path, 'w', encoding='utf-8', newline='') as f:
                    f.write(new_content)
                
                stats['files_modified'] += 1
                stats['total_replacements'] += replacements
                
                rel_path = file_path.relative_to(root_path)
                print(f"  ✓ {rel_path} ({replacements} замен)")
        
        except Exception as e:
            stats['errors'] += 1
            rel_path = file_path.relative_to(root_path) if root_path in file_path.parents else file_path
            print(f"  ✗ Ошибка при обработке {rel_path}: {e}", file=sys.stderr)
    
    return stats


def main():
    """
    Главная функция скрипта.
    """
    # Определяем корневую директорию проекта
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent  # scripts/ -> корень проекта
    
    # Можно передать путь через аргумент командной строки
    if len(sys.argv) > 1:
        project_root = Path(sys.argv[1]).resolve()
    
    print("=" * 70)
    print("СКРИПТ ЗАМЕНЫ omnesagent -> omnesagent В СОДЕРЖИМОМ ФАЙЛОВ")
    print("=" * 70)
    
    stats = process_directory(str(project_root))
    
    print("\n" + "=" * 70)
    print("РЕЗУЛЬТАТЫ ОБРАБОТКИ")
    print("=" * 70)
    print(f"  Всего обработано файлов:      {stats['files_processed']}")
    print(f"  Изменено файлов:              {stats['files_modified']}")
    print(f"  Всего произведено замен:      {stats['total_replacements']}")
    print(f"  Пропущено (бинарные и т.д.):  {stats['files_skipped']}")
    print(f"  Ошибок при обработке:         {stats['errors']}")
    print("=" * 70)
    
    if stats['errors'] > 0:
        sys.exit(1)
    else:
        print("\n✓ Готово!")


if __name__ == '__main__':
    main()
