// Git Diff Badge Info for task rows in ADE Sidebar.

class DiffBadgeInfo {
  final int addedLines;
  final int deletedLines;

  const DiffBadgeInfo({
    required this.addedLines,
    required this.deletedLines,
  });

  bool get hasChanges => addedLines > 0 || deletedLines > 0;

  String get formatted => '+$addedLines -$deletedLines';

  Map<String, dynamic> toJson() => {
        'added_lines': addedLines,
        'deleted_lines': deletedLines,
      };

  factory DiffBadgeInfo.fromJson(Map<String, dynamic> json) => DiffBadgeInfo(
        addedLines: json['added_lines'] as int? ?? 0,
        deletedLines: json['deleted_lines'] as int? ?? 0,
      );
}
