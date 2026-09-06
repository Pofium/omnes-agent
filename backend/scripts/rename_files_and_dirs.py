#!/usr/bin/env python3
"""
Скрипт для переименования всех файлов и директорий, содержащих 'omnesagent' в названии.
Поддерживает различные регистры и варианты написания:
- omnesagent -> omnesagent (lowercase)
- Omnesagent -> Omnesagent (capitalized)
- OMNESAGENT -> OMNESAGENT (uppercase)
- OmnesAgent -> OmnesAgent (PascalCase)
- omnes_agent -> omnes_agent (snake_case)
- omnes-agent -> omnes-agent (kebab-case)
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Tuple, Optional


def smart_rename_name(name: str) -> Optional[str]:
    """
    Переименовывает имя файла/директории, заменяя omnesagent -> omnesagent
    с сохранением регистра. Возвращает новое имя или None, если замен не требуется.
    """
    original_name = name
    changed = False
    
    # 1. Snake case: omnes_agent -> omnes_agent
    def snake_replace(m):
        text = m.group(0)
        if text.isupper():
            return 'OMNES_AGENT'
        elif text[0].isupper():
            return 'Omnes_Agent'
        else:
            return 'omnes_agent'
    
    new_name, count = re.subn(r'omnes_agent', snake_replace, name, flags=re.IGNORECASE)
    if count > 0:
        name = new_name
        changed = True
    
    # 2. Kebab case: omnes-agent -> omnes-agent
    def kebab_replace(m):
        text = m.group(0)
        if text.isupper():
            return 'OMNES-AGENT'
        elif text[0].isupper():
            return 'Omnes-Agent'
        else:
            return 'omnes-agent'
    
    new_name, count = re.subn(r'omnes-agent', kebab_replace, name, flags=re.IGNORECASE)
    if count > 0:
        name = new_name
        changed = True
    
    # 3. Dot case: omnes.agent -> omnes.agent
    def dot_replace(m):
        text = m.group(0)
        if text.isupper():
            return 'OMNES.AGENT'
        elif text[0].isupper():
            return 'Omnes.Agent'
        else:
            return 'omnes.agent'
    
    new_name, count = re.subn(r'zero\.claw', dot_replace, name, flags=re.IGNORECASE)
    if count > 0:
        name = new_name
        changed = True
    
    # 4. Слитный вариант: omnesagent -> omnesagent (сохраняем регистр)
    def smart_replace(m):
        text = m.group(0)
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
        elif text.islower():
            return 'omnesagent'
        elif text.isupper():
            return 'OMNESAGENT'
        elif text[0].isupper() and text[1:].islower():
            return 'Omnesagent'
        else:
            return 'OmnesAgent'
    
    new_name, count = re.subn(r'omnesagent', smart_replace, name, flags=re.IGNORECASE)
    if count > 0:
        name = new_name
        changed = True
    
    if changed and name != original_name:
        return name
    return None


# Директории, которые нужно пропустить (не переименовывать внутри них)
EXCLUDED_DIRS = {
    '.git', 'target', 'node_modules', '.venv', 'venv',
    '__pycache__', 'dist', 'build', '.cargo',
    '.gitlab', '.idea', '.vs', '.vscode', '.zed'
}


def collect_rename_ops(root_dir: str) -> List[Tuple[Path, Path]]:
    """
    Собирает все операции переименования в порядке от самых вложенных к корню.
    Возвращает список кортежей (старый_путь, новый_путь).
    """
    root_path = Path(root_dir).resolve()
    
    # Сначала собираем все файлы и директории, сортируем по глубине (от глубоких к поверхностным)
    all_items: List[Tuple[int, Path, bool]] = []  # (глубина, путь, является_директорией)
    
    for dirpath, dirnames, filenames in os.walk(root_path, topdown=True):
        # Удаляем исключённые директории из обхода
        dirnames[:] = [d for d in dirnames if d not in EXCLUDED_DIRS]
        
        dir_path = Path(dirpath)
        try:
            depth = len(dir_path.relative_to(root_path).parts)
        except ValueError:
            depth = 0
        
        # Добавляем директории
        for dirname in dirnames:
            full_path = dir_path / dirname
            all_items.append((depth, full_path, True))
        
        # Добавляем файлы
        for filename in filenames:
            full_path = dir_path / filename
            all_items.append((depth, full_path, False))
    
    # Сортируем в порядке убывания глубины (сначала самые вложенные)
    all_items.sort(key=lambda x: (-x[0], str(x[1])))
    
    # Теперь определяем, что нужно переименовать
    rename_ops: List[Tuple[Path, Path]] = []
    renamed_paths: set = set()
    
    for depth, item_path, is_dir in all_items:
        # Проверяем существование (могли уже переименовать родителя)
        if not item_path.exists():
            continue
        
        # Получаем новое имя для последнего компонента пути
        old_name = item_path.name
        new_name = smart_rename_name(old_name)
        
        if new_name is not None and new_name != old_name:
            new_path = item_path.parent / new_name
            
            # Проверяем, не существует ли уже такое имя
            if new_path.exists():
                print(f"  ⚠ Пропускаю: {item_path} -> {new_path} (уже существует)")
                continue
            
            rename_ops.append((item_path, new_path))
    
    return rename_ops


def execute_renames(rename_ops: List[Tuple[Path, Path]], dry_run: bool = False) -> Tuple[int, List[str]]:
    """
    Выполняет операции переименования.
    Возвращает (количество_успешных_переименований, список_ошибок).
    """
    success_count = 0
    errors: List[str] = []
    
    for old_path, new_path in rename_ops:
        try:
            if dry_run:
                print(f"  [DRY RUN] {old_path.name}  ->  {new_path.name}")
            else:
                old_path.rename(new_path)
                print(f"  ✓ Переименован: {old_path.name}  ->  {new_path.name}")
            success_count += 1
        except Exception as e:
            error_msg = f"{old_path} -> {new_path}: {e}"
            errors.append(error_msg)
            print(f"  ✗ Ошибка: {error_msg}", file=sys.stderr)
    
    return success_count, errors


def main():
    """
    Главная функция скрипта.
    """
    # Определяем корневую директорию проекта
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent  # scripts/ -> корень проекта
    
    dry_run = False
    
    # Парсим аргументы
    args = sys.argv[1:]
    if '--dry-run' in args or '-n' in args:
        dry_run = True
        args = [a for a in args if a not in ('--dry-run', '-n')]
    
    if len(args) > 0:
        project_root = Path(args[0]).resolve()
    
    print("=" * 70)
    print("СКРИПТ ПЕРЕИМЕНОВАНИЯ ФАЙЛОВ И ДИРЕКТОРИЙ omnesagent -> omnesagent")
    if dry_run:
        print("  [РЕЖИМ ПРОВЕРКИ - без реальных изменений]")
    print("=" * 70)
    print(f"Корневая директория: {project_root}")
    print()
    
    # Собираем операции переименования
    print("Анализ файлов и директорий...")
    rename_ops = collect_rename_ops(str(project_root))
    
    if not rename_ops:
        print("\nНет объектов для переименования.")
        return
    
    print(f"\nНайдено объектов для переименования: {len(rename_ops)}")
    print("-" * 70)
    
    # Показываем список
    for old, new in rename_ops[:20]:  # Первые 20 для превью
        print(f"  {old.relative_to(project_root)}")
        print(f"    -> {new.relative_to(project_root)}")
    if len(rename_ops) > 20:
        print(f"  ... и ещё {len(rename_ops) - 20}")
    
    print("-" * 70)
    
    # Если не dry-run, спрашиваем подтверждение
    if not dry_run:
        print("\nВНИМАНИЕ! Будут выполнены реальные переименования.")
        response = input("Продолжить? [y/N]: ").strip().lower()
        if response not in ('y', 'yes', 'да'):
            print("Отменено пользователем.")
            return
        print()
    
    # Выполняем переименования
    success_count, errors = execute_renames(rename_ops, dry_run=dry_run)
    
    print("\n" + "=" * 70)
    print("РЕЗУЛЬТАТЫ")
    print("=" * 70)
    print(f"  Всего запланировано переименований:  {len(rename_ops)}")
    print(f"  Успешно выполнено:                   {success_count}")
    print(f"  Ошибок:                              {len(errors)}")
    if dry_run:
        print("  [Это режим проверки, реальные изменения не вносились]")
    print("=" * 70)
    
    if errors:
        print("\nОшибки:")
        for err in errors:
            print(f"  - {err}")
        sys.exit(1)
    else:
        if not dry_run:
            print("\n✓ Готово!")


if __name__ == '__main__':
    main()
